use eframe::egui::Color32;
use petgraph::{graph::NodeIndex, Graph, Undirected};
use rand::Rng;
use std::collections::HashMap;

use colored::Colorize;
use log::info;

use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone, Server as ConfigServer},
    network::NodeId
};

use messages::high_level_messages::ServerType;

use crate::{constants::{HEIGHT, WIDTH}, logic::{nodes::{types::ClientType, NodeGUI}, state::GUIState}};

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

pub fn topology(
    state: &mut GUIState,
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

    let coordinates = fruchterman_reingold(&graph, 500, WIDTH - 25.0, HEIGHT - 25.0);

    for drone in drones {
        let (x, y) = coordinates.get(vertexes.get(&drone.id).unwrap()).unwrap();
        let new_drone = NodeGUI::new_drone(drone, *x, *y);

        for drone in new_drone.neighbor.clone() {
            if !state.edges.contains_key(&drone) {
                let (vec, _) = state
                    .edges
                    .entry(new_drone.id)
                    .or_insert_with(|| (Vec::new(), Color32::GRAY));
                vec.push(drone);
            }
        }

        state.nodes.insert(new_drone.id, new_drone);
    }

    let half = clients.len() / 2;
    for (count, client) in clients.iter().enumerate() {
        let (x, y) = coordinates.get(vertexes.get(&client.id).unwrap()).unwrap();
        let new_client = if count < half {
            NodeGUI::new_client(client, *x, *y, Some(ClientType::Chat))
        } else {
            NodeGUI::new_client(client, *x, *y, Some(ClientType::Media))
        };

        state
            .edges
            .entry(new_client.id)
            .or_insert_with(|| (Vec::new(), Color32::GRAY));

        state.nodes.insert(new_client.id, new_client);
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

        state
            .edges
            .entry(new_server.id)
            .or_insert_with(|| (Vec::new(), Color32::GRAY));

        state.nodes.insert(new_server.id, new_server);

        count -= 1;
    }

    info!("[ {} ] Successfully composed the topology", "GUI".green());
    state.initialized = true;
}
