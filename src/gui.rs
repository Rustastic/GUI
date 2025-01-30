use crossbeam_channel::{Receiver, Sender, TryRecvError};

use eframe::egui;
use simulation_controller::SimulationController;
use wg_2024::network::NodeId;

use crate::{GUICommands, GUIEvents};

pub struct SimCtrlGUI {
    sim_ctrl: SimulationController,
    sender: Sender<GUICommands>,
    receiver: Receiver<GUIEvents>
}

impl SimCtrlGUI {
    pub fn new(sim_ctrl: SimulationController, sender: Sender<GUICommands>, receiver: Receiver<GUIEvents>) -> Self {
        Self {
            sim_ctrl,
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
    }

    fn handle_events(&mut self, drone: NodeId, event: GUIEvents) {
        match event {
            GUIEvents::PacketSent(src, dest) => todo!(),
            GUIEvents::PacketDropped(src, packet) => todo!(),
        }
    }*/
}

impl eframe::App for SimCtrlGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
