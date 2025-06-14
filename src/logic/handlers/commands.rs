use crossbeam_channel::TrySendError;

use colored::Colorize;
use log::{error, info};

use messages::gui_commands::GUICommands;

use crate::logic::{actions::*, state::GUIState};

pub struct CommandHandler;

impl CommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_commands(&self, state: &mut GUIState) {
        // Handle spawn command
        if let Some(command) = state.spawn.command.take() {
            self.send_spawn_command(state, command);
        }

        // Handle node-specific commands
        for (_, node) in state.nodes.clone().iter_mut() {
            if let Some(command) = node.command.take() {
                self.send_command(state, command);
            }
        }
    }

    fn send_command(&self, state: &mut GUIState, command: GUICommands) {
        match state.sender.try_send(command.clone()) {
            Ok(()) => {
                info!("[ {} ] Handling {:?}", "GUI".green(), command);
                match command {
                    GUICommands::Crash(drone) => {
                        crash(state, drone);
                    }
                    GUICommands::RemoveSender(drone, to_remove) => {
                        remove_sender(state, drone, to_remove);
                    }
                    GUICommands::AddSender(drone, to_add) => {
                        add_sender(state, drone, to_add);
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
                info!("[ {} ] Handling {:?}", "GUI".green(), command);
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
    }
}
