use std::{collections::HashMap, f32::consts::PI};
use crossbeam_channel::{Receiver, Sender};

use colored::Colorize;
use eframe::egui::{self, Color32};

use wg_2024::{config::Drone as ConfigDrone, network::NodeId};

use crate::{GUICommands, GUIEvents};

#[derive(Clone, Debug)]
pub struct SimCtrlGUI {
    sender: Sender<GUICommands>,
    receiver: Receiver<GUIEvents>,

    initialized: bool,
    nodes: HashMap<NodeId, DroneGUI>,
    edges: HashMap<NodeId, Vec<NodeId>>,

    spawn_button: bool,
    spawn_toggle: bool,
    spawn_id: Option<String>,
    spawn_neighbors: Vec<NodeId>,
    spawn_pdr: Option<String>,
    spawn_command: Option<GUICommands>
}

#[derive(Clone, Debug)]
struct DroneGUI {
    id: NodeId,
    neighbor: Vec<NodeId>,
    pdr: f32,
    x: f32,
    y: f32,
    color: egui::Color32,

    command: Option<GUICommands>,

    selected: bool,
    crashed: bool,
    remove_sender: bool,
    add_sender: bool,
    set_pdr: bool,
    pdr_value: Option<String>
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
            spawn_command: None
        }
    }

    fn topology(&mut self, topology: Vec<ConfigDrone>) {
        let radius = 200.0;
        let center_x = 400.0;
        let center_y = 400.0;

        for (i, drone) in topology.iter().enumerate() {
            let angle = i as f32 * (2.0 * PI / 10.0);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();

            let new_drone = DroneGUI {
                id: drone.id,
                neighbor: drone.connected_node_ids.clone(),
                pdr: drone.pdr,
                x,
                y,
                color: Color32::BLUE,

                command: None,

                selected: false,
                crashed: false,
                remove_sender: false,
                add_sender: false,
                set_pdr: false,
                pdr_value: None,
            };

            for drone in new_drone.neighbor.clone() {
                if !self.edges.contains_key(&drone) {
                    let vec = self.edges.entry(new_drone.id).or_insert_with(Vec::new);
                    vec.push(drone);
                }
            }

            self.nodes.insert(new_drone.id, new_drone);
        }

        self.initialized = true;
    }

    fn handle_events(&mut self, event: GUIEvents) {
        match event {
            GUIEvents::PacketSent(src, dest, packet) => (),
            GUIEvents::PacketDropped(src, packet) => (),
            GUIEvents::Topology(topology) => self.topology(topology)
        }
    }

    fn crash(&mut self, drone: &NodeId) {
        let instance = self.nodes.get_mut(drone).unwrap();
        match self.sender.send(GUICommands::Crash(instance.id)) {
            Ok(()) => {
                // remove from edge hashmap
                self.edges.remove(&instance.id);

                // remove edges starting from neighbor
                for neighbor_id in instance.neighbor.iter() {
                    // get edges starting from neighbor
                    if let Some(neighbor_drone) = self.edges.get_mut(neighbor_id) {
                        neighbor_drone.retain(|drone| *drone != instance.id);
                    }
                }
                let id = instance.id;
                self.nodes.remove(&id);
            },
            Err(e) => panic!("Voglio la mamma: {}", e),
        }

        let neighbors = self.nodes.get(drone).unwrap().neighbor.clone();
        let id = self.nodes.get(drone).unwrap().id.clone();
        for node in neighbors {
            let a =self.nodes.get_mut(&node).unwrap();
            a.neighbor.retain(|&x| x != id);
        }

        let instance = self.nodes.get_mut(drone).unwrap();
        instance.neighbor.clear();
        instance.pdr = 1.0;
        instance.crashed = true;
        instance.command = None;
    }

    fn remove_sender(&mut self, drone: &NodeId, to_remove: NodeId) {
        let instance = self.nodes.get_mut(drone).unwrap();
        match self.sender.send(GUICommands::RemoveSender(instance.id, to_remove)) {
            Ok(_) => {
                // get edges of instance
                if let Some(edge) = self.edges.get_mut(&instance.id) {
                    if edge.contains(&to_remove) {
                        edge.retain(|&node| node != to_remove);
                    }
                } 
                if let Some(edge) = self.edges.get_mut(&to_remove) {
                    if edge.contains(&instance.id) {
                        edge.retain(|&node| node != instance.id);
                    }
                }
                
            },
            Err(e) => panic!("IO ODIO IL GOVERNO"),
        }

        // Remove neighbor from the current instance.
        instance.neighbor.retain(|&x| x != to_remove);

        // Remove neighbor from to_remove
        let id = self.nodes.get(drone).unwrap().id.clone();
        let neighbor = self.nodes.get_mut(&to_remove).unwrap();
        neighbor.neighbor.retain(|&x| x != id);
        
    }

    fn add_sender(&mut self, drone: &NodeId, to_add: NodeId) {
        let instance = self.nodes.get_mut(drone).unwrap();
        match self.sender.send(GUICommands::AddSender(instance.id, to_add)) {
            Ok(_) => {
                instance.neighbor.push(to_add);
                self.edges.entry(*drone).or_insert_with(Vec::new).push(to_add);

                let neighbor = self.nodes.get_mut(&to_add).unwrap();
                neighbor.neighbor.push(*drone);
            },
            Err(e) => panic!("IO ODIO IL GOVERNO"),
        }

        self.nodes.get_mut(drone).unwrap().command = None;
    }

    fn set_pdr(&mut self, drone: &NodeId, pdr: &f32) {
        let instance = self.nodes.get_mut(drone).unwrap();
        match self.sender.send(GUICommands::SetPDR(instance.id, *pdr)) {
            Ok(_) => {
                instance.pdr = *pdr;                
            },
            Err(e) => panic!("IO ODIO IL GOVERNO"),
        }

        instance.command = None;
    }

    fn spawn(&mut self, id: &NodeId, neighbors: &Vec<NodeId>, pdr: f32) {
        match self.sender.send(GUICommands::Spawn(*id, neighbors.clone(), pdr)) {
            Ok(()) => {
                self.spawn_id = None;
                self.spawn_neighbors.clear();
                self.spawn_pdr = None;

                // add to nodes
                let new_drone = DroneGUI {
                    id: *id,
                    neighbor: neighbors.clone(),
                    pdr,
                    x: 400.0,
                    y: 400.0,
                    color: Color32::BLUE,
                    
                    command: None,

                    selected: false,
                    crashed: false,
                    remove_sender: false,
                    add_sender: false,
                    set_pdr: false,
                    pdr_value: None,
                };
                self.nodes.insert(*id, new_drone);

                // add edges
                self.edges.insert(*id, neighbors.clone());

                // ad to various instances neighbors
                for drone in neighbors {
                    self.nodes.get_mut(drone).unwrap().neighbor.push(*drone);
                }

                self.spawn_command = None;
            },
            Err(_) => panic!(""),
        };
    }
}

impl eframe::App for SimCtrlGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.initialized {
            match self.receiver.try_recv() {
                Ok(event) => match event.clone() {
                    GUIEvents::Topology(_) => self.handle_events(event),
                    _ => panic!("CAZZZOOOOOOOOOOOOOOOOOOOOOOOOOOOO"),
                },
                Err(e)=> match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => eprintln!(
                        "[ {} ]: GUICommands receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                }
            }
        } else {
            match self.receiver.try_recv() {
                Ok(event) => self.handle_events(event),
                Err(e)=> match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => {
                        eprintln!(
                            "[ {} ]: GUICommands receiver channel disconnected: {}",
                            "Simulation Controller".red(),
                            e
                        );
                        return;
                    }
                }
                
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Simulation Controller");

                if self.spawn_toggle {
                    ui.vertical(|ui| {
                        ui.heading("Spawn a New Drone");
                
                        // ID Input
                        ui.horizontal(|ui| {
                            ui.label("Enter Drone ID:");
                            let text_id = self.spawn_id.clone().unwrap_or_default().to_string();
                            let mut buffer_id = text_id.clone(); // Buffer for mutation
                
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
                                    } else {
                                        eprintln!("Invalid PDR value");
                                    }
                                } else {
                                    eprintln!("Invalid ID value");
                                }
                            }
                        }
                    });
                }
                
                if self.spawn_button { 
                    // Button to Open the Spawn Form
                    if ui.button("Spawn Drone").clicked() {
                        self.spawn_toggle = true;
                        self.spawn_button = false;
                    }
                }
    
                // Allocating space for drawing and preparing the painter for rendering
                let (_response, painter) =
                    ui.allocate_painter(egui::Vec2::new(900.0, 900.0), egui::Sense::hover());
    
                // Drawing edges (connections) between drones
                for (start_id, neighbor) in self.edges.clone() {
                    let start = self.nodes.get(&start_id).unwrap();
                    for end_id in neighbor {
                        let end = self.nodes.get(&end_id).unwrap();
                        painter.line_segment(
                            [egui::pos2(start.x, start.y), egui::pos2(end.x, end.y)],
                            egui::Stroke::new(2.0, Color32::LIGHT_GRAY),
                        );
                    }
                }
    
                // Drawing the nodes (drones) and handling user interaction for selection
                for (_, pos) in self.nodes.iter_mut() {
                    let screen_pos = egui::pos2(pos.x, pos.y);
                    let radius = 10.0;
    
                    // Allocating space for each drone's graphical representation
                    let response = ui.allocate_rect(
                        egui::Rect::from_center_size(screen_pos, egui::Vec2::splat(radius * 2.0)),
                        egui::Sense::click(),
                    );
    
                    // Detecting if the drone is clicked and updating its selected status
                    if response.clicked() {
                        pos.selected = true;
                    }
    
                    // Drawing the drone as a filled circle
                    painter.circle_filled(screen_pos, radius, pos.color);
                }

                // Displaying a pop-up with detailed information when a drone is selected
                for (_, instance) in self.nodes.iter_mut() {
                    if instance.selected {
                        egui::Window::new(format!("Node {}", instance.id))
                            .fixed_size([100.0, 100.0]) // Window size
                            .resizable(false) // disable resizable
                            .collapsible(true) // activate collapsable
                            .show(ctx, |ui| {
                                // Displaying information about the selected drone.
                                ui.label(format!("Id: {}", instance.id));
                                if !instance.crashed {
                                    ui.label(format!(
                                        "Neighbors: {:?}",
                                        instance.neighbor
                                    ));
                                    ui.label(format!("PDR: {}", instance.pdr));
                                }
                                ui.add_space(10.0);
    
                                // Buttons to change the color of the selected drone
                                if !instance.crashed {
                                    ui.horizontal_centered(|ui| {
                                        if ui.button("Crash").clicked() {
                                            instance.command = Some(GUICommands::Crash(instance.id));
                                        }
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
                                        if ui.button("Set PacketDropRate").clicked() {
                                            instance.set_pdr =! instance.set_pdr;
                                            instance.remove_sender = false;
                                            instance.add_sender = false;

                                        }
                                    });
                                }

                                if !instance.crashed {
                                    if instance.remove_sender {
                                        let mut value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Sender to remove: ")
                                            .selected_text(value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                let mut options = Vec::<String>::new();
                                                for numbers in instance.neighbor.clone() {
                                                    options.push(numbers.to_string());
                                                }

                                                for option in options {
                                                    if ui.selectable_label(
                                                        false,
                                                        &option,
                                                    ).clicked() {
                                                        value = Some(option.to_string());
                                                        instance.remove_sender = false;

                                                        match value.unwrap().parse::<u8>() {
                                                            Ok(digit) => instance.command = Some(GUICommands::RemoveSender(instance.id, digit)),
                                                            Err(_) => panic!(""),
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    if instance.add_sender {
                                        let mut value: Option<String> = None;
                                        egui::ComboBox::from_label("Select Sender to add: ")
                                            .selected_text(value.clone().unwrap_or("None".to_string()))
                                            .show_ui(ui, |ui| {
                                                let mut options = Vec::<String>::new();
                                                for (numbers, _) in self.edges.iter() {
                                                    if !instance.neighbor.contains(numbers) && *numbers != instance.id {
                                                        options.push(numbers.to_string());
                                                    }
                                                }

                                                for option in options {
                                                    if ui.selectable_label(
                                                        false,
                                                        &option,
                                                    ).clicked() {
                                                        value = Some(option.to_string());
                                                        instance.remove_sender = false;

                                                        match value.unwrap().parse::<u8>() {
                                                            Ok(digit) => instance.command = Some(GUICommands::AddSender(instance.id, digit)),
                                                            Err(_) => panic!(""),
                                                        }
                                                    }
                                                }
                                            });
                                    }

                                    if instance.set_pdr{
                                        ui.horizontal(|ui| {
                                            ui.label("Enter desired PDR:");
                                        
                                            // Ensure there's a default value for the input field
                                            let text_input = instance.pdr_value.clone().unwrap_or_default();
                                            let mut buffer = text_input.clone(); // Copy for mutation
                                        
                                            let text_edit = ui.text_edit_singleline(&mut buffer);
                                        
                                            // Update instance.remove_sender_value only if the text changed
                                            if text_edit.changed() {
                                                instance.pdr_value = Some(buffer);
                                            }
                                        
                                            // "Confirm" button to process input
                                            if ui.button("Confirm").clicked() {
                                                if let Some(pdr_value) = &instance.pdr_value {
                                                    match pdr_value.parse::<f32>() {
                                                        Ok(digit) => {
                                                            instance.command = Some(GUICommands::SetPDR(instance.id, digit));
                                                            instance.set_pdr = false; // Close input mode
                                                        }
                                                        Err(_) => eprintln!("Invalid PDR input"),
                                                    }
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

        for (_, instance) in self.nodes.clone() {
            if let Some(command) = &instance.command {
                match command {
                    GUICommands::Crash(drone) => self.crash(drone),
                    GUICommands::RemoveSender(drone, to_remove) => self.remove_sender(drone, *to_remove),
                    GUICommands::AddSender(drone, to_add) => self.add_sender(drone, *to_add),
                    GUICommands::SetPDR(drone, pdr) => self.set_pdr(drone, pdr),
                    _ => ()
                }       
            }
        }

        if let Some(command) = &self.spawn_command.clone() {
            match command {
                GUICommands::Spawn(id, neighbor, pdr) => self.spawn(id, &neighbor.clone(), *pdr),
                _ => ()
            }
        }
    }
}
