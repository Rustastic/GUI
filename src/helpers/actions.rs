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
    let mut rng = rand::rng();
    for node_index in graph.node_indices() {
        positions.insert(
            node_index,
            (
                rng.random_range(0.0..max_width),
                rng.random_range(0.0..max_height),
            ),
        );
    }

    let k = 0.2; // Repulsion strength
    let attraction_multiplier = 0.1; // Attraction strength

    for _ in 0..iterations {
        let mut displacements: HashMap<NodeIndex, (f32, f32)> = HashMap::with_capacity(node_count);
        for node_index in graph.node_indices() {
            displacements.insert(node_index, (0.0, 0.0));
        }

        // 2. Calculate repulsive forces
        for i in graph.node_indices() {
            for j in graph.node_indices() {
                if i != j {
                    let dx = positions[&j].0 - positions[&i].0;
                    let dy = positions[&j].1 - positions[&i].1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance > 0.0 {
                        let repulsion_force = k / distance;
                        *displacements.get_mut(&i).unwrap() = (
                            displacements[&i].0 - repulsion_force * dx / distance,
                            displacements[&i].1 - repulsion_force * dy / distance,
                        );
                    }
                }
            }
        }

        // 3. Calculate attractive forces
        for edge in graph.edge_indices() {
            let (u, v) = graph.edge_endpoints(edge).unwrap();
            let dx = positions[&v].0 - positions[&u].0;
            let dy = positions[&v].1 - positions[&u].1;
            let attraction_force = attraction_multiplier * (dx * dx + dy * dy).sqrt();
            *displacements.get_mut(&u).unwrap() = (
                displacements[&u].0 + attraction_force * dx / (dx * dx + dy * dy).sqrt(),
                displacements[&u].1 + attraction_force * dy / (dx * dx + dy * dy).sqrt(),
            );
            *displacements.get_mut(&v).unwrap() = (
                displacements[&v].0 - attraction_force * dx / (dx * dx + dy * dy).sqrt(),
                displacements[&v].1 - attraction_force * dy / (dx * dx + dy * dy).sqrt(),
            );
        }

        // 4. Update positions (with a limit to prevent wild swings)
        let max_displacement = 0.1 * f32::min(max_width, max_height);
        for node_index in graph.node_indices() {
            let mut displacement = (0.0, 0.0);

            // Calculate net displacement for this node (from repulsion and attraction forces)
            for other_node in graph.node_indices() {
                if node_index != other_node {
                    let dx = positions[&other_node].0 - positions[&node_index].0;
                    let dy = positions[&other_node].1 - positions[&node_index].1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance > 0.0 {
                        let repulsion_force = k / distance;
                        displacement.0 -= repulsion_force * dx / distance;
                        displacement.1 -= repulsion_force * dy / distance;
                    }
                }
            }

            for edge in graph.edge_indices() {
                let (u, v) = graph.edge_endpoints(edge).unwrap();
                if u == node_index || v == node_index {
                    let other_node = if u == node_index { v } else { u };
                    let dx = positions[&other_node].0 - positions[&node_index].0;
                    let dy = positions[&other_node].1 - positions[&node_index].1;
                    let attraction_force = attraction_multiplier * (dx * dx + dy * dy).sqrt();
                    if u == node_index {
                        displacement.0 += attraction_force * dx / (dx * dx + dy * dy).sqrt();
                        displacement.1 += attraction_force * dy / (dx * dx + dy * dy).sqrt();
                    } else {
                        displacement.0 -= attraction_force * dx / (dx * dx + dy * dy).sqrt();
                        displacement.1 -= attraction_force * dy / (dx * dx + dy * dy).sqrt();
                    }
                }
            }

            let displacement_magnitude =
                (displacement.0 * displacement.0 + displacement.1 * displacement.1).sqrt();
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

    let coordinates = fruchterman_reingold(&graph, 150, WIDTH, HEIGHT);

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
