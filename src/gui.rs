use crossbeam_channel::{Receiver, Sender, TryRecvError};

use colored::Colorize;
use eframe::egui;

use wg_2024::network::NodeId;

use crate::{GUICommands, GUIEvents};

pub struct SimCtrlGUI {
    sender: Sender<GUICommands>,
    receiver: Receiver<GUIEvents>
}

impl SimCtrlGUI {
    pub fn new(sender: Sender<GUICommands>, receiver: Receiver<GUIEvents>) -> Self {
        Self {
            sender,
            receiver
        }
    }

    /*fn handle_commands(&mut self, drone: NodeId, command: GUICommands) {
        match command {
            GUICommands::Spawn(drone) => todo!(),
            GUICommands::Crash(node_id) => todo!(),
            GUICommands::RemoveSender(drone, neighbor) => todo!(),
            GUICommands::AddSender(drone, neighbor) => todo!(),
            GUICommands::SetPDR(drone, pdr) => todo!(),
        }
    }*/

    fn handle_events(&mut self, event: GUIEvents) {
        match event {
            GUIEvents::PacketSent(src, dest, packet) => todo!(),
            GUIEvents::PacketDropped(src, packet) => todo!(),
            GUIEvents::Topology(topology) => todo!()
        }
    }
}

impl eframe::App for SimCtrlGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

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
