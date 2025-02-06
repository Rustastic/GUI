use crossbeam_channel::{Receiver, Sender};
use std::{collections::HashMap, num, thread};

use colored::Colorize;
use eframe::egui::{self, Color32};

use log::{error, info, warn};
use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone},
    network::NodeId,
    packet::NodeType,
};

use crate::{
    actions,
    commands::{GUICommands, GUIEvents},
};

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
}

#[derive(Clone, Debug)]
pub struct NodeGUI {
    pub id: NodeId,
    pub neighbor: Vec<NodeId>,
    pub pdr: f32,
    x: f32,
    y: f32,
    pub node_type: NodeType,
    color: egui::Color32,

    pub command: Option<GUICommands>,

    selected: bool,
    crashed: bool,
    remove_sender: bool,
    add_sender: bool,
    set_pdr: bool,
    pdr_value: Option<String>,

    send_message: bool,
    send_message_value: Option<String>,
    register_to: bool,
    register_value: Option<NodeId>,
    logout: bool,
}

impl NodeGUI {
    pub fn new_drone(drone: ConfigDrone, x: f32, y: f32) -> Self {
        Self {
            id: drone.id,
            neighbor: drone.connected_node_ids.clone(),
            pdr: drone.pdr,
            x,
            y,
            node_type: NodeType::Drone,
            color: Color32::BLUE,

            command: None,

            selected: false,
            crashed: false,
            remove_sender: false,
            add_sender: false,
            set_pdr: false,
            pdr_value: None,

            send_message: false,
            send_message_value: None,
            register_to: false,
            register_value: None,
            logout: false,
        }
    }

    pub fn new_client(client: ConfigClient, x: f32, y: f32) -> Self {
        Self {
            id: client.id,
            neighbor: client.connected_drone_ids.clone(),
            pdr: 0.0,
            x,
            y,
            node_type: NodeType::Client,
            color: Color32::YELLOW,

            command: None,

            selected: false,
            crashed: false,
            remove_sender: false,
            add_sender: false,
            set_pdr: false,
            pdr_value: None,

            send_message: false,
            send_message_value: None,
            register_to: false,
            register_value: None,
            logout: false,
        }
    }
}

impl SimCtrlGUI {
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
        }
    }

    fn handle_events(&mut self, event: GUIEvents) {
        match event {
            // light up edge for 0.5 sec in green
            GUIEvents::PacketSent(src, dest, _) => {
                if self.edges.get(&src).unwrap().0.contains(&dest) {
                    self.edges.get_mut(&src).unwrap().1 = Color32::GREEN;
                    thread::sleep(std::time::Duration::from_secs_f32(0.5));
                    self.edges.get_mut(&src).unwrap().1 = Color32::GRAY;
                } else if self.edges.get(&dest).unwrap().0.contains(&src) {
                    self.edges.get_mut(&dest).unwrap().1 = Color32::GREEN;
                    thread::sleep(std::time::Duration::from_secs_f32(0.5));
                    self.edges.get_mut(&dest).unwrap().1 = Color32::GRAY;
                }
            }
            // light up node  for 0.5 sec in red
            GUIEvents::PacketDropped(src, _) => {
                self.nodes.get_mut(&src).unwrap().color = Color32::RED;
                thread::sleep(std::time::Duration::from_secs_f32(0.5));
                self.nodes.get_mut(&src).unwrap().color = Color32::BLUE;
            }
            GUIEvents::Topology(drones, clients) => actions::topology(self, drones, clients),

            // show message
            GUIEvents::MessageReceived(src, msg) => (),
            // support for pop-up
            GUIEvents::CommunicationServerList(items) => (),
            // support for pop-up
            GUIEvents::ClientList(items) => (),
        }
    }
}

impl eframe::App for SimCtrlGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Not initialized from Network Initializer message
        if !self.initialized {
            warn!("[ {} ] Waiting for initialization", "GUI".green());
            // Wait for Topology message
            match self.receiver.try_recv() {
                Ok(event) => match event.clone() {
                    GUIEvents::Topology(_, _) => self.handle_events(event),
                    _ => error!(
                        "[ {} ] Received NON-Topology GUIEvent before Initialization",
                        "GUI".red()
                    ),
                },
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => eprintln!(
                        "[ {} ]: GUICommands receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                },
            }
        } else {
            // Check for messages
            match self.receiver.try_recv() {
                Ok(event) => self.handle_events(event),
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
                                    let label = format!("{}", neighbor);
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
                if self.spawn_button {
                    if ui.button("Spawn Drone").clicked() {
                        self.spawn_toggle = true;
                        self.spawn_button = false;
                        info!("[ {} ] Spawn button pressed", "GUI".green());
                    }
                }

                // Allocating space for drawing and preparing the painter for rendering
                let (_response, painter) =
                    ui.allocate_painter(egui::Vec2::new(900.0, 900.0), egui::Sense::hover());

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
                for (_, pos) in self.nodes.iter_mut() {
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

                let mut client_list = Vec::<NodeId>::new();
                for (id, instance) in self.nodes.iter() {
                    if instance.node_type == NodeType::Client {
                        client_list.push(*id);
                    }
                }

                // Displaying a pop-up with drone's information
                for (_, instance) in self.nodes.iter_mut() {
                    if instance.selected {
                        egui::Window::new(format!("{:?} {}",instance.node_type, instance.id))
                            .fixed_size([100.0, 100.0]) // Window size
                            .resizable(false) // disable resizable
                            .collapsible(true) // activate collapsable
                            .show(ctx, |ui| {

                                // Display information
                                ui.label(format!("Id: {}", instance.id));
                                if !instance.crashed {
                                    ui.label(format!(
                                        "Neighbors: {:?}",
                                        instance.neighbor
                                    ));
                                    ui.label(format!("PDR: {}", instance.pdr));
                                }
                                ui.add_space(10.0);

                                // Action buttons
                                if !instance.crashed {
                                    ui.horizontal_centered(|ui| {
                                        if ui.button("RemoveSender").clicked() {
                                            instance.remove_sender = !instance.remove_sender;
                                            instance.add_sender = false;
                                            instance.set_pdr = false;
                                        }
                                        if ui.button("AddSender").clicked() {
                                            instance.add_sender = !instance.add_sender;
                                            instance.remove_sender = false;
                                            instance.set_pdr = false;
                                        }
                                        if instance.node_type == NodeType::Drone {
                                            if ui.button("Crash").clicked() {
                                                instance.command = Some(GUICommands::Crash(instance.id));
                                            }
                                            if ui.button("SetPacketDropRate").clicked() {
                                                instance.set_pdr = !instance.set_pdr;
                                                instance.remove_sender = false;
                                                instance.add_sender = false;
                                            }
                                        }
                                        if instance.node_type == NodeType::Client {
                                            if ui.button("SendMessage").clicked() {
                                                instance.send_message = !instance.send_message;
                                                instance.add_sender = false;
                                                instance.remove_sender = false;
                                                instance.register_to = false;
                                            }
                                            if ui.button("RegisterTo").clicked() {
                                                instance.register_to = !instance.register_to;
                                                instance.add_sender = false;
                                                instance.remove_sender = false;
                                                instance.send_message = false;
                                            }
                                            if ui.button("LogOut").clicked() {
                                                if let Some(server) = instance.register_value {
                                                    instance.command = Some(GUICommands::LogOut(instance.id, server));
                                                }
                                            }
                                        }
                                    });
                                }

                                // if not crashed
                                if !instance.crashed && !instance.logout{
                                    // if pressed RemoveSender button
                                    if instance.remove_sender {
                                        let mut _value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Sender to remove: ")
                                            .selected_text(_value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                // Get options
                                                let mut options = Vec::<String>::new();
                                                for numbers in instance.neighbor.clone() {
                                                    options.push(numbers.to_string());
                                                }

                                                // Check if option is selected
                                                for option in options {
                                                    // If something selected
                                                    if ui.selectable_label(
                                                        false,
                                                        &option,
                                                    ).clicked() {
                                                        // Get selected option
                                                        _value = Some(option.to_string());
                                                        instance.remove_sender = false;

                                                        // Parse and handle
                                                        match _value.unwrap().parse::<u8>() {
                                                            Ok(digit) => {
                                                                info!(
                                                                    "[ {} ] Passing to handler GUICommands::RemoveSender({}, {})",
                                                                    "GUI".green(),
                                                                    instance.id,
                                                                    digit
                                                                );
                                                                instance.command = Some(GUICommands::RemoveSender(instance.id, digit))
                                                            },
                                                            Err(e) => error!(
                                                                "[ {} ] Unable to parse neighbor NodeId in Crash GUICommand: {}",
                                                                "GUI".red(),
                                                                e
                                                            ),
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    // if pressed AddSender button
                                    if instance.add_sender {
                                        let mut _value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Sender to add: ")
                                            .selected_text(_value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                // Get options
                                                let mut options = Vec::<String>::new();
                                                for (numbers, _) in self.edges.iter() {
                                                    if !instance.neighbor.contains(numbers) && *numbers != instance.id {
                                                        options.push(numbers.to_string());
                                                    }
                                                }

                                                // If something selected
                                                for option in options {
                                                    // If something selected
                                                    if ui.selectable_label(
                                                        false,
                                                        &option,
                                                    ).clicked() {
                                                        // Get selected option
                                                        _value = Some(option.to_string());
                                                        instance.remove_sender = false;

                                                        // Parse and handle
                                                        match _value.unwrap().parse::<u8>() {
                                                            Ok(digit) => {
                                                                info!("[ {} ] Passing to handler GUICommands::AddSender({}, {})", "GUI".green(), instance.id, digit);
                                                                instance.command = Some(GUICommands::AddSender(instance.id, digit))
                                                            },
                                                            Err(e) => {
                                                                error!(
                                                                    "[ {} ] Unable to parse neighbor NodeId in GUICommand::AddSender: {}",
                                                                    "GUI".red(),
                                                                    e
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    if instance.set_pdr{
                                        ui.horizontal(|ui| {
                                            ui.label("Enter desired PDR:");

                                            // Default value
                                            let text_input = instance.pdr_value.clone().unwrap_or_default();
                                            // Copy for mutation
                                            let mut buffer = text_input.clone();

                                            let text_edit = ui.text_edit_singleline(&mut buffer);

                                            // Update only if changed
                                            if text_edit.changed() {
                                                instance.pdr_value = Some(buffer);
                                            }

                                            // "Confirm" button to process input
                                            if ui.button("Confirm").clicked() {
                                                if let Some(pdr_value) = &instance.pdr_value {
                                                    match pdr_value.parse::<f32>() {
                                                        Ok(digit) => {
                                                            info!(
                                                                "[ {} ] Passing to handler GUICommands::SetPDR({}, {})",
                                                                "GUI".green(),
                                                                instance.id,
                                                                digit
                                                            );
                                                            instance.command = Some(GUICommands::SetPDR(instance.id, digit));
                                                            instance.set_pdr = false;
                                                        }
                                                        Err(e) => error!("[ {} ] Invalid PDR input: {}", "GUI".red(), e),
                                                    }
                                                }
                                            }
                                        });
                                    }

                                    if instance.send_message {
                                        let mut value: Option<String> = None;
                                        ui.vertical(|ui| {
                                            // Title
                                            ui.heading("Send a Message");

                                            // Multi-Select Neighbor Dropdown
                                            ui.label("Select a Client:");
                                            egui::ComboBox::from_label("Clients:")
                                                .selected_text(value.clone().unwrap_or("None".to_string()))
                                                .show_ui(ui, |ui| {
                                                    for client in client_list.iter() {
                                                        let label = client.to_string();
                                                        let is_selected = value.as_ref().map_or(false, |v| v == &label);

                                                        if ui.selectable_label(is_selected, label.clone()).clicked() {
                                                            value = Some(label); // Update selection properly
                                                        }
                                                    }
                                                });

                                            // Message Input
                                            ui.horizontal(|ui| {
                                                ui.label("Enter Message:");
                                                let text_message = instance.send_message_value.clone().unwrap_or_default();
                                                let mut buffer_message = text_message.clone();

                                                let text_edit = ui.text_edit_singleline(&mut buffer_message);
                                                if text_edit.changed() {
                                                    instance.send_message_value = Some(buffer_message);
                                                }
                                            });

                                            // "Send" Button
                                            if ui.button("Send").clicked() {
                                                if let (Some(client), Some(message)) = (value.clone(), instance.send_message_value.clone()) {
                                                    if let Ok(client_id) = client.parse::<u8>() {
                                                        info!("[ {} ] Sending message to {}: {}", "GUI".green(), client_id, message);
                                                        instance.command = Some(GUICommands::SendMessageTo(instance.id, client_id, message));
                                                    } else {
                                                        error!("[ {} ] Invalid client ID format", "GUI".red());
                                                    }
                                                } else {
                                                    error!("[ {} ] Missing client or message", "GUI".red());
                                                }
                                            }
                                        });
                                    }

                                    if instance.register_to {
                                        let mut value: Option<String> = None;

                                        ui.vertical(|ui| {
                                            // Title
                                            ui.heading("Send a Message");

                                            // Multi-Select Neighbor Dropdown
                                            ui.label("Select a Client:");
                                            egui::ComboBox::from_label("Clients:")
                                                .selected_text(value.clone().unwrap_or("None".to_string()))
                                                .show_ui(ui, |ui| {
                                                    let mut options = Vec::<String>::new();
                                                    for client in client_list.iter() {
                                                        options.push(client.to_string());
                                                    }

                                                    for option in options {
                                                        if ui.selectable_label(false, &option).clicked() {
                                                            value = Some(option.to_string());
                                                            instance.remove_sender = false;

                                                            // Parse and handle selection
                                                            match value.clone().unwrap().parse::<u8>() {
                                                                Ok(digit) => {
                                                                    info!("[ {} ] Selected Client: {}", "GUI".green(), digit);
                                                                }
                                                                Err(e) => {
                                                                    error!("[ {} ] Unable to parse client ID: {}", "GUI".red(), e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                });

                                            // Message Input
                                            ui.horizontal(|ui| {
                                                ui.label("Enter Message:");
                                                let text_message = instance.send_message_value.clone().unwrap_or_default();
                                                let mut buffer_message = text_message.clone();

                                                let text_edit = ui.text_edit_singleline(&mut buffer_message);
                                                if text_edit.changed() {
                                                    instance.send_message_value = Some(buffer_message);
                                                }
                                            });

                                            // "Send" Button
                                            if ui.button("Send").clicked() {
                                                if let (Some(client), Some(message)) = (value.clone(), instance.send_message_value.clone()) {
                                                    if let Ok(client_id) = client.parse::<u8>() {
                                                        info!("[ {} ] Sending message to {}: {}", "GUI".green(), client_id, message);
                                                        instance.command = Some(GUICommands::SendMessageTo(instance.id, client_id, message));
                                                    } else {
                                                        error!("[ {} ] Invalid client ID format", "GUI".red());
                                                    }
                                                } else {
                                                    error!("[ {} ] Missing client or message", "GUI".red());
                                                }
                                            }
                                        });
                                    }
                                }

                                ui.add_space(10.0);

                                // Button to close the window
                                if ui.button("Close").clicked() {
                                    instance.selected = false;
                                }
                            });
                    }
                }
            });
        }

        // Handle Commands
        for (_, instance) in self.nodes.clone() {
            if let Some(command) = &instance.command {
                info!("[ {} ] Handling {:?}", "GUI".green(), command);
                match command {
                    GUICommands::Crash(drone) => actions::crash(self, drone),
                    GUICommands::RemoveSender(drone, to_remove) => {
                        actions::remove_sender(self, drone, to_remove)
                    }
                    GUICommands::AddSender(drone, to_add) => {
                        actions::add_sender(self, drone, *to_add)
                    }
                    GUICommands::SetPDR(drone, pdr) => actions::set_pdr(self, drone, pdr),

                    GUICommands::SendMessageTo(src, dest, msg) => {
                        actions::send_message(self, src, dest, msg.clone());
                    }
                    GUICommands::RegisterTo(client, server) => {
                        actions::register(self, client, server);
                    }
                    GUICommands::LogOut(client, server) => {
                        actions::logout(self, client, server);
                    }
                    _ => error!("[ {} ] Not supposed to handle {:?}", "GUI".red(), command),
                }
            }
        }

        if let Some(command) = &self.spawn_command.clone() {
            info!("[ {} ] Handling {:?}", "GUI".green(), command);
            match command {
                GUICommands::Spawn(id, neighbor, pdr) => {
                    actions::spawn(self, id, &neighbor.clone(), *pdr)
                }
                _ => error!("[ {} ] Not supposed to handle {:?}", "GUI".red(), command),
            }
        }
    }
}
