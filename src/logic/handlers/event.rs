use std::time::Instant;

use crossbeam_channel::TryRecvError;

use colored::Colorize;
use eframe::egui::{self, Color32};
use log::{error, info};

use messages::gui_commands::GUIEvents;
use wg_2024::packet::{NodeType, PacketType};

use crate::logic::{actions::topology, state::GUIState};

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_events(&self, state: &mut GUIState, ctx: &egui::Context) {
        match state.receiver.try_recv() {
            Ok(event) => self.process_event(state, event, ctx),
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
                    self.process_event(state, event, ctx);
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

    fn process_event(&self, state: &mut GUIState, event: GUIEvents, ctx: &egui::Context) {
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
            GUIEvents::PacketDropped(src, _) => {
                if let Some(node) = state.nodes.get_mut(&src) {
                    node.color = Color32::RED;
                }
                state.nodes.get_mut(&src).unwrap().last_packet_time = Some(Instant::now());
                state.nodes.get_mut(&src).unwrap().pending_reset = true;
                ctx.request_repaint();
            }
            GUIEvents::MessageReceived(src, dest, msg) => {
                info!(
                    "[ {} ]: [ Client {} ] received the message {:?} from [ Client {} ]",
                    "GUI".green(),
                    dest,
                    msg,
                    src
                );
                let formatted_msg = format!("[{}] {}", src, msg);
                state
                    .nodes
                    .get_mut(&dest)
                    .unwrap()
                    .chat_params
                    .recv_message_client_value = Some(formatted_msg);
            }
        }
    }
}
