#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::Color32;
use petgraph::{graph::NodeIndex, Graph, Undirected};
use rand::Rng;
use std::collections::HashMap;

use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone, Server as ConfigServer},
    network::NodeId,
    packet::NodeType,
};

use colored::Colorize;
use log::{error, info};

use crate::{commands::{ClientType, GUICommands, ServerType}, NodeGUI, SimCtrlGUI, HEIGHT, WIDTH};

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

    let k = 100.0; // Repulsion strength
    let attraction_multiplier = 0.05; // Attraction strength
    let mut temperature = 4.0; // Start with a high temperature

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

        // 2. Calculate repulsive forces
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

        // 3. Calculate attractive forces
        for edge in graph.edge_indices() {
            let (u, v) = graph.edge_endpoints(edge).unwrap();
            let dx = positions[&v].0 - positions[&u].0;
            let dy = positions[&v].1 - positions[&u].1;
            let attraction_force = attraction_multiplier;

            *displacements.get_mut(&u).unwrap() = (
                displacements[&u].0 + attraction_force * dx,
                displacements[&u].1 + attraction_force * dy,
            );
            *displacements.get_mut(&v).unwrap() = (
                displacements[&v].0 - attraction_force * dx,
                displacements[&v].1 - attraction_force * dy,
            );
        }

        // 4. Update positions with temperature
        let max_displacement = temperature * f32::min(max_width, max_height);
        for node_index in graph.node_indices() {
            let displacement = displacements.get(&node_index).unwrap();
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

        // decrease temperature to make future changes less important
        temperature *= 0.99;
    }

    positions
}

pub fn topology(
    sim_ctrl: &mut SimCtrlGUI,
    drones: Vec<ConfigDrone>,
    clients: Vec<ConfigClient>,
    servers: Vec<ConfigServer>,
) {
    let mut graph = Graph::<(), (), Undirected>::new_undirected();
    let mut vertexes = HashMap::<NodeId, NodeIndex>::new();

    for drone in drones.iter() {
        let vertex_id = graph.add_node(());
        vertexes.insert(drone.id, vertex_id);
    }

    for client in clients.iter() {
        let vertex_id = graph.add_node(());
        vertexes.insert(client.id, vertex_id);
    }

    for server in servers.iter() {
        let vertex_id = graph.add_node(());
        vertexes.insert(server.id, vertex_id);
    }

    for drone in drones.iter() {
        for neighbor in drone.connected_node_ids.iter() {
            graph.add_edge(
                *vertexes.get(&drone.id).unwrap(),
                *vertexes.get(neighbor).unwrap(),
                (),
            );
        }
    }

    for client in clients.iter() {
        for neighbor in client.connected_drone_ids.iter() {
            graph.add_edge(
                *vertexes.get(&client.id).unwrap(),
                *vertexes.get(neighbor).unwrap(),
                (),
            );
        }
    }

    for server in servers.iter() {
        for neighbor in server.connected_drone_ids.iter() {
            graph.add_edge(
                *vertexes.get(&server.id).unwrap(),
                *vertexes.get(neighbor).unwrap(),
                (),
            );
        }
    }

    let coordinates = fruchterman_reingold(&graph, 300, WIDTH, HEIGHT);

    for drone in drones.iter() {
        let (x, y) = coordinates.get(vertexes.get(&drone.id).unwrap()).unwrap();
        let new_drone = NodeGUI::new_drone(drone.clone(), *x, *y);

        for drone in new_drone.neighbor.clone() {
            if !sim_ctrl.edges.contains_key(&drone) {
                let (vec, _) = sim_ctrl
                    .edges
                    .entry(new_drone.id)
                    .or_insert_with(|| (Vec::new(), Color32::GRAY));
                vec.push(drone);
            }
        }

        sim_ctrl.nodes.insert(new_drone.id, new_drone);
    }

    let half = clients.len() / 2;
    for (count, client) in clients.iter().enumerate() {
        let (x, y) = coordinates.get(vertexes.get(&client.id).unwrap()).unwrap();
        let new_client;
        if count < half {
            new_client = NodeGUI::new_client(client.clone(), *x, *y, Some(ClientType::Chat));
        } else {
            new_client = NodeGUI::new_client(client.clone(), *x, *y, Some(ClientType::Media));
        }


        if !sim_ctrl.edges.contains_key(&new_client.id) {
            sim_ctrl
                .edges
                .insert(new_client.id, (Vec::new(), Color32::GRAY));
        }

        sim_ctrl.nodes.insert(new_client.id, new_client);
    }

    let third = servers.len() / 3;
    let mut count = servers.len();
    for server in servers.iter() {
        let (x, y) = coordinates.get(vertexes.get(&server.id).unwrap()).unwrap();

        let new_server;
        if count > (third * 2) {
            new_server = NodeGUI::new_server(server.clone(), *x, *y, Some(ServerType::Communication));
        } else if count > third {
            new_server = NodeGUI::new_server(server.clone(), *x, *y, Some(ServerType::Image));
        } else {
            new_server = NodeGUI::new_server(server.clone(), *x, *y, Some(ServerType::Text));
        }

        if !sim_ctrl.edges.contains_key(&new_server.id) {
            sim_ctrl
                .edges
                .insert(new_server.id, (Vec::new(), Color32::GRAY));
        }

        sim_ctrl.nodes.insert(new_server.id, new_server);

        count -= 1;
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
                    neighbor_drone.0.retain(|drone| *drone != instance.id);
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
        Err(e) => {
            error!(
                "[ {} ] Unable to send GUICommand::Crash from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            );
        }
    }
}

pub fn remove_sender(sim_ctrl: &mut SimCtrlGUI, node_id: &NodeId, to_remove: &NodeId) {
    if sim_ctrl.nodes.get(node_id).unwrap().node_type == NodeType::Client
        && sim_ctrl.nodes.get(node_id).unwrap().neighbor.len() == 1
    {
        sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            "Each client must be connected to at least one and at most two drones"
        );
        return;
    } else if sim_ctrl.nodes.get(&to_remove).unwrap().node_type == NodeType::Client
        && sim_ctrl.nodes.get(&to_remove).unwrap().neighbor.len() == 1
    {
        sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            "Each client must be connected to at least one and at most two drones"
        );
        return;
    }

    if sim_ctrl.nodes.get(node_id).unwrap().node_type == NodeType::Server
        && sim_ctrl.nodes.get(node_id).unwrap().neighbor.len() == 2
    {
        sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            "Each server must be connected to least two drones"
        );
        return;
    } else if sim_ctrl.nodes.get(to_remove).unwrap().node_type == NodeType::Server
        && sim_ctrl.nodes.get(to_remove).unwrap().neighbor.len() == 2
    {
        sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            "Each server must be connected to least two drones"
        );
        return;
    }

    let instance = sim_ctrl.nodes.get_mut(node_id).unwrap();
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
                if edge.0.contains(to_remove) {
                    edge.0.retain(|&node| node != *to_remove);
                }
            }
            if let Some(edge) = sim_ctrl.edges.get_mut(to_remove) {
                if edge.0.contains(&instance.id) {
                    edge.0.retain(|&node| node != instance.id);
                }
            }

            // Remove neighbor from the current instance.
            instance.neighbor.retain(|&x| x != *to_remove);
            instance.command = None;

            // Remove neighbor from to_remove
            let id = sim_ctrl.nodes.get(node_id).unwrap().id.clone();
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

pub fn add_sender(sim_ctrl: &mut SimCtrlGUI, node_id: &NodeId, to_add: NodeId) {
    if sim_ctrl.nodes.get(node_id).unwrap().node_type == NodeType::Client
        && sim_ctrl.nodes.get(&to_add).unwrap().node_type == NodeType::Client
    {
        sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            "Two clients can be connected between eachother"
        );
        return;
    } else if sim_ctrl.nodes.get(node_id).unwrap().node_type == NodeType::Client {
        if sim_ctrl.nodes.get(node_id).unwrap().neighbor.len() == 2 {
            sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                "Each client must be connected to at most two drones"
            );
            return;
        }
    } else if sim_ctrl.nodes.get(&to_add).unwrap().node_type == NodeType::Client {
        if sim_ctrl.nodes.get(&to_add).unwrap().neighbor.len() == 2 {
            sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                "Each client must be connected to at most two drones"
            );
            return;
        }
    }

    let instance = sim_ctrl.nodes.get_mut(node_id).unwrap();
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
            let (vec, _) = sim_ctrl
                .edges
                .entry(*node_id)
                .or_insert_with(|| (Vec::new(), Color32::GRAY));
            vec.push(to_add);

            let neighbor = sim_ctrl.nodes.get_mut(&to_add).unwrap();
            neighbor.neighbor.push(*node_id);
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }

    sim_ctrl.nodes.get_mut(node_id).unwrap().command = None;
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
            let mut rng = rand::rng();
            let (x, y) = (rng.random_range(0.0..WIDTH), rng.random_range(0.0..HEIGHT));
            let new_drone = NodeGUI::new_drone(drone, x, y);

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
            sim_ctrl
                .edges
                .insert(*id, (neighbors.clone(), Color32::GRAY));

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

pub fn send_message(sim_ctrl: &mut SimCtrlGUI, src: &NodeId, dest: &NodeId, msg: String) {
    match sim_ctrl.sender.send(GUICommands::SendMessageTo(*src, *dest, msg.clone())) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::SendMessageTo({}, {}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            src,
            dest,
            msg
        ),
        Err(e) => error!("[ {} ] Unable to send GUICommand::SendMessageTo({}, {}, {}) from GUI to Simulation Controller: {}",
            "GUI".red(),
            src,
            dest,
            msg,
            e
        ),
    }
    sim_ctrl.nodes.get_mut(src).unwrap().command = None;
}

pub fn register(sim_ctrl: &mut SimCtrlGUI, client: &NodeId, server: &NodeId) {
    match sim_ctrl.sender.send(GUICommands::RegisterTo(*client, *server)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::RegisterTo({}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server
        ),
        Err(e) => error!("[ {} ] Unable to send GUICommand::RegisterTo({}, {}) from GUI to Simulation Controller: {}",
            "GUI".red(),
            client,
            server,
            e
        ),
    }
    sim_ctrl.nodes.get_mut(client).unwrap().command = None;
}

pub fn logout(sim_ctrl: &mut SimCtrlGUI, client: &NodeId, server: &NodeId) {
    match sim_ctrl.sender.send(GUICommands::LogOut(*client, *server)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::LogOut({}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server
        ),
        Err(e) => error!("[ {} ] Unable to send GUICommand::LogOut({}, {}) from GUI to Simulation Controller: {}",
            "GUI".red(),
            client,
            server,
            e
        ),
    }
    sim_ctrl.nodes.get_mut(client).unwrap().command = None;
}

pub fn ask_for_file_list(sim_ctrl: &mut SimCtrlGUI, client: &NodeId, server: &NodeId) {
    match sim_ctrl.sender.send(GUICommands::AskForFileList(*client, *server)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::AskForFileList({}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server
        ),
        Err(e) => error!("[ {} ] Unable to send GUICommand::AskForFileList({}, {}) from GUI to Simulation Controller: {}",
            "GUI".red(),
            client,
            server,
            e
        ),
    }
    sim_ctrl.nodes.get_mut(client).unwrap().command = None;
}

pub fn get_file(sim_ctrl: &mut SimCtrlGUI, client: &NodeId, server: &NodeId, title: String) {
    match sim_ctrl.sender.send(GUICommands::GetFile(*client, *server, title.clone())) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::GetFile({}, {}, {:?}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server,
            title
        ),
        Err(e) => error!("[ {} ] Unable to send GUICommand::GetFile({}, {}, {:?}) from GUI to Simulation Controller: {}",
            "GUI".red(),
            client,
            server,
            title,
            e
        ),
    }
    sim_ctrl.nodes.get_mut(client).unwrap().command = None;
}