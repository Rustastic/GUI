use colored::Colorize;
use log::{error, info};

use wg_2024::network::NodeId;

use messages::gui_commands::GUICommands;

use crate::logic::state::GUIState;

pub fn remove_sender(state: &mut GUIState, node_id: NodeId, to_remove: NodeId) {
    match state.sender.send(GUICommands::RemoveSender(node_id, to_remove)) {
        Ok(()) => {
            info!(
                "[ {} ] Successfully sent GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                node_id,
                to_remove
            );
        },
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                node_id,
                to_remove,
                e
            );
        },
    }
    state.nodes.get_mut(&node_id).unwrap().command = None;
}

pub fn add_sender(state: &mut GUIState, node_id: NodeId, to_add: NodeId) {
    match state.sender.send(GUICommands::AddSender(node_id, to_add)) {
        Ok(()) => {
            info!(
                "[ {} ] Successfully sent GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                node_id,
                to_add
            );
        },
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                node_id,
                to_add,
                e
            );
        },
    }
    state.nodes.get_mut(&node_id).unwrap().command = None;
}
