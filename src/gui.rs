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
    drones: Vec<DroneGUI>,
    edges: HashMap<NodeId, NodeId>
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
            drones: Vec::new(),
            edges: HashMap::new()
        }
    }

    fn topology(&mut self, topology: Vec<ConfigDrone>) {
        let mut i = 1;

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

            for drone in new_drone.neighbor.iter() {
                if !self.edges.contains_key(drone) {
                    self.edges.insert(new_drone.id.clone(), *drone);
                }
            }

            self.drones.push(new_drone);

            i += 1;
        }
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
                ui.heading("Drone Simulation GUI");

                if ui.button("Spawn Drone").clicked() {
                    //self.command_send.send(DroneCommand::Spawn(1)).unwrap();
                }
                if ui.button("Crash Drone").clicked() {
                    //self.command_send.send(DroneCommand::Crash(1)).unwrap();
                }

                ui.separator();
                ui.label("Simulation Events:");
                /*match self.event_recv.try_recv() {
                    Ok(event) => ui.label(format!("Event: {:?}", event)),
                    Err(TryRecvError::Empty) => ui.label("No new events."),
                    Err(TryRecvError::Disconnected) => ui.label("Error: Event channel disconnected."),
                }*/
            });

            ctx.request_repaint(); // Refresh GUI
        }
    }
}
