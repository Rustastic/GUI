use crossbeam_channel::TrySendError;

use colored::Colorize;
use log::error;

use messages::gui_commands::GUICommands;

use crate::logic::{actions::*, state::GUIState};

pub struct CommandHandler;

impl CommandHandler {
    pub fn new() -> Self {
        Self
    }

    /*fn send_command(&self, state: &mut GUIState, command: GUICommands) {
        match state.sender.try_send(command.clone()) {
            Ok(()) => {
                match command {
                    GUICommands::Crash(drone) => {
                        crash(state, drone);
                    }
                    GUICommands::RemoveSender(node_id, to_remove) => {
                    }
                    GUICommands::AddSender(node_id, to_add) => {
                        add_sender(state, node_id, to_add);
                    }
                    GUICommands::SetPDR(drone, pdr) => {
                        set_pdr(state, drone, pdr);
                    }
                    GUICommands::SendMessageTo(src, dest, msg) => {
                        send_message(state, src, dest, msg.as_str());
                    }
                    GUICommands::RegisterTo(client, server) => {
                        register(state, client, server);
                    }
                    GUICommands::GetClientList(client) => {
                        get_list(state, client);
                    }
                    GUICommands::LogOut(client, server) => {
                        logout(state, client, server);
                    }
                    GUICommands::AskForFileList(client, server) => {
                        ask_for_file_list(state, client, server);
                    }
                    GUICommands::GetFile(client, server, title) => {
                        get_file(state, client, server, title.as_str());
                    }
                    _ => error!("[ {} ] Not supposed to handle {:?}", "GUI".red(), command),
                }
            }
            Err(TrySendError::Full(_)) => {
                error!("[ {} ] Command channel is full", "GUI".red());
            }
            Err(TrySendError::Disconnected(_)) => {
                error!("[ {} ] Command channel disconnected", "GUI".red());
            }
        }
    }

    fn send_spawn_command(&self, state: &mut GUIState, command: GUICommands) {
        match state.sender.try_send(command.clone()) {
            Ok(()) => {
                match command {
                    GUICommands::Spawn(id, neighbor, pdr) => {
                        spawn(state, id, &neighbor.clone(), pdr);
                    }
                    _ => error!("[ {} ] Not supposed to handle {:?}", "GUI".red(), command),
                }
            }
            Err(TrySendError::Full(_)) => {
                error!("[ {} ] Command channel is full", "GUI".red());
            }
            Err(TrySendError::Disconnected(_)) => {
                error!("[ {} ] Command channel disconnected", "GUI".red());
            }
        }
    }*/
}
