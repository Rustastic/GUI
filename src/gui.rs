use std::{collections::HashMap, f32::consts::PI};
use crossbeam_channel::{Receiver, Sender, TryRecvError};

use colored::Colorize;
use eframe::egui::{self, Color32};

use wg_2024::{config::Drone as ConfigDrone, network::NodeId};

use crate::{GUICommands, GUIEvents};

pub struct SimCtrlGUI {
    sender: Sender<GUICommands>,
    receiver: Receiver<GUIEvents>,

    initialized: bool,
    nodes: HashMap<NodeId, DroneGUI>,
    edges: HashMap<NodeId, Vec<NodeId>>
}

struct DroneGUI {
    id: NodeId,
    neighbor: Vec<NodeId>,
    pdr: f32,
    x: f32,
    y: f32,
    color: egui::Color32,

    selected: bool,
    crashed: bool,
    remove_sender: bool,
    remove_sender_value: Option<String>
}

impl SimCtrlGUI {
    pub fn new(sender: Sender<GUICommands>, receiver: Receiver<GUIEvents>) -> Self {
        Self {
            sender,
            receiver,
            initialized: false,
            nodes: HashMap::new(),
            edges: HashMap::new()
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

                selected: false,
                crashed: false,
                remove_sender: false,
                remove_sender_value: None,
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

    fn handle_commands(&mut self, drone: NodeId, command: GUICommands) {
        match command {
            GUICommands::Spawn => (),
            GUICommands::Crash(node_id) => (),
            GUICommands::RemoveSender(drone, neighborz) => (),
            GUICommands::AddSender(drone, neighbor) => (),
            GUICommands::SetPDR(drone, pdr) => (),
        }
    }

    fn handle_events(&mut self, event: GUIEvents) {
        match event {
            GUIEvents::PacketSent(src, dest, packet) => (),
            GUIEvents::PacketDropped(src, packet) => (),
            GUIEvents::Topology(topology) => self.topology(topology)
        }
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
                                            match self.sender.send(GUICommands::Crash(instance.id)) {
                                                Ok(()) => {
                                                    // change color to red
                                                    instance.color = egui::Color32::RED;

                                                    // remove from edge hashmap
                                                    self.edges.remove(&instance.id);

                                                    // remove edges starting from neighbor
                                                    for neighbor_id in instance.neighbor.iter() {
                                                        // get edges starting from neighbor
                                                        if let Some(neighbor_drone) = self.edges.get_mut(neighbor_id) {
                                                            for (index, drone) in neighbor_drone.clone().iter_mut().enumerate() {
                                                                // if they end in the crashing drone
                                                                if *drone == instance.id {
                                                                    neighbor_drone.remove(index);
                                                                }
                                                            }
                                                        }
                                                    }

                                                    instance.neighbor.clear();
                                                    instance.pdr = 0.0;

                                                    instance.crashed = true;
                                                },
                                                Err(e) => panic!("Voglio la mamma: {}", e),
                                            }
                                        }
                                        if ui.button("RemoveSender").clicked() {
                                            instance.remove_sender = !instance.remove_sender;
                                            /*match self.sender.send(GUICommands::RemoveSender(instance, ())) {
                                                
                                            }*/

                                            if instance.remove_sender {
                                                egui::ComboBox::from_label("Select an option")
                                                    .selected_text(instance.remove_sender_value.clone().unwrap_or("None".to_string()))
                                                    .show_ui(ui, |ui| {
                                                        for option in &["Option 1", "Option 2", "Option 3"] {
                                                            if ui.selectable_label(
                                                                instance.remove_sender_value.as_deref() == Some(*option),
                                                                *option,
                                                            )
                                                            .clicked()
                                                            {
                                                                instance.remove_sender_value = Some(option.to_string());
                                                                instance.remove_sender = false;
                                                            }
                                                        }
                                                    });
                                            }
                                        }
                                        if ui.button("AddSender").clicked() {

                                        }
                                        if ui.button("Set PacketDropRate").clicked() {

                                        }
                                        
                                        /*if ui.button("Red").clicked() {
                                            instance.color = egui::Color32::RED;
                                        }
                                        if ui.button("Green").clicked() {
                                            instance.color = egui::Color32::GREEN;
                                        }
                                        if ui.button("Blue").clicked() {
                                            instance.color = egui::Color32::BLUE;
                                        }*/
                                    });
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
    }
}
