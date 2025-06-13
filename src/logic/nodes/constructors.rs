// src/nodes/constructors.rs
use eframe::egui::Color32;
use messages::high_level_messages::ServerType;
use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone, Server as ConfigServer},
    packet::NodeType,
};

use crate::logic::nodes::types::ClientType;
use super::{NodeGUI, DroneParams, ChatParam, MediaParams};

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
            pending_reset: false,
        }
    }

    #[must_use]
    pub fn new_client(
        client: &ConfigClient,
        x: f32,
        y: f32,
        client_type: Option<ClientType>,
    ) -> Self {
        let color = match client_type {
            Some(ClientType::Chat) => Color32::YELLOW,
            Some(ClientType::Media) => Color32::ORANGE,
            None => Color32::PLACEHOLDER,
        };

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
            pending_reset: false,
        }
    }

    #[must_use]
    pub fn new_server(
        server: &ConfigServer,
        x: f32,
        y: f32,
        server_type: Option<ServerType>,
    ) -> Self {
        let color = match server_type {
            Some(ServerType::Chat) => Color32::GREEN,
            Some(ServerType::Text) => Color32::PURPLE,
            Some(ServerType::Media) => Color32::RED,
            None => Color32::PLACEHOLDER,
        };

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
            pending_reset: false,
        }
    }
}