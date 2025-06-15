use colored::Colorize;
use log::{error, info};

use wg_2024::network::NodeId;

use messages::gui_commands::GUICommands;

use crate::logic::state::GUIState;

pub fn ask_for_file_list(state: &mut GUIState, client: NodeId, server: NodeId) {
    match state.sender.send(GUICommands::AskForFileList(client, server)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::AskForFileList({}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::AskForFileList({}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                client,
                server,
                e
            );
        },
    }
    state.nodes.get_mut(&client).unwrap().command = None;
}

pub fn get_file(state: &mut GUIState, client: NodeId, server: NodeId, title: &str) {
    match state.sender.send(GUICommands::GetFile(client, server, title.to_string())) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::GetFile({}, {}, {:?}) from GUI to Simulation Controller",
            "GUI".green(),
            client,
            server,
            title
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::GetFile({}, {}, {:?}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                client,
                server,
                title,
                e
            );
        },
    }
    state.nodes.get_mut(&client).unwrap().command = None;
}
