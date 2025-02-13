use std::time::Instant;

use eframe::egui::{self, Color32};
use messages::{gui_commands::GUICommands, high_level_messages::ServerType};
use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone, Server as ConfigServer},
    network::NodeId,
    packet::NodeType,
};

use crate::helpers::types::ClientType;

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

#[derive(Debug, Clone)]
pub struct DroneParams {
    pub crashed: bool,
    pub set_pdr: bool,
    pub pdr_value: Option<String>,
}

impl DroneParams {
    fn new() -> Self {
        Self {
            crashed: false,
            set_pdr: false,
            pdr_value: None,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct ChatParam {
    pub send_message: bool,
    pub send_message_msg_value: Option<String>,
    pub send_message_client_value: Option<String>,
    pub recv_message_client_value: Option<String>,
    pub register_to: bool,
    pub register_value: Option<NodeId>,
    pub get_client_list: bool,
    pub client_list_value: Option<Vec<NodeId>>,
    pub logout: bool,
}

impl ChatParam {
    fn new() -> Self {
        Self {
            send_message: false,
            send_message_msg_value: None,
            send_message_client_value: None,
            recv_message_client_value: None,
            register_to: false,
            register_value: None,
            get_client_list: false,
            client_list_value: None,
            logout: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaParams {
    pub ask_for_file_list: bool,
    pub server_value: Option<NodeId>,
    pub get_file: bool,
}

impl MediaParams {
    fn new() -> Self {
        Self {
            ask_for_file_list: false,
            server_value: None,
            get_file: false,
        }
    }
}

impl NodeGUI {
    #[must_use]
    pub fn new_drone(drone: &ConfigDrone, x: f32, y: f32) -> Self {
        Self {
            id: drone.id,
            neighbor: drone.connected_node_ids.clone(),
            pdr: drone.pdr,
            x,
            y,
            node_type: NodeType::Drone,
            client_type: None,
            server_type: None,
            color: Color32::LIGHT_BLUE,

            command: None,

            selected: false,
            remove_sender: false,
            add_sender: false,

            drone_params: DroneParams::new(),
            chat_params: ChatParam::new(),
            media_params: MediaParams::new(),

            last_packet_time: None,
            pending_reset: false
        }
    }

    #[must_use]
    pub fn new_client(
        client: &ConfigClient,
        x: f32,
        y: f32,
        client_type: Option<ClientType>,
    ) -> Self {
        let mut color = Color32::PLACEHOLDER;
        if let Some(ctype) = client_type {
            match ctype {
                ClientType::Chat => color = Color32::YELLOW,
                ClientType::Media => color = Color32::ORANGE,
            }
        }

        Self {
            id: client.id,
            neighbor: client.connected_drone_ids.clone(),
            pdr: 0.0,
            x,
            y,
            node_type: NodeType::Client,
            client_type,
            server_type: None,
            color,

            command: None,

            selected: false,
            remove_sender: false,
            add_sender: false,

            drone_params: DroneParams::new(),
            chat_params: ChatParam::new(),
            media_params: MediaParams::new(),

            last_packet_time: None,
            pending_reset: false
        }
    }

    #[must_use]
    pub fn new_server(
        server: &ConfigServer,
        x: f32,
        y: f32,
        server_type: Option<ServerType>,
    ) -> Self {
        let mut color = Color32::PLACEHOLDER;
        if let Some(stype) = server_type {
            match stype {
                ServerType::Chat => color = Color32::GREEN,
                ServerType::Text => color = Color32::PURPLE,
                ServerType::Media => color = Color32::RED,
            }
        }

        Self {
            id: server.id,
            neighbor: server.connected_drone_ids.clone(),
            pdr: 0.0,
            x,
            y,
            node_type: NodeType::Server,
            client_type: None,
            server_type,
            color,

            command: None,

            selected: false,
            remove_sender: false,
            add_sender: false,

            drone_params: DroneParams::new(),
            chat_params: ChatParam::new(),
            media_params: MediaParams::new(),

            last_packet_time: None,
            pending_reset: false
        }
    }
}
