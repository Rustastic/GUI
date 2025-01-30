use crossbeam_channel::{Sender, Receiver};
use eframe::egui;

use simulation_controller::SimulationController;

use crate::GUIActions;

pub struct SimCtrlGUI {
    sim_ctrl: SimulationController,
    receiver: Receiver<GUIActions>
}

impl SimCtrlGUI {
    pub fn new(sim_ctrl: SimulationController, receiver: Receiver<GUIActions>) -> Self {
        Self {
            sim_ctrl,
            receiver
        }
    }
}

impl eframe::App for SimCtrlGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Drone Network Simulation");

            if ui.button("Spawn Drone").clicked() {
                //controller.handle_gui_command(DroneCommand::Spawn(1)); // Example with NodeId = 1
            }
            if ui.button("Crash Drone").clicked() {
                //controller.handle_gui_command(DroneCommand::Crash(1));
            }
            if ui.button("Remove Sender").clicked() {
                //controller.handle_gui_command(DroneCommand::RemoveSender(1));
            }
            if ui.button("Add Sender").clicked() {
                //controller.handle_gui_command(DroneCommand::AddSender(1, Sender::default()));
            }
            if ui.button("Set PDR").clicked() {
                //controller.handle_gui_command(DroneCommand::SetPacketDropRate(0.5));
            }

            ui.separator();
            ui.label("Simulation Events:");
            /*match self.controller.try_recv() {
                Ok(event) => ui.label(format!("Event: {:?}", event)),
                Err(_) => ui.label("No new events."),
            }*/
        });

        ctx.request_repaint(); // Refresh GUI
    }
}
