use eframe::egui;
use messages::{gui_commands::GUICommands, high_level_messages::ServerType};
use std::time::Instant;
use wg_2024::{network::NodeId, packet::NodeType};

use super::{ChatParam, DroneParams, MediaParams};
use crate::logic::nodes::types::ClientType;

#[derive(Clone, Debug)]
pub struct NodeGUI {
    pub id: NodeId,
    pub neighbor: Vec<NodeId>,
    pub pdr: f32,
    pub x: f32,
    pub y: f32,
    pub node_type: NodeType,
    pub client_type: Option<ClientType>,
    pub server_type: Option<ServerType>,
    pub color: egui::Color32,

    pub command: Option<GUICommands>,

    pub selected: bool,
    pub remove_sender: bool,
    pub add_sender: bool,

    pub drone_params: DroneParams,
    pub chat_params: ChatParam,
    pub media_params: MediaParams,

    pub last_packet_time: Option<Instant>,
    pub pending_reset: bool,
}
