use colored::Colorize;
use log::{error, info};

use wg_2024::network::NodeId;

use messages::gui_commands::GUICommands;

use crate::logic::{nodes::NodeGUI, state::GUIState};

pub fn crash(state: &mut GUIState, drone: NodeId) {
    match state.sender.send(GUICommands::Crash(drone)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::Crash() from GUI to Simulation Controller",
            "GUI".green(),
        ),
        Err(e) => {
            error!(
                "[ {} ] Unable to send GUICommand::Crash() from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            );
        }
    }
}

pub fn set_pdr(state: &mut GUIState, instance: &mut NodeGUI, pdr: f32) {
    match state.sender.send(GUICommands::SetPDR(instance.id, pdr)) {
        Ok(()) => {
            info!("[ {} ] Successfully sent GUICommand::SetPDR({}, {}) from GUI to Simulation Controller", "GUI".green(), instance.id, pdr);
            instance.pdr = pdr;
        }
        Err(e) => {
            error!(
                "[ {} ] Unable to send GUICommand::SetPDR from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            );
        },
    }
}

pub fn spawn(state: &mut GUIState, id: NodeId, neighbors: &Vec<NodeId>, pdr: f32) {
    match state.sender.send(GUICommands::Spawn(id, neighbors.clone(), pdr)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::Spawn({}, {:?}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            id,
            neighbors,
            pdr
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::Spawn({}, {:?}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                id,
                neighbors,
                pdr,
                e
            );
        },
    }
}
