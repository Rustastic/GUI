use std::{collections::HashMap, f32::consts::PI};

use colored::Colorize;
use log::{error, info};
use petgraph::{graph::NodeIndex, Graph, Undirected};
use rand::Rng;
use wg_2024::{config::Drone as ConfigDrone, network::NodeId};

use crate::{commands::GUICommands, DroneGUI, SimCtrlGUI, HEIGHT, WIDTH};

fn fruchterman_reingold(
    graph: &Graph<(), (), Undirected>,
    iterations: usize,
    max_width: f32,
    max_height: f32,
) -> HashMap<NodeIndex, (f32, f32)> {
    let node_count = graph.node_count();
    let mut positions: HashMap<NodeIndex, (f32, f32)> = HashMap::with_capacity(node_count);

    // 1. Initialize node positions (randomly, within bounds)
    let mut rng = rand::rng(); // Use thread_rng for better randomness
    for node_index in graph.node_indices() {
        positions.insert(
            node_index,
            (
                rng.random_range(0.0..max_width),
                rng.random_range(0.0..max_height),
            ),
        );
    }

    let k = 100.0; // Repulsion strength (adjust as needed)
    let attraction_multiplier = 0.05; // Attraction strength (adjust as needed)
    let mut temperature = 5.0; // Start with a high temperature

    for _ in 0..iterations {
        let mut displacements: HashMap<NodeIndex, (f32, f32)> = HashMap::with_capacity(node_count);
        for node_index in graph.node_indices() {
            displacements.insert(node_index, (0.0, 0.0));
        }

        let mut distances: HashMap<(NodeIndex, NodeIndex), f32> = HashMap::new();
        for i in graph.node_indices() {
            for j in graph.node_indices() {
                if i != j {
                    let dx = positions[&j].0 - positions[&i].0;
                    let dy = positions[&j].1 - positions[&i].1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    distances.insert((i, j), distance);
                }
            }
        }

        // 2. Calculate repulsive forces (optimized)
        for i in graph.node_indices() {
            for j in graph.node_indices() {
                if i != j {
                    let distance = distances[&(i, j)];
                    if distance > 0.0 {
                        let repulsion_force = k / distance;
                        let dx = positions[&j].0 - positions[&i].0;
                        let dy = positions[&j].1 - positions[&i].1;

                        *displacements.get_mut(&i).unwrap() = (
                            displacements[&i].0 - repulsion_force * dx / distance,
                            displacements[&i].1 - repulsion_force * dy / distance,
                        );
                        *displacements.get_mut(&j).unwrap() = (
                            displacements[&j].0 + repulsion_force * dx / distance,
                            displacements[&j].1 + repulsion_force * dy / distance,
                        );
                    }
                }
            }
        }

        // 3. Calculate attractive forces (optimized)
        for edge in graph.edge_indices() {
            let (u, v) = graph.edge_endpoints(edge).unwrap();
            let dx = positions[&v].0 - positions[&u].0;
            let dy = positions[&v].1 - positions[&u].1;
            let attraction_force = attraction_multiplier; // No need for sqrt or normalization

            *displacements.get_mut(&u).unwrap() = (
                displacements[&u].0 + attraction_force * dx,
                displacements[&u].1 + attraction_force * dy,
            );
            *displacements.get_mut(&v).unwrap() = (
                displacements[&v].0 - attraction_force * dx,
                displacements[&v].1 - attraction_force * dy,
            );
        }

        // 4. Update positions (with temperature-controlled limit)
        let max_displacement = temperature * f32::min(max_width, max_height);
        for node_index in graph.node_indices() {
            let displacement = displacements.get(&node_index).unwrap();
            let displacement_magnitude = (displacement.0 * displacement.0 + displacement.1 * displacement.1).sqrt();

            if displacement_magnitude > 0.0 {
                let scale = f32::min(1.0, max_displacement / displacement_magnitude);
                let new_x = positions[&node_index].0 + displacement.0 * scale;
                let new_y = positions[&node_index].1 + displacement.1 * scale;

                positions.insert(
                    node_index,
                    (new_x.clamp(0.0, max_width), new_y.clamp(0.0, max_height)),
                );
            }
        }

        temperature *= 0.99; // Cool down
    }

    positions
}

pub fn topology(sim_ctrl: &mut SimCtrlGUI, topology: Vec<ConfigDrone>) {
    let mut graph = Graph::<(), (), Undirected>::new_undirected();
    let mut vertexes = HashMap::<NodeId, NodeIndex>::new();

    for drone in topology.iter() {
        let vertex_id = graph.add_node(());
        vertexes.insert(drone.id, vertex_id);
    }

    for drone in topology.iter() {
        for neighbor in drone.connected_node_ids.iter() {
            graph.add_edge(
                *vertexes.get(&drone.id).unwrap(),
                *vertexes.get(neighbor).unwrap(),
                (),
            );
        }
    }

    let coordinates = fruchterman_reingold(&graph, 300, WIDTH, HEIGHT);

    for drone in topology.iter() {
        let (x, y) = coordinates.get(vertexes.get(&drone.id).unwrap()).unwrap();
        let new_drone = DroneGUI::new(drone.clone(), *x, *y);

        for drone in new_drone.neighbor.clone() {
            if !sim_ctrl.edges.contains_key(&drone) {
                let vec = sim_ctrl.edges.entry(new_drone.id).or_insert_with(Vec::new);
                vec.push(drone);
            }
        }

        sim_ctrl.nodes.insert(new_drone.id, new_drone);
    }

    info!("[ {} ] Successfully composed the topology", "GUI".green());
    sim_ctrl.initialized = true;
}

pub fn crash(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl.sender.send(GUICommands::Crash(instance.id)) {
        Ok(()) => {
            info!(
                "[ {} ] Successfully sent GUICommand::Crash from GUI to Simulation Controller",
                "GUI".green()
            );
            // remove from edge hashmap
            sim_ctrl.edges.remove(&instance.id);

            // remove edges starting from neighbor
            for neighbor_id in instance.neighbor.iter() {
                // get edges starting from neighbor
                if let Some(neighbor_drone) = sim_ctrl.edges.get_mut(neighbor_id) {
                    neighbor_drone.retain(|drone| *drone != instance.id);
                }
            }

            instance.command = None;

            let neighbors = sim_ctrl.nodes.get(drone).unwrap().neighbor.clone();
            let id = sim_ctrl.nodes.get(drone).unwrap().id.clone();
            for node in neighbors {
                let a = sim_ctrl.nodes.get_mut(&node).unwrap();
                a.neighbor.retain(|&x| x != id);
            }

            let id = sim_ctrl.nodes.get(drone).unwrap().id;
            sim_ctrl.nodes.remove(&id);
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::Crash from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }
}

pub fn remove_sender(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId, to_remove: &NodeId) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl
        .sender
        .send(GUICommands::RemoveSender(instance.id, *to_remove))
    {
        Ok(_) => {
            info!(
                "[ {} ] Successfully sent GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                instance.id,
                to_remove
            );

            if let Some(edge) = sim_ctrl.edges.get_mut(&instance.id) {
                if edge.contains(to_remove) {
                    edge.retain(|&node| node != *to_remove);
                }
            }
            if let Some(edge) = sim_ctrl.edges.get_mut(to_remove) {
                if edge.contains(&instance.id) {
                    edge.retain(|&node| node != instance.id);
                }
            }

            // Remove neighbor from the current instance.
            instance.neighbor.retain(|&x| x != *to_remove);
            instance.command = None;

            // Remove neighbor from to_remove
            let id = sim_ctrl.nodes.get(drone).unwrap().id.clone();
            let neighbor = sim_ctrl.nodes.get_mut(to_remove).unwrap();
            neighbor.neighbor.retain(|&x| x != id);
        }
        Err(e) => {
            error!(
                "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            )
        }
    }
}

pub fn add_sender(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId, to_add: NodeId) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl
        .sender
        .send(GUICommands::AddSender(instance.id, to_add))
    {
        Ok(_) => {
            info!(
                "[ {} ] Successfully sent GUICommand::AddSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                instance.id,
                to_add
            );

            instance.neighbor.push(to_add);
            sim_ctrl
                .edges
                .entry(*drone)
                .or_insert_with(Vec::new)
                .push(to_add);

            let neighbor = sim_ctrl.nodes.get_mut(&to_add).unwrap();
            neighbor.neighbor.push(*drone);
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }

    sim_ctrl.nodes.get_mut(drone).unwrap().command = None;
}

pub fn set_pdr(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId, pdr: &f32) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl.sender.send(GUICommands::SetPDR(instance.id, *pdr)) {
        Ok(_) => {
            info!("[ {} ] Successfully sent GUICommand::SetPDR({}, {}) from GUI to Simulation Controller", "GUI".green(), instance.id, pdr);
            instance.pdr = *pdr;
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::SetPDR from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }

    instance.command = None;
}

pub fn spawn(sim_ctrl: &mut SimCtrlGUI, id: &NodeId, neighbors: &Vec<NodeId>, pdr: f32) {
    match sim_ctrl
        .sender
        .send(GUICommands::Spawn(*id, neighbors.clone(), pdr))
    {
        Ok(()) => {
            sim_ctrl.spawn_id = None;
            sim_ctrl.spawn_neighbors.clear();
            sim_ctrl.spawn_pdr = None;

            let drone = ConfigDrone {
                id: *id,
                connected_node_ids: neighbors.clone(),
                pdr,
            };

            // add to nodes
            let new_drone = DroneGUI::new(drone, 400.0, 400.0);

            // ad to various instances neighbors
            for drone in neighbors {
                sim_ctrl
                    .nodes
                    .get_mut(drone)
                    .unwrap()
                    .neighbor
                    .push(new_drone.id);
            }

            sim_ctrl.nodes.insert(*id, new_drone);

            // add edges
            sim_ctrl.edges.insert(*id, neighbors.clone());

            sim_ctrl.spawn_command = None;

            info!("[ {} ] Successfully sent GUICommand::Spawn({}, {:?}, {}) from GUI to Simulation Controller", "GUI".green(), id, neighbors, pdr)
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::Spawn from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    };
}
