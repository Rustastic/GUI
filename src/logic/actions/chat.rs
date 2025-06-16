use colored::Colorize;
use log::{error, info};

use wg_2024::network::NodeId;

use messages::gui_commands::GUICommands;

use crate::logic::state::GUIState;

pub fn send_message(state: &mut GUIState, src: NodeId, dest: NodeId, msg: &str) {
    match state.sender.send(GUICommands::SendMessageTo(src, dest, msg.to_string())) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::SendMessageTo({}, {}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            src,
            dest,
            msg
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::SendMessageTo({}, {}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                src,
                dest,
                msg,
                e
            );
        },
    }
}

pub fn register(state: &mut GUIState, client: NodeId, server: NodeId) {
    match state.sender.send(GUICommands::RegisterTo(client, server)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::RegisterTo({}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::RegisterTo({}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                client,
                server,
                e
            );
        },
    }
}

pub fn get_list(state: &mut GUIState, client: NodeId) {
    match state.sender.send(GUICommands::GetClientList(client)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::GetClientList({}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::GetClientList({}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                client,
                e
            );
        },
    }
}

pub fn logout(state: &mut GUIState, client: NodeId, server: NodeId) {
    match state.sender.send(GUICommands::LogOut(client, server)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::LogOut({}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::LogOut({}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                client,
                server,
                e
            );
        }
    }
}
