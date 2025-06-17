use colored::Colorize;
use eframe::egui;
use log::{error, info};
use std::time::Duration;
use wg_2024::packet::NodeType;

use crate::{
    constants::DRONE_COLOR,
    logic::{
        actions::{
            add_sender, ask_for_file_list, crash, get_file, get_list, logout, register,
            remove_sender, send_message, set_pdr,
        },
        nodes::{types::ClientType, NodeGUI},
        state::GUIState,
    },
    ui::network::NetworkVisualization,
};

impl NetworkVisualization {
    pub fn render_nodes(&mut self, state: &mut GUIState, ctx: &egui::Context) {
        // Update node colors based on packet animation timing
        if state.show_animation {
            Self::update_node_animations(state);
        }

        // Collect node IDs of selected nodes
        let mut selected_ids = Vec::new();
        for (id, instance) in &state.nodes {
            if instance.selected {
                selected_ids.push(*id);
            }
        }

        // Now mutate the actual instances
        for node_id in selected_ids {
            if let Some(mut instance) = state.nodes.remove(&node_id) {
                self.render_node_window(state, &mut instance, ctx);

                state.nodes.insert(node_id, instance);
            }
        }
    }

    #[allow(clippy::explicit_iter_loop)]
    pub fn show_animation(&self, state: &mut GUIState, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Show Animations").clicked() {
                state.show_animation = !state.show_animation;
                info!(
                    "[ {} ] Show animation: {}",
                    "GUI".green(),
                    state.show_animation
                );

                for (_, instance) in state.nodes.iter_mut() {
                    let color = self.get_node_color(instance);
                    instance.color = color;
                }
            }
        });
    }

    #[allow(clippy::explicit_iter_loop)]
    fn update_node_animations(state: &mut GUIState) {
        for (_, instance) in state.nodes.iter_mut() {
            if let Some(start_time) = instance.last_packet_time {
                if start_time.elapsed() > Duration::from_secs_f32(0.005) {
                    if instance.pending_reset && instance.node_type == NodeType::Drone {
                        instance.color = DRONE_COLOR;
                    }
                    instance.pending_reset = false;
                }
            }
        }
    }

    fn render_node_window(
        &self,
        state: &mut GUIState,
        instance: &mut NodeGUI,
        ctx: &egui::Context,
    ) {
        let title = Self::get_window_title(instance);

        egui::Window::new(title)
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| {
                if !instance.drone_params.crashed {
                    Self::render_node_info(ui, instance);
                    Self::render_action_buttons(state, ui, instance);
                    self.render_interactive_controls(state, ui, instance);
                }

                Self::render_status_info(ui, instance);

                ui.add_space(20.0);

                if ui.button("Close").clicked() {
                    instance.selected = false;
                }
            });
    }

    fn get_window_title(instance: &NodeGUI) -> String {
        match instance.node_type {
            NodeType::Server => {
                format!(
                    "{:?}{:?} {}",
                    instance.server_type.unwrap(),
                    instance.node_type,
                    instance.id
                )
            }
            NodeType::Client => {
                format!(
                    "{:?}::{:?} {}",
                    instance.client_type.unwrap(),
                    instance.node_type,
                    instance.id
                )
            }
            NodeType::Drone => {
                format!("{:?} {}", instance.node_type, instance.id)
            }
        }
    }

    fn render_node_info(ui: &mut egui::Ui, instance: &NodeGUI) {
        ui.label(format!("Id: {}", instance.id));
        ui.label(format!("Neighbors: {:?}", instance.neighbor));

        if instance.node_type == NodeType::Drone {
            ui.label(format!("PDR: {}", instance.pdr));
        }

        ui.add_space(10.0);
    }

    fn render_action_buttons(state: &mut GUIState, ui: &mut egui::Ui, instance: &mut NodeGUI) {
        ui.horizontal_wrapped(|ui| {
            // Common buttons for all node types
            if ui.button("RemoveSender").clicked() {
                Self::toggle_remove_sender(instance);
            }

            if ui.button("AddSender").clicked() {
                Self::toggle_add_sender(instance);
            }

            // Drone-specific buttons
            if instance.node_type == NodeType::Drone {
                if ui.button("Crash").clicked() {
                    crash(state, instance.id);
                }

                if ui.button("SetPacketDropRate").clicked() {
                    Self::toggle_set_pdr(instance);
                }
            }

            // Client-specific buttons
            if instance.node_type == NodeType::Client {
                Self::render_client_buttons(state, ui, instance);
            }
        });
    }

    fn render_client_buttons(state: &mut GUIState, ui: &mut egui::Ui, instance: &mut NodeGUI) {
        if let Some(ClientType::Chat) = instance.client_type {
            if ui.button("SendMessage").clicked() {
                Self::toggle_send_message(instance);
            }

            if ui.button("GetClientList").clicked() {
                get_list(state, instance.id);
            }

            if ui.button("RegisterTo").clicked() {
                Self::toggle_register_to(instance);
            }

            if ui.button("LogOut").clicked() {
                if let Some(server) = instance.chat_params.register_value {
                    logout(state, instance.id, server);
                }
            }
        } else if let Some(ClientType::Media) = instance.client_type {
            if ui.button("AskForFile").clicked() {
                Self::toggle_ask_for_file_list(instance);
            }
        }
    }

    fn render_interactive_controls(
        &self,
        state: &mut GUIState,
        ui: &mut egui::Ui,
        instance: &mut NodeGUI,
    ) {
        if !instance.drone_params.crashed && !instance.chat_params.logout {
            Self::render_sender_controls(state, ui, instance);
            Self::render_drone_controls(state, ui, instance);
            self.render_chat_controls(state, ui, instance);
            self.render_media_controls(state, ui, instance);
        }
    }

    fn render_sender_controls(state: &mut GUIState, ui: &mut egui::Ui, instance: &mut NodeGUI) {
        if instance.remove_sender {
            Self::render_remove_sender_dropdown(state, ui, instance);
        }

        if instance.add_sender {
            Self::render_add_sender_dropdown(state, ui, instance);
        }
    }

    fn render_remove_sender_dropdown(
        state: &mut GUIState,
        ui: &mut egui::Ui,
        instance: &mut NodeGUI,
    ) {
        egui::ComboBox::from_label("Select Sender to remove:")
            .selected_text("None")
            .show_ui(ui, |ui| {
                let mut options: Vec<String> =
                    instance.neighbor.iter().map(ToString::to_string).collect();
                options.sort_by_key(|s| s.parse::<i32>().unwrap_or(0));

                for option in options {
                    if ui.selectable_label(false, &option).clicked() {
                        if let Ok(digit) = option.parse::<u8>() {
                            instance.remove_sender = false;
                            remove_sender(state, instance.id, digit);
                        } else {
                            error!("[ {} ] Invalid neighbor ID: {}", "GUI".red(), option);
                        }
                    }
                }
            });
    }

    fn render_add_sender_dropdown(state: &mut GUIState, ui: &mut egui::Ui, instance: &mut NodeGUI) {
        egui::ComboBox::from_label("Select Sender to add:")
            .selected_text("None")
            .show_ui(ui, |ui| {
                let mut options: Vec<String> = state
                    .edges
                    .keys()
                    .filter(|&id| !instance.neighbor.contains(id) && *id != instance.id)
                    .map(ToString::to_string)
                    .collect();
                options.sort_by_key(|s| s.parse::<i32>().unwrap_or(0));

                for option in options {
                    if ui.selectable_label(false, &option).clicked() {
                        if let Ok(digit) = option.parse::<u8>() {
                            instance.add_sender = false;
                            add_sender(state, instance.id, digit);
                        } else {
                            error!("[ {} ] Invalid neighbor ID: {}", "GUI".red(), option);
                        }
                    }
                }
            });
    }

    fn render_drone_controls(state: &mut GUIState, ui: &mut egui::Ui, instance: &mut NodeGUI) {
        if instance.drone_params.set_pdr {
            ui.horizontal(|ui| {
                ui.label("Enter desired PDR:");

                let text_input = instance.drone_params.pdr_value.clone().unwrap_or_default();
                let mut buffer = text_input;

                let text_edit = ui.text_edit_singleline(&mut buffer);
                if text_edit.changed() {
                    instance.drone_params.pdr_value = Some(buffer);
                }

                if ui.button("Confirm").clicked() {
                    if let Some(pdr_value) = &instance.drone_params.pdr_value {
                        match pdr_value.parse::<f32>() {
                            Ok(pdr) => {
                                if (0.0..=1.0).contains(&pdr) {
                                    instance.drone_params.set_pdr = false;
                                    set_pdr(state, instance.id, pdr);
                                } else {
                                    error!(
                                        "[ {} ] Invalid PDR input: {}",
                                        "GUI".red(),
                                        "The PDR value must be between 0.0 and 1.0"
                                    );
                                }
                            }
                            Err(e) => error!("[ {} ] Invalid PDR input: {}", "GUI".red(), e),
                        }
                    }
                }
            });
        }
    }

    fn render_chat_controls(
        &self,
        state: &mut GUIState,
        ui: &mut egui::Ui,
        instance: &mut NodeGUI,
    ) {
        if instance.chat_params.send_message && instance.chat_params.client_list_value.is_some() {
            Self::render_send_message_form(state, ui, instance);
        }

        if instance.chat_params.register_to {
            self.render_register_to_dropdown(state, ui, instance);
        }
    }

    fn render_send_message_form(state: &mut GUIState, ui: &mut egui::Ui, instance: &mut NodeGUI) {
        ui.vertical(|ui| {
            ui.heading("Send a Message");

            // Client selection
            ui.label("Select a Client:");
            egui::ComboBox::from_label("Select Client:")
                .selected_text(
                    instance
                        .chat_params
                        .send_message_client_value
                        .clone()
                        .unwrap_or("None".to_string()),
                )
                .show_ui(ui, |ui| {
                    if let Some(client_list) = &instance.chat_params.client_list_value {
                        let mut options: Vec<String> = client_list
                            .iter()
                            .filter(|&&id| id != instance.id)
                            .map(ToString::to_string)
                            .collect();
                        options.sort_by_key(|s| s.parse::<i32>().unwrap_or(0));

                        for option in options {
                            if ui.selectable_label(false, &option).clicked() {
                                instance.chat_params.send_message_client_value = Some(option);
                            }
                        }
                    }
                });

            // Message input
            ui.horizontal(|ui| {
                ui.label("Enter Message:");
                let text_input = instance
                    .chat_params
                    .send_message_msg_value
                    .clone()
                    .unwrap_or_default();
                let mut buffer = text_input;

                let text_edit = ui.text_edit_singleline(&mut buffer);
                if text_edit.changed() {
                    instance.chat_params.send_message_msg_value = Some(buffer);
                }
            });

            // Send button
            if ui.button("Send").clicked() {
                if let (Some(client), Some(message)) = (
                    instance.chat_params.send_message_client_value.clone(),
                    instance.chat_params.send_message_msg_value.clone(),
                ) {
                    if let Ok(client_id) = client.parse::<u8>() {
                        instance.chat_params.send_message = false;
                        instance.chat_params.client_list_value = None;
                        instance.chat_params.send_message_client_value = None;
                        instance.chat_params.send_message_msg_value = None;

                        send_message(state, instance.id, client_id, &message);
                    } else {
                        error!("[ {} ] Invalid client ID format", "GUI".red());
                    }
                } else {
                    error!("[ {} ] Missing client or message", "GUI".red());
                }
            }
        });
    }

    fn render_register_to_dropdown(
        &self,
        state: &mut GUIState,
        ui: &mut egui::Ui,
        instance: &mut NodeGUI,
    ) {
        egui::ComboBox::from_label("Select Server to register to:")
            .selected_text("None")
            .show_ui(ui, |ui| {
                let mut comm_servers = self.get_communication_servers(state);
                comm_servers.sort_unstable();
                let options: Vec<String> = comm_servers.iter().map(|&x| x.to_string()).collect();

                for option in options {
                    if ui.selectable_label(false, &option).clicked() {
                        if let Ok(digit) = option.parse::<u8>() {
                            instance.chat_params.register_value = Some(digit);
                            instance.chat_params.register_to = false;

                            register(state, instance.id, digit);
                        } else {
                            error!("[ {} ] Invalid Server ID: {}", "GUI".red(), option);
                        }
                    }
                }
            });
    }

    fn render_media_controls(
        &self,
        state: &mut GUIState,
        ui: &mut egui::Ui,
        instance: &mut NodeGUI,
    ) {
        if instance.media_params.ask_for_file_list {
            self.render_file_server_dropdown(state, ui, instance);
        }

        if instance.media_params.server_value.is_some()
            && state
                .file_list
                .contains_key(&instance.media_params.server_value.unwrap())
            && instance.media_params.get_file
        {
            Self::render_file_selection(state, ui, instance);
        }
    }

    fn render_file_server_dropdown(
        &self,
        state: &mut GUIState,
        ui: &mut egui::Ui,
        instance: &mut NodeGUI,
    ) {
        egui::ComboBox::from_label("Select Server to get List:")
            .selected_text("None")
            .show_ui(ui, |ui| {
                let mut text_server = self.get_text_content_servers(state);
                text_server.sort_unstable();
                let options: Vec<String> = text_server.iter().map(|&x| x.to_string()).collect();

                for option in options {
                    if ui.selectable_label(false, &option).clicked() {
                        if let Ok(digit) = option.parse::<u8>() {
                            instance.media_params.server_value = Some(digit);
                            instance.media_params.ask_for_file_list = false;
                            instance.media_params.get_file = true;

                            ask_for_file_list(state, instance.id, digit);
                        } else {
                            error!("[ {} ] Invalid Server ID: {}", "GUI".red(), option);
                        }
                    }
                }
            });
    }

    fn render_file_selection(state: &mut GUIState, ui: &mut egui::Ui, instance: &mut NodeGUI) {
        egui::ComboBox::from_label("Select file:")
            .selected_text("None")
            .show_ui(ui, |ui| {
                if let Some(server_id) = instance.media_params.server_value {
                    if let Some(file_list) = state.file_list.get(&server_id) {
                        let mut options = file_list.clone();
                        options.sort();

                        for option in options {
                            if ui.selectable_label(false, &option).clicked() {
                                instance.media_params.get_file = false;

                                get_file(state, instance.id, server_id, &option);
                            }
                        }
                    }
                }
            });

        if ui.button("Exit").clicked() {
            instance.media_params.get_file = false;
        }
    }

    fn render_status_info(ui: &mut egui::Ui, instance: &NodeGUI) {
        if instance.node_type == NodeType::Client {
            if let Some(ClientType::Chat) = instance.client_type {
                match &instance.chat_params.recv_message_client_value {
                    Some(msg) => {
                        ui.label(format!("MessageReceived: {msg:?}"));
                    }
                    None => {
                        ui.label("MessageReceived: None");
                    }
                }
            }
        }
    }

    // Helper methods for button toggles
    fn toggle_remove_sender(instance: &mut NodeGUI) {
        instance.remove_sender = !instance.remove_sender;
        instance.add_sender = false;
        instance.drone_params.set_pdr = false;
        instance.chat_params.send_message = false;
        instance.chat_params.register_to = false;
        instance.media_params.ask_for_file_list = false;
    }

    fn toggle_add_sender(instance: &mut NodeGUI) {
        instance.add_sender = !instance.add_sender;
        instance.remove_sender = false;
        instance.drone_params.set_pdr = false;
        instance.chat_params.send_message = false;
        instance.chat_params.register_to = false;
        instance.media_params.ask_for_file_list = false;
    }

    fn toggle_set_pdr(instance: &mut NodeGUI) {
        instance.drone_params.set_pdr = !instance.drone_params.set_pdr;
        instance.remove_sender = false;
        instance.add_sender = false;
    }

    fn toggle_send_message(instance: &mut NodeGUI) {
        instance.chat_params.send_message = !instance.chat_params.send_message;
        instance.add_sender = false;
        instance.remove_sender = false;
        instance.chat_params.register_to = false;
        instance.chat_params.get_client_list = false;
    }

    fn toggle_register_to(instance: &mut NodeGUI) {
        instance.chat_params.register_to = !instance.chat_params.register_to;
        instance.add_sender = false;
        instance.remove_sender = false;
        instance.chat_params.send_message = false;
        instance.chat_params.get_client_list = false;
    }

    fn toggle_ask_for_file_list(instance: &mut NodeGUI) {
        instance.media_params.ask_for_file_list = !instance.media_params.ask_for_file_list;
        instance.add_sender = false;
        instance.remove_sender = false;
    }
}
