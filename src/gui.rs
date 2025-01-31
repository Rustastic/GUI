use std::collections::HashMap;
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
    selected: bool,
    color: egui::Color32
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
        let mut i = 0;

        for drone in topology {
            let new_drone = DroneGUI {
                id: drone.id,
                neighbor: drone.connected_node_ids,
                pdr: drone.pdr,
                x: (i * 100) as f32,
                y: 100.0,
                selected: false,
                color: Color32::BLUE,
            };

            for drone in new_drone.neighbor.clone() {
                if !self.edges.contains_key(&drone) {
                    let vec = self.edges.entry(new_drone.id).or_insert_with(Vec::new);
                    vec.push(drone);
                }
            }

            self.nodes.insert(new_drone.id, new_drone);

            i += 1;
        }

        self.initialized = true;
    }

    fn handle_commands(&mut self, drone: NodeId, command: GUICommands) {
        match command {
            GUICommands::Spawn => (),
            GUICommands::Crash(node_id) => (),
            GUICommands::RemoveSender(drone, neighbor) => (),
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
                    crossbeam_channel::TryRecvError::Disconnected => eprintln!(
                        "[ {} ]: GUICommands receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
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
                                ui.label(format!(
                                    "Neighbors: {:?}",
                                    instance.neighbor
                                ));
                                ui.label(format!("PDR: {}", instance.pdr));
                                ui.add_space(10.0);
    
                                // Buttons to change the color of the selected drone
                                ui.horizontal_centered(|ui| {
                                    if ui.button("Red").clicked() {
                                        instance.color = egui::Color32::RED;
                                    }
                                    if ui.button("Green").clicked() {
                                        instance.color = egui::Color32::GREEN;
                                    }
                                    if ui.button("Blue").clicked() {
                                        instance.color = egui::Color32::BLUE;
                                    }
                                });
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
