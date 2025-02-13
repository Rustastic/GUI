#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, Color32, Context, Pos2};

use crossbeam_channel::{Receiver, Sender};
use std::{collections::HashMap, time::{Duration, Instant}};

use colored::Colorize;
use log::{error, info, warn};

use wg_2024::{network::NodeId, packet::NodeType};

use messages::{
    gui_commands::{GUICommands, GUIEvents},
    high_level_messages::ServerType,
};

use crate::{helpers::types::ClientType, node::NodeGUI};

pub const HEIGHT: f32 = 900.0;
pub const WIDTH: f32 = 900.0;

#[derive(Clone, Debug)]
pub struct SimCtrlGUI {
    pub sender: Sender<GUICommands>,
    receiver: Receiver<GUIEvents>,

    pub initialized: bool,
    pub nodes: HashMap<NodeId, NodeGUI>,
    pub edges: HashMap<NodeId, (Vec<NodeId>, Color32)>,

    spawn_button: bool,
    spawn_toggle: bool,
    pub spawn_id: Option<String>,
    pub spawn_neighbors: Vec<NodeId>,
    pub spawn_pdr: Option<String>,
    pub spawn_command: Option<GUICommands>,

    // i choose to use a hashmap because if u ask from multiple client the file list in this way they can all be saved 
    pub file_list: HashMap<NodeId, Vec<String>>,
}

impl SimCtrlGUI {
    #[must_use]
    pub fn new(sender: Sender<GUICommands>, receiver: Receiver<GUIEvents>) -> Self {
        Self {
            sender,
            receiver,
            initialized: false,
            nodes: HashMap::new(),
            edges: HashMap::new(),

            spawn_button: true,
            spawn_toggle: false,
            spawn_id: None,
            spawn_neighbors: Vec::new(),
            spawn_pdr: None,
            spawn_command: None,
            file_list: HashMap::new(),
        }
    }
}

impl eframe::App for SimCtrlGUI {
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Not initialized from Network Initializer message
        if self.initialized {
            // Check for messages
            match self.receiver.try_recv() {
                Ok(event) => self.handle_events(event, ctx),
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => {
                        eprintln!(
                            "[ {} ]: GUICommands receiver channel disconnected: {}",
                            "Simulation Controller".red(),
                            e
                        );
                        return;
                    }
                },
            }

            // GUI
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Simulation Controller");

                // 
                let y_pos = 40.0;
                let x_pos = 10.0;

                let circles = [
                    (x_pos, Color32::LIGHT_BLUE, "Drone"),
                    (x_pos + 65.0, Color32::GREEN, "CommunicationServer"),
                    (x_pos + 220.0, Color32::BLUE, "TextContentServer"),
                    (x_pos + 352.5, Color32::RED, "MediaContentServer"),
                    (x_pos + 495.0, Color32::YELLOW, "ChatClient"),
                    (x_pos + 582.5, Color32::ORANGE, "MediaClient"),
                ];

                ui.horizontal(|ui| {
                    for (x, color, label) in circles {
                        ui.horizontal(|ui| {
                            let center = Pos2::new(x, y_pos);
                            ui.painter().add(egui::Shape::circle_filled(center, 5.0, color));
                            ui.add_space(15.0);
                            ui.label(label);
                        });
                        ui.add_space(5.0);
                    }
                });

                ui.add_space(10.0);

                // Spawn button
                if self.spawn_toggle {
                    ui.vertical(|ui| {
                        // Title
                        ui.heading("Spawn a New Drone");

                        // ID Input
                        ui.horizontal(|ui| {
                            ui.label("Enter Drone ID:");
                            let text_id = self.spawn_id.clone().unwrap_or_default().to_string();
                            let mut buffer_id = text_id.clone();

                            let text_edit = ui.text_edit_singleline(&mut buffer_id);
                            if text_edit.changed() {
                                self.spawn_id = Some(buffer_id);
                            }
                        });

                        // Multi-Select Neighbor Dropdown
                        ui.label("Select Neighbors:");
                        egui::ComboBox::from_label("Neighbors")
                            .selected_text(format!("{:?}", self.spawn_neighbors))
                            .show_ui(ui, |ui| {
                                for &neighbor in self.nodes.keys() {
                                    let label = format!("{neighbor}");
                                    let is_selected = self.spawn_neighbors.contains(&neighbor);

                                    if ui.selectable_label(is_selected, label).clicked() {
                                        if is_selected {
                                            self.spawn_neighbors.retain(|&n| n != neighbor);
                                        } else {
                                            self.spawn_neighbors.push(neighbor);
                                        }
                                    }
                                }
                            });

                        // PDR Input
                        ui.horizontal(|ui| {
                            ui.label("Enter PDR:");
                            let text_pdr = self.spawn_pdr.clone().unwrap_or_default().to_string();
                            let mut buffer_pdr = text_pdr.clone(); // Buffer for mutation

                            let text_edit = ui.text_edit_singleline(&mut buffer_pdr);
                            if text_edit.changed() {
                                self.spawn_pdr = Some(buffer_pdr);
                            }
                        });

                        // "Spawn" Button
                        if ui.button("Spawn").clicked() {
                            if let (Some(id_str), Some(pdr_str)) = (&self.spawn_id, &self.spawn_pdr) {
                                if let Ok(id) = id_str.parse::<u8>() {
                                    if let Ok(pdr) = pdr_str.parse::<f32>() {
                                        let neighbors = self.spawn_neighbors.clone();

                                        self.spawn_command = Some(GUICommands::Spawn(id, neighbors, pdr));

                                        self.spawn_toggle = false;
                                        self.spawn_button = true;

                                        info!("[ {} ] Spawning new Drone: {}", "GUI".green(), id);
                                    } else {
                                        error!("[ {} ] Invalid PDR value", "GUI".red());
                                    }
                                } else {
                                    error!("[ {} ] Invalid ID value", "GUI".red());
                                }
                            }
                        }
                    });
                }

                if self.spawn_button && ui.button("Spawn Drone").clicked() {
                    self.spawn_toggle = true;
                    self.spawn_button = false;
                    info!("[ {} ] Spawn button pressed", "GUI".green());
                }

                // Allocating space for drawing and preparing the painter for rendering
                let (_response, painter) =
                    ui.allocate_painter(egui::Vec2::new(WIDTH, HEIGHT), egui::Sense::hover());

                // Offset the painter by 100 pixels down
                let painter = painter.with_clip_rect(egui::Rect::from_min_max(
                    egui::pos2(0.0, 100.0),
                    egui::pos2(WIDTH, HEIGHT),
                ));


                // Drawing connections between drones/client
                for (start_id, neighbor) in self.edges.clone() {
                    let start = self.nodes.get(&start_id).unwrap();
                    for end_id in neighbor.0 {
                        let end = self.nodes.get(&end_id).unwrap();
                        painter.line_segment(
                            [egui::pos2(start.x, start.y), egui::pos2(end.x, end.y)],
                            egui::Stroke::new(2.0, neighbor.1),
                        );
                    }
                }

                // Drawing the drones/client
                for (_, pos) in &mut self.nodes.iter_mut() {
                    let screen_pos = egui::pos2(pos.x, pos.y);
                    let radius = 10.0;

                    let response = ui.allocate_rect(
                        egui::Rect::from_center_size(screen_pos, egui::Vec2::splat(radius * 2.0)),
                        egui::Sense::click(),
                    );

                    // Detecting if the drone is clicked and updating its selected status
                    if response.clicked() {
                        pos.selected = true;
                    }

                    // Draw the drone
                    painter.circle_filled(screen_pos, radius, pos.color);
                }

                let mut cclient_list = Vec::<NodeId>::new();
                for (id, instance) in &self.nodes {
                    if instance.node_type == NodeType::Client && instance.client_type.unwrap() == ClientType::Chat {
                        cclient_list.push(*id);
                    }
                }

                let mut mclient_list = Vec::<NodeId>::new();
                for (id, instance) in &self.nodes {
                    if instance.node_type == NodeType::Client && instance.client_type.unwrap() == ClientType::Media {
                        mclient_list.push(*id);
                    }
                }

                let mut cserver_list = Vec::<NodeId>::new();
                for (id, instance) in &self.nodes {
                    if instance.node_type == NodeType::Server && instance.server_type.unwrap() == ServerType::Chat {
                        cserver_list.push(*id);
                    }
                }

                let mut tserver_list = Vec::<NodeId>::new();
                for (id, instance) in &self.nodes {
                    if instance.node_type == NodeType::Server && instance.server_type.unwrap() == ServerType::Text {
                        tserver_list.push(*id);
                    }
                }

                let mut iserver_list = Vec::<NodeId>::new();
                for (id, instance) in &self.nodes {
                    if instance.node_type == NodeType::Server &&  instance.server_type.unwrap() == ServerType::Media {
                        iserver_list.push(*id);
                    }
                }

                for (_, instance) in &mut self.nodes.iter_mut() {
                    if let Some(start_time) = instance.last_packet_time {
                        if start_time.elapsed() > Duration::from_secs_f32(0.25) {
                            if instance.pending_reset {
                                instance.color = Color32::LIGHT_BLUE;
                            }
                            instance.pending_reset = false;
                        }
                    }
                }

                // Displaying a pop-up with drone's information
                for (_, instance) in &mut self.nodes.iter_mut() {

                    if instance.selected {
                        let title;
                        if instance.node_type == NodeType::Server {
                            title = format!("{:?}{:?} {}", instance.server_type.unwrap(),instance.node_type,  instance.id);
                        } else if instance.node_type == NodeType::Client {
                            title = format!("{:?}::{:?} {}", instance.client_type.unwrap(), instance.node_type, instance.id);
                        } else {
                            title = format!("{:?} {}",instance.node_type, instance.id);
                        }

                        egui::Window::new(title)
                            .fixed_size([100.0, 100.0]) // Window size
                            .resizable(false) // disable resizable
                            .collapsible(true) // activate collapsable
                            .show(ctx, |ui| {

                                if !instance.drone_params.crashed {

                                    // Display information
                                    ui.label(format!("Id: {}", instance.id));
                                    ui.label(format!(
                                        "Neighbors: {:?}",
                                        instance.neighbor
                                    ));
                                    if instance.node_type == NodeType::Drone {
                                        ui.label(format!("PDR: {}", instance.pdr));
                                    }
                                    ui.add_space(10.0);

                                    // Action buttons
                                    ui.horizontal_centered(|ui| {
                                        if ui.button("RemoveSender").clicked() {
                                            instance.remove_sender = !instance.remove_sender;
                                            instance.add_sender = false;
                                            instance.drone_params.set_pdr = false;
                                            instance.chat_params.send_message = false;
                                            instance.chat_params.register_to = false;
                                        }
                                        if ui.button("AddSender").clicked() {
                                            instance.add_sender = !instance.add_sender;
                                            instance.remove_sender = false;
                                            instance.drone_params.set_pdr = false;
                                            instance.chat_params.send_message = false;
                                            instance.chat_params.register_to = false;
                                        }
                                        if instance.node_type == NodeType::Drone {
                                            if ui.button("Crash").clicked() {
                                                instance.command = Some(GUICommands::Crash(instance.id));
                                            }
                                            if ui.button("SetPacketDropRate").clicked() {
                                                instance.drone_params.set_pdr = !instance.drone_params.set_pdr;
                                                instance.remove_sender = false;
                                                instance.add_sender = false;
                                            }
                                        } else if instance.node_type == NodeType::Client {
                                            if instance.client_type.unwrap() == ClientType::Chat {
                                                if ui.button("SendMessage").clicked() {
                                                    instance.chat_params.send_message = !instance.chat_params.send_message;
                                                    instance.add_sender = false;
                                                    instance.remove_sender = false;
                                                    instance.chat_params.register_to = false;
                                                    instance.chat_params.get_client_list = false;
                                                }
                                                if ui.button("GetClientList").clicked() {
                                                    instance.command = Some(GUICommands::GetClientList(instance.id));
                                                }
                                                if ui.button("RegisterTo").clicked() {
                                                    instance.chat_params.register_to = !instance.chat_params.register_to;
                                                    instance.add_sender = false;
                                                    instance.remove_sender = false;
                                                    instance.chat_params.send_message = false;
                                                    instance.chat_params.get_client_list = false;
                                                }
                                                if ui.button("LogOut").clicked() {
                                                    if let Some(server) = instance.chat_params.register_value {
                                                        instance.command = Some(GUICommands::LogOut(instance.id, server));
                                                    }
                                                }
                                            } else if ui.button("AskForFile").clicked() {
                                                instance.media_params.ask_for_file_list = !instance.media_params.ask_for_file_list;
                                                instance.add_sender = false;
                                                instance.remove_sender = false;
                                            }
                                        }
                                    });
                                }

                                // if not crashed
                                if !instance.drone_params.crashed && !instance.chat_params.logout {
                                    // if pressed RemoveSender button
                                    if instance.remove_sender {
                                        let value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Sender to remove: ")
                                            .selected_text(value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                // Get options
                                                let mut options = Vec::<String>::new();
                                                for numbers in instance.neighbor.clone() {
                                                    options.push(numbers.to_string());
                                                }

                                                // Check if option is selected
                                                for option in options {
                                                    if ui.selectable_label(false, &option).clicked() {
                                                        let value = Some(option.to_string());
                                                        instance.remove_sender = false;

                                                        if let Some(value_str) = value {
                                                            match value_str.parse::<u8>() {
                                                                Ok(digit) => {
                                                                    info!(
                                                                        "[ {} ] Passing to handler GUICommands::RemoveSender({}, {})",
                                                                        "GUI".green(),
                                                                        instance.id,
                                                                        digit
                                                                    );
                                                                    instance.command = Some(GUICommands::RemoveSender(instance.id, digit));
                                                                    instance.remove_sender = false;
                                                                },
                                                                Err(e) => error!(
                                                                    "[ {} ] Unable to parse neighbor NodeId in Crash GUICommand: {}",
                                                                    "GUI".red(),
                                                                    e
                                                                ),
                                                            }
                                                        } else {
                                                            error!("[ {} ] Value is None after selectable_label click. This is unexpected.", "GUI".red());
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    // if pressed AddSender button
                                    if instance.add_sender {
                                        let value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Sender to add: ")
                                            .selected_text(value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                // Get options
                                                let mut options = Vec::<String>::new();
                                                for numbers in self.edges.keys() {
                                                    if !instance.neighbor.contains(numbers) && *numbers != instance.id {
                                                        options.push(numbers.to_string());
                                                    }
                                                }

                                                // If something selected
                                                for option in options {
                                                    if ui.selectable_label(false, &option).clicked() {
                                                        let value = Some(option.to_string());
                                                        instance.remove_sender = false;

                                                        if let Some(value_str) = value {
                                                            match value_str.parse::<u8>() {
                                                                Ok(digit) => {
                                                                    info!(
                                                                        "[ {} ] Passing to handler GUICommands::AddSender({}, {})",
                                                                        "GUI".green(),
                                                                        instance.id,
                                                                        digit
                                                                    );
                                                                    instance.command = Some(GUICommands::AddSender(instance.id, digit));
                                                                    instance.add_sender = false;
                                                                },
                                                                Err(e) => {
                                                                    error!(
                                                                        "[ {} ] Unable to parse neighbor NodeId in GUICommand::AddSender: {}",
                                                                        "GUI".red(),
                                                                        e
                                                                    );
                                                                }
                                                            }
                                                        } else {
                                                            error!("[ {} ] option value is None after click. This is unexpected.", "GUI".red());
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    if instance.drone_params.set_pdr{
                                        ui.horizontal(|ui| {
                                            ui.label("Enter desired PDR:");

                                            // Default value
                                            let text_input = instance.drone_params.pdr_value.clone().unwrap_or_default();
                                            // Copy for mutation
                                            let mut buffer = text_input.clone();

                                            let text_edit = ui.text_edit_singleline(&mut buffer);

                                            // Update only if changed
                                            if text_edit.changed() {
                                                instance.drone_params.pdr_value = Some(buffer);
                                            }

                                            // "Confirm" button to process input
                                            if ui.button("Confirm").clicked() {
                                                if let Some(pdr_value) = &instance.drone_params.pdr_value {
                                                    match pdr_value.parse::<f32>() {
                                                        Ok(digit) => {
                                                            info!(
                                                                "[ {} ] Passing to handler GUICommands::SetPDR({}, {})",
                                                                "GUI".green(),
                                                                instance.id,
                                                                digit
                                                            );
                                                            instance.command = Some(GUICommands::SetPDR(instance.id, digit));
                                                            instance.drone_params.set_pdr = false;
                                                        }
                                                        Err(e) => error!("[ {} ] Invalid PDR input: {}", "GUI".red(), e),
                                                    }
                                                }
                                            }
                                        });
                                    }

                                    if instance.chat_params.send_message && instance.chat_params.client_list_value.is_some() {
                                        ui.vertical(|ui| {
                                            // Title
                                            ui.heading("Send a Message");

                                            // Multi-Select Neighbor Dropdown
                                            ui.label("Select a Client:");
                                            egui::ComboBox::from_label("Select Client: ")
                                                .selected_text(instance.chat_params.send_message_client_value.clone().unwrap_or("None".to_string()))
                                                .show_ui(ui, |ui| {
                                                    let iter_options = instance.chat_params.client_list_value.clone().unwrap();
                                                    let mut options = Vec::<String>::new();
                                                    for numbers in iter_options {
                                                        if numbers != instance.id {
                                                            options.push(numbers.to_string());
                                                        }
                                                    }

                                                    for option in options {
                                                        if ui.selectable_label(false, &option).clicked() {
                                                            instance.chat_params.send_message_client_value = Some(option.to_string());
                                                        }
                                                    }
                                                });

                                            // Message Input
                                            ui.horizontal(|ui| {
                                                ui.label("Enter Message:");
                                                let text_input = instance.chat_params.send_message_msg_value.clone().unwrap_or_default();
                                                let mut buffer = text_input.clone();

                                                let text_edit = ui.text_edit_singleline(&mut buffer);
                                                if text_edit.changed() {
                                                    instance.chat_params.send_message_msg_value = Some(buffer);
                                                }
                                            });

                                            // "Send" Button
                                            if ui.button("Send").clicked() {
                                                if let (Some(client), Some(message)) = (instance.chat_params.send_message_client_value.clone(), instance.chat_params.send_message_msg_value.clone()) {
                                                    if let Ok(client_id) = client.parse::<u8>() {
                                                        info!("[ {} ] Sending message to {}: {}", "GUI".green(), client_id, message);
                                                        instance.command = Some(GUICommands::SendMessageTo(instance.id, client_id, message));
                                                        instance.chat_params.send_message = false;
                                                        instance.chat_params.client_list_value = None;
                                                        instance.chat_params.send_message_client_value = None;
                                                        instance.chat_params.send_message_msg_value = None;
                                                    } else {
                                                        error!("[ {} ] Invalid client ID format", "GUI".red());
                                                    }
                                                } else {
                                                    error!("[ {} ] Missing client or message", "GUI".red());
                                                }
                                            }
                                        });
                                    }

                                    if instance.chat_params.register_to {
                                        let value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Server to register to: ")
                                            .selected_text(value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                // Get options
                                                let mut options = Vec::<String>::new();
                                                for numbers in &cserver_list {
                                                    options.push(numbers.to_string());
                                                }

                                                // If something selected
                                                for option in options {
                                                    if ui.selectable_label(false, &option).clicked() {
                                                        let value = Some(option.to_string());
                                                        instance.chat_params.register_to = false;

                                                        if let Some(value_str) = value {
                                                            match value_str.parse::<u8>() {
                                                                Ok(digit) => {
                                                                    instance.chat_params.register_value = Some(digit);
                                                                    info!(
                                                                        "[ {} ] Passing to handler GUICommands::RegisterTo({}, {})",
                                                                        "GUI".green(),
                                                                        instance.id,
                                                                        digit
                                                                    );
                                                                    instance.command = Some(GUICommands::RegisterTo(instance.id, digit));
                                                                },
                                                                Err(e) => {
                                                                    error!(
                                                                        "[ {} ] Unable to parse neighbor NodeId in GUICommand::RegisterTo: {}",
                                                                        "GUI".red(),
                                                                        e
                                                                    );
                                                                }
                                                            }
                                                        } else {
                                                            error!("[ {} ] Option value is None after click. This is unexpected.", "GUI".red());
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    if instance.media_params.ask_for_file_list {
                                        let value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Server to get List: ")
                                            .selected_text(value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                // Get options
                                                let mut options = Vec::<String>::new();
                                                for numbers in tserver_list.clone() {
                                                    options.push(numbers.to_string());
                                                }

                                                // Check if option is selected
                                                for option in options {
                                                    if ui.selectable_label(false, &option).clicked() {
                                                        let value = Some(option.to_string());
                                                        instance.media_params.ask_for_file_list = false;

                                                        if let Some(value_str) = value {
                                                            match value_str.parse::<u8>() {
                                                                Ok(digit) => {
                                                                    info!(
                                                                        "[ {} ] Passing to handler GUICommands::RemoveSender({}, {})",
                                                                        "GUI".green(),
                                                                        instance.id,
                                                                        digit
                                                                    );
                                                                    instance.media_params.server_value = Some(digit);
                                                                    instance.command = Some(GUICommands::AskForFileList(instance.id, digit));
                                                                    instance.media_params.ask_for_file_list = false;
                                                                    instance.media_params.get_file = true;
                                                                },
                                                                Err(e) => error!(
                                                                    "[ {} ] Unable to parse neighbor NodeId in Crash GUICommand: {}",
                                                                    "GUI".red(),
                                                                    e
                                                                ),
                                                            }
                                                        } else {
                                                            error!("[ {} ]: Option value is None after click. This is unexpected.", "GUI".red());
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    if instance.media_params.server_value.is_some() && self.file_list.contains_key(&instance.media_params.server_value.unwrap()) && instance.media_params.get_file {
                                        let value: Option<String> = None;
                                        egui::ComboBox::from_label("Select file: ")
                                            .selected_text(value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                // Get options
                                                let mut options = Vec::<String>::new();
                                                let vec = self.file_list.get(&instance.media_params.server_value.unwrap()).unwrap();
                                                for numbers in vec {
                                                    options.push(numbers.clone());
                                                }

                                                // If something selected
                                                for option in options {
                                                    if ui.selectable_label(false, &option).clicked() {
                                                        let value = Some(option.to_string());
                                                        instance.remove_sender = false;

                                                        if let Some(value_str) = value {
                                                            if let Some(server_value) = instance.media_params.server_value {
                                                                info!(
                                                                    "[ {} ] Passing to handler GUICommands::Get({}, {}, {:?})",
                                                                    "GUI".green(),
                                                                    instance.id,
                                                                    server_value,
                                                                    value_str
                                                                );
                                                                instance.command = Some(GUICommands::GetFile(instance.id, server_value, value_str));
                                                                instance.media_params.get_file = false;
                                                            } else {
                                                                error!("[ {} ] instance.server_value is None. Cannot execute GetFile command.", "GUI".red());
                                                            }
                                                        } else {
                                                            error!("[ {} ] Value is None after selectable_label click. This is unexpected.", "GUI".red());
                                                        }
                                                    }
                                                }
                                            });
                                            if ui.button("Exit").clicked() {
                                                instance.media_params.get_file = false;
                                            }
                                    }
                                }

                                ui.add_space(20.0);

                                if instance.node_type == NodeType::Client && instance.client_type.unwrap() == ClientType::Chat {
                                    match &instance.chat_params.recv_message_client_value {
                                        Some(msg) => {
                                            ui.label(format!("MessageReceived: {msg:?}"));
                                        }
                                        None => {
                                            ui.label("MessageReceived: None".to_string());
                                        }
                                    }
                                }

                                ui.add_space(20.0);

                                // Button to close the window
                                if ui.button("Close").clicked() {
                                    instance.selected = false;
                                }
                            });
                    }
                }

            });
        } else {
            warn!("[ {} ] Waiting for initialization", "GUI".green());
            // Wait for Topology message
            match self.receiver.try_recv() {
                Ok(event) => {
                    if let GUIEvents::Topology(_, _, _) = event.clone() {
                        self.handle_events(event, ctx);
                    } else {
                        error!(
                            "[ {} ] Received NON-Topology GUIEvent before Initialization",
                            "GUI".red()
                        );
                    }
                }
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => eprintln!(
                        "[ {} ]: GUICommands receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                },
            }
        }

        // Handle Commands
        self.handle_commands();

        ctx.request_repaint()
    }
}
