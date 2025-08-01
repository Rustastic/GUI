use std::time::Instant;

use crossbeam_channel::TryRecvError;

use colored::Colorize;
use eframe::egui::{self, Color32};
use log::{error, info};

use messages::gui_commands::GUIEvents;
use rand::Rng;
use wg_2024::{
    config::Drone as ConfigDrone,
    packet::{NodeType, PacketType},
};

use crate::{
    constants::{HEIGHT, NODE_RADIUS, WIDTH},
    logic::{actions::topology, nodes::NodeGUI, state::GUIState},
};

pub struct EventHandler;

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn handle_events(&self, state: &mut GUIState, ctx: &egui::Context) {
        match state.receiver.try_recv() {
            Ok(event) => Self::process_event(state, event, ctx),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                error!(
                    "[ {} ]: GUICommands receiver channel disconnected",
                    "GUI".red()
                );
            }
        }
    }

    pub fn handle_initialization(&self, state: &mut GUIState, ctx: &egui::Context) {
        match state.receiver.try_recv() {
            Ok(event) => {
                if let GUIEvents::Topology(_, _, _) = event {
                    Self::process_event(state, event, ctx);
                } else {
                    error!(
                        "[ {} ] Received NON-Topology GUIEvent before Initialization",
                        "GUI".red()
                    );
                }
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                eprintln!(
                    "[ {} ]: GUICommands receiver channel disconnected during init",
                    "Simulation Controller".red()
                );
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn process_event(state: &mut GUIState, event: GUIEvents, ctx: &egui::Context) {
        match event {
            GUIEvents::Topology(drones, clients, servers) => {
                info!("[ {} ]: Received Topology", "GUI".green());
                topology(state, &drones, &clients, &servers);
            }
            GUIEvents::FileList(server, _, items) => {
                info!(
                    "[ {} ]: Received FileList from [ Client {} ]",
                    "GUI".green(),
                    server
                );
                state.file_list.insert(server, items);
            }
            GUIEvents::ClientList(client, client_list) => {
                info!(
                    "[ {} ]: Received ClientList from [ Client {} ]",
                    "GUI".green(),
                    client
                );
                state
                    .nodes
                    .get_mut(&client)
                    .unwrap()
                    .chat_params
                    .client_list_value = Some(client_list);
            }
            GUIEvents::PacketSent(src, _, packet) => {
                if state.show_animation {
                    if let Some(node) = state.nodes.get_mut(&src) {
                        if node.node_type == NodeType::Drone {
                            match packet.pack_type {
                                PacketType::MsgFragment(_) => node.color = Color32::BLUE,
                                PacketType::Ack(_) => node.color = Color32::DARK_GREEN,
                                PacketType::Nack(_) => node.color = Color32::DARK_RED,
                                PacketType::FloodRequest(_) => node.color = Color32::WHITE,
                                PacketType::FloodResponse(_) => node.color = Color32::DARK_GRAY,
                            }
                        }
                    }
                    state.nodes.get_mut(&src).unwrap().last_packet_time = Some(Instant::now());
                    state.nodes.get_mut(&src).unwrap().pending_reset = true;
                    ctx.request_repaint();
                }
            }
            GUIEvents::PacketDropped(src, _) => {
                if state.show_animation {
                    if let Some(node) = state.nodes.get_mut(&src) {
                        node.color = Color32::RED;
                    }
                    state.nodes.get_mut(&src).unwrap().last_packet_time = Some(Instant::now());
                    state.nodes.get_mut(&src).unwrap().pending_reset = true;
                    ctx.request_repaint();
                }
            }
            GUIEvents::MessageReceived(src, dest, msg) => {
                info!(
                    "[ {} ]: [ Client {} ] received the message {:?} from [ Client {} ]",
                    "GUI".green(),
                    dest,
                    msg,
                    src
                );
                let formatted_msg = format!("[{src}] -> {msg}");
                state
                    .nodes
                    .get_mut(&dest)
                    .unwrap()
                    .chat_params
                    .recv_message_client_value = Some(formatted_msg);
            }
            GUIEvents::RemoveSender(node_id, to_remove) => {
                if let Some(edge) = state.edges.get_mut(&node_id) {
                    if edge.0.contains(&to_remove) {
                        edge.0.retain(|&node| node != to_remove);
                    }
                }

                let instance = state.nodes.get_mut(&node_id).unwrap();

                // Remove neighbor from the current instance.
                instance.neighbor.retain(|&x| x != to_remove);
            }
            GUIEvents::AddSender(node_id, to_add) => {
                let neighbor = state.nodes.get_mut(&node_id).unwrap();
                neighbor.neighbor.push(to_add);
                let (vec, _) = state
                    .edges
                    .entry(node_id)
                    .or_insert_with(|| (Vec::new(), Color32::GRAY));
                vec.push(to_add);
            }
            GUIEvents::Spawn(id, neighbors, pdr) => {
                state.spawn.id = None;
                state.spawn.neighbors.clear();
                state.spawn.pdr = None;

                let drone = ConfigDrone {
                    id,
                    connected_node_ids: neighbors.clone(),
                    pdr,
                };

                // add to nodes
                let mut rng = rand::rng();
                let (x, y) = (
                    rng.random_range((100.0 + NODE_RADIUS)..(WIDTH - NODE_RADIUS)),
                    rng.random_range((100.0 + NODE_RADIUS)..(HEIGHT - NODE_RADIUS)),
                );
                let new_drone = NodeGUI::new_drone(&drone, x, y);

                state.nodes.insert(id, new_drone);

                info!(
                    "[ {} ] Successfully created Drone({}, {:?}, {})",
                    "GUI".green(),
                    id,
                    neighbors,
                    pdr
                );
            }
            GUIEvents::Crash(drone) => {
                // remove from edge hashmap
                state.edges.remove(&drone);

                let instance = state.nodes.get_mut(&drone).unwrap();

                // remove edges starting from neighbor
                for neighbor_id in &instance.neighbor {
                    // get edges starting from neighbor
                    if let Some(neighbor_drone) = state.edges.get_mut(neighbor_id) {
                        neighbor_drone.0.retain(|x| *x != drone);
                    }
                }

                let neighbors = state.nodes.get(&drone).unwrap().neighbor.clone();
                let id = state.nodes.get(&drone).unwrap().id;
                for node in neighbors {
                    let a = state.nodes.get_mut(&node).unwrap();
                    a.neighbor.retain(|&x| x != id);
                }

                let id = state.nodes.get(&drone).unwrap().id;
                state.nodes.remove(&id);
            }
        }
    }
}
