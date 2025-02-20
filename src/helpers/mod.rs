use eframe::egui::Color32;
use petgraph::{graph::NodeIndex, Graph, Undirected};
use rand::Rng;
use std::collections::HashMap;

use colored::Colorize;
use log::{error, info};

use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone, Server as ConfigServer},
    network::NodeId,
    packet::NodeType,
};

use messages::{gui_commands::GUICommands, high_level_messages::ServerType};

use crate::{node::NodeGUI, SimCtrlGUI, HEIGHT, WIDTH};
use types::ClientType;

pub mod handlers;
pub mod types;

impl SimCtrlGUI {
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
            let mut displacements: HashMap<NodeIndex, (f32, f32)> =
                HashMap::with_capacity(node_count);
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
                        (new_x.clamp(25.0, max_width), new_y.clamp(125.0, max_height)),
                    );
                }
            }

            // decrease temperature to make future changes less important
            temperature *= 0.99;
        }

        positions
    }

    pub(super) fn topology(
        &mut self,
        drones: &[ConfigDrone],
        clients: &[ConfigClient],
        servers: &[ConfigServer],
    ) {
        let mut graph = Graph::<(), (), Undirected>::new_undirected();
        let mut vertexes = HashMap::<NodeId, NodeIndex>::new();

        for drone in drones {
            let vertex_id = graph.add_node(());
            vertexes.insert(drone.id, vertex_id);
        }

        for client in clients {
            let vertex_id = graph.add_node(());
            vertexes.insert(client.id, vertex_id);
        }

        for server in servers {
            let vertex_id = graph.add_node(());
            vertexes.insert(server.id, vertex_id);
        }

        for drone in drones {
            for neighbor in &drone.connected_node_ids {
                graph.add_edge(
                    *vertexes.get(&drone.id).unwrap(),
                    *vertexes.get(neighbor).unwrap(),
                    (),
                );
            }
        }

        for client in clients {
            for neighbor in &client.connected_drone_ids {
                graph.add_edge(
                    *vertexes.get(&client.id).unwrap(),
                    *vertexes.get(neighbor).unwrap(),
                    (),
                );
            }
        }

        for server in servers {
            for neighbor in &server.connected_drone_ids {
                graph.add_edge(
                    *vertexes.get(&server.id).unwrap(),
                    *vertexes.get(neighbor).unwrap(),
                    (),
                );
            }
        }

        let coordinates = Self::fruchterman_reingold(&graph, 500, WIDTH - 25.0, HEIGHT - 25.0);

        for drone in drones {
            let (x, y) = coordinates.get(vertexes.get(&drone.id).unwrap()).unwrap();
            let new_drone = NodeGUI::new_drone(drone, *x, *y);

            for drone in new_drone.neighbor.clone() {
                if !self.edges.contains_key(&drone) {
                    let (vec, _) = self
                        .edges
                        .entry(new_drone.id)
                        .or_insert_with(|| (Vec::new(), Color32::GRAY));
                    vec.push(drone);
                }
            }

            self.nodes.insert(new_drone.id, new_drone);
        }

        let half = clients.len() / 2;
        for (count, client) in clients.iter().enumerate() {
            let (x, y) = coordinates.get(vertexes.get(&client.id).unwrap()).unwrap();
            let new_client = if count < half {
                NodeGUI::new_client(client, *x, *y, Some(ClientType::Chat))
            } else {
                NodeGUI::new_client(client, *x, *y, Some(ClientType::Media))
            };

            self.edges
                .entry(new_client.id)
                .or_insert_with(|| (Vec::new(), Color32::GRAY));

            self.nodes.insert(new_client.id, new_client);
        }

        let third = servers.len() / 3;
        let mut count = servers.len();
        for server in servers {
            let (x, y) = coordinates.get(vertexes.get(&server.id).unwrap()).unwrap();

            let new_server;
            if count > (third * 2) {
                new_server = NodeGUI::new_server(server, *x, *y, Some(ServerType::Text));
            } else if count > third {
                new_server = NodeGUI::new_server(server, *x, *y, Some(ServerType::Media));
            } else {
                new_server = NodeGUI::new_server(server, *x, *y, Some(ServerType::Chat));
            }

            self.edges
                .entry(new_server.id)
                .or_insert_with(|| (Vec::new(), Color32::GRAY));

            self.nodes.insert(new_server.id, new_server);

            count -= 1;
        }

        info!("[ {} ] Successfully composed the topology", "GUI".green());
        self.initialized = true;
    }

    pub(super) fn crash(&mut self, drone: NodeId) {
        let instance = self.nodes.get_mut(&drone).unwrap();
        match self.sender.send(GUICommands::Crash(instance.id)) {
            Ok(()) => {
                info!(
                    "[ {} ] Successfully sent GUICommand::Crash from GUI to Simulation Controller",
                    "GUI".green()
                );
                // remove from edge hashmap
                self.edges.remove(&instance.id);

                // remove edges starting from neighbor
                for neighbor_id in &instance.neighbor {
                    // get edges starting from neighbor
                    if let Some(neighbor_drone) = self.edges.get_mut(neighbor_id) {
                        neighbor_drone.0.retain(|drone| *drone != instance.id);
                    }
                }

                instance.command = None;

                let neighbors = self.nodes.get(&drone).unwrap().neighbor.clone();
                let id = self.nodes.get(&drone).unwrap().id;
                for node in neighbors {
                    let a = self.nodes.get_mut(&node).unwrap();
                    a.neighbor.retain(|&x| x != id);
                }

                let id = self.nodes.get(&drone).unwrap().id;
                self.nodes.remove(&id);
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

    pub(super) fn remove_sender(&mut self, node_id: NodeId, to_remove: NodeId) {
        if self.nodes.get(&node_id).unwrap().node_type == NodeType::Client
            && self.nodes.get(&node_id).unwrap().neighbor.len() == 1
        {
            self.nodes.get_mut(&node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::RemoveSender to [ Client {} ] from GUI to Simulation Controller: {}",
                "GUI".red(),
                node_id,
                "Each client must be connected to at least one and at most two drones"
            );
            return;
        } else if self.nodes.get(&to_remove).unwrap().node_type == NodeType::Client
            && self.nodes.get(&to_remove).unwrap().neighbor.len() == 1
        {
            self.nodes.get_mut(&node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::RemoveSender to [ Client {} ] from GUI to Simulation Controller: {}",
                "GUI".red(),
                to_remove,
                "Each client must be connected to at least one and at most two drones"
            );
            return;
        }

        if self.nodes.get(&node_id).unwrap().node_type == NodeType::Server
            && self.nodes.get(&node_id).unwrap().neighbor.len() == 2
        {
            self.nodes.get_mut(&node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::RemoveSender to [ Server {} ] from GUI to Simulation Controller: {}",
                "GUI".red(),
                node_id,
                "Each server must be connected to least two drones"
            );
            return;
        } else if self.nodes.get(&to_remove).unwrap().node_type == NodeType::Server
            && self.nodes.get(&to_remove).unwrap().neighbor.len() == 2
        {
            self.nodes.get_mut(&node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::RemoveSender to [ Server {} ] from GUI to Simulation Controller: {}",
                "GUI".red(),
                to_remove,
                "Each server must be connected to least two drones"
            );
            return;
        }

        let instance = self.nodes.get_mut(&node_id).unwrap();
        match self
            .sender
            .send(GUICommands::RemoveSender(instance.id, to_remove))
        {
            Ok(()) => {
                info!(
                    "[ {} ] Successfully sent GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller",
                    "GUI".green(),
                    instance.id,
                    to_remove
                );

                if let Some(edge) = self.edges.get_mut(&instance.id) {
                    if edge.0.contains(&to_remove) {
                        edge.0.retain(|&node| node != to_remove);
                    }
                }
                if let Some(edge) = self.edges.get_mut(&to_remove) {
                    if edge.0.contains(&instance.id) {
                        edge.0.retain(|&node| node != instance.id);
                    }
                }

                // Remove neighbor from the current instance.
                instance.neighbor.retain(|&x| x != to_remove);
                instance.command = None;

                // Remove neighbor from to_remove
                let id = self.nodes.get(&node_id).unwrap().id;
                let neighbor = self.nodes.get_mut(&to_remove).unwrap();
                neighbor.neighbor.retain(|&x| x != id);
            }
            Err(e) => {
                error!(
                    "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
                    "GUI".red(),
                    e
                );
            }
        }
    }

    pub(super) fn add_sender(&mut self, node_id: NodeId, to_add: NodeId) {
        if self.nodes.get(&node_id).unwrap().node_type == NodeType::Client
            && self.nodes.get(&to_add).unwrap().node_type == NodeType::Client
        {
            self.nodes.get_mut(&node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                "Two clients can be connected between eachother"
            );
            return;
        } else if self.nodes.get(&node_id).unwrap().node_type == NodeType::Client {
            if self.nodes.get(&node_id).unwrap().neighbor.len() == 2 {
                self.nodes.get_mut(&node_id).unwrap().command = None;
                error!(
                    "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
                    "GUI".red(),
                    "Each client must be connected to at most two drones"
                );
                return;
            }
        } else if self.nodes.get(&to_add).unwrap().node_type == NodeType::Client
            && self.nodes.get(&to_add).unwrap().neighbor.len() == 2
        {
            self.nodes.get_mut(&node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                "Each client must be connected to at most two drones"
            );
            return;
        }

        let instance = self.nodes.get_mut(&node_id).unwrap();
        match self
            .sender
            .send(GUICommands::AddSender(instance.id, to_add))
        {
            Ok(()) => {
                info!(
                    "[ {} ] Successfully sent GUICommand::AddSender({}, {}) from GUI to Simulation Controller",
                    "GUI".green(),
                    instance.id,
                    to_add
                );

                instance.neighbor.push(to_add);
                let (vec, _) = self
                    .edges
                    .entry(node_id)
                    .or_insert_with(|| (Vec::new(), Color32::GRAY));
                vec.push(to_add);

                let neighbor = self.nodes.get_mut(&to_add).unwrap();
                neighbor.neighbor.push(node_id);
            }
            Err(e) => error!(
                "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            ),
        }

        self.nodes.get_mut(&node_id).unwrap().command = None;
    }

    pub(super) fn set_pdr(&mut self, drone: NodeId, pdr: f32) {
        let instance = self.nodes.get_mut(&drone).unwrap();
        match self.sender.send(GUICommands::SetPDR(instance.id, pdr)) {
            Ok(()) => {
                info!("[ {} ] Successfully sent GUICommand::SetPDR({}, {}) from GUI to Simulation Controller", "GUI".green(), instance.id, pdr);
                instance.pdr = pdr;
            }
            Err(e) => error!(
                "[ {} ] Unable to send GUICommand::SetPDR from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            ),
        }

        instance.command = None;
    }

    pub(super) fn spawn(&mut self, id: NodeId, neighbors: &Vec<NodeId>, pdr: f32) {
        if self.nodes.contains_key(&id) {
            error!(
                "[ {} ]: failed to spawn a new Drone because a Drone with Id {} already exists",
                "GUI".red(),
                id
            );
        } else {
            for node in neighbors {
                let instance = self.nodes.get(node).unwrap();
                if instance.node_type == NodeType::Client && instance.neighbor.len() == 2 {
                    error!(
                        "[ {} ]: failed to spawn a new Drone [ Client {} ] is already connected to 2 drones",
                        "GUI".red(),
                        instance.id
                    );
                    self.spawn_command = None;
                    return;
                }
            }

            if (0.0..=1.0).contains(&pdr) {
                match self
                    .sender
                    .send(GUICommands::Spawn(id, neighbors.clone(), pdr))
                {
                    Ok(()) => {
                        self.spawn_id = None;
                        self.spawn_neighbors.clear();
                        self.spawn_pdr = None;

                        let drone = ConfigDrone {
                            id,
                            connected_node_ids: neighbors.clone(),
                            pdr,
                        };

                        // add to nodes
                        let mut rng = rand::rng();
                        let (x, y) = (rng.random_range(0.0..WIDTH), rng.random_range(0.0..HEIGHT));
                        let new_drone = NodeGUI::new_drone(&drone, x, y);

                        // ad to various instances neighbors
                        for drone in neighbors {
                            self.nodes
                                .get_mut(drone)
                                .unwrap()
                                .neighbor
                                .push(new_drone.id);
                        }

                        self.nodes.insert(id, new_drone);

                        // add edges
                        self.edges.insert(id, (neighbors.clone(), Color32::GRAY));

                        info!("[ {} ] Successfully sent GUICommand::Spawn({}, {:?}, {}) from GUI to Simulation Controller", "GUI".green(), id, neighbors, pdr);
                    }
                    Err(e) => error!(
                        "[ {} ] Unable to send GUICommand::Spawn from GUI to Simulation Controller: {}",
                        "GUI".red(),
                        e
                    ),
                }
            } else {
                error!(
                    "[ {} ]: The PDR number is out of range. Please enter a number between 0.00 and 1.00",
                    "GUI".red(),
                );
            }   
        }

        self.spawn_command = None;
    }

    pub(super) fn send_message(&mut self, src: NodeId, dest: NodeId, msg: &str) {
        match self.sender.send(GUICommands::SendMessageTo(src, dest, msg.to_string())) {
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
        self.nodes.get_mut(&src).unwrap().command = None;
    }

    pub(super) fn register(&mut self, client: NodeId, server: NodeId) {
        match self.sender.send(GUICommands::RegisterTo(client, server)) {
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
        self.nodes.get_mut(&client).unwrap().command = None;
    }

    pub(super) fn get_list(&mut self, client: NodeId) {
        match self.sender.send(GUICommands::GetClientList(client)) {
            Ok(()) => info!(
                "[ {} ] Successfully sent GUICommand::GetClientList({}) from GUI to Simulation Controller",
                "GUI".green(),
                client,
            ),
            Err(e) => error!("[ {} ] Unable to send GUICommand::GetClientList({}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                client,
                e
            ),
        }
        self.nodes.get_mut(&client).unwrap().command = None;
    }

    pub(super) fn logout(&mut self, client: NodeId, server: NodeId) {
        match self.sender.send(GUICommands::LogOut(client, server)) {
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
        self.nodes.get_mut(&client).unwrap().command = None;
    }

    pub(super) fn ask_for_file_list(&mut self, client: NodeId, server: NodeId) {
        match self.sender.send(GUICommands::AskForFileList(client, server)) {
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
        self.nodes.get_mut(&client).unwrap().command = None;
    }

    pub(super) fn get_file(&mut self, client: NodeId, server: NodeId, title: &str) {
        match self.sender.send(GUICommands::GetFile(client, server, title.to_string())) {
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
        self.nodes.get_mut(&client).unwrap().command = None;
    }
}
