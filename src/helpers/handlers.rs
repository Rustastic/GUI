use eframe::egui::{Color32, Context};
use std::{thread, time::Instant};

use colored::Colorize;
use log::{error, info};

use messages::gui_commands::{GUICommands, GUIEvents};

use crate::SimCtrlGUI;

impl SimCtrlGUI {
    pub fn handle_events(&mut self, event: GUIEvents, ctx: &Context) {
        match event {
            // light up edge for 0.5 sec in green
            GUIEvents::PacketSent(src, _, _) => {
                if let Some(node) = self.nodes.get_mut(&src) {
                    node.color = Color32::ORANGE;
                }
                self.nodes.get_mut(&src).unwrap().last_packet_time = Some(Instant::now());
                self.nodes.get_mut(&src).unwrap().pending_reset = true;
                ctx.request_repaint();
            }
            // light up node  for 0.5 sec in red
            GUIEvents::PacketDropped(src, _) => {
                if let Some(node) = self.nodes.get_mut(&src) {
                    node.color = Color32::RED;
                }
                self.nodes.get_mut(&src).unwrap().last_packet_time = Some(Instant::now());
                self.nodes.get_mut(&src).unwrap().pending_reset = true;
                ctx.request_repaint();
            }
            GUIEvents::Topology(drones, clients, servers) => {
                info!("[ {} ]: Received Topology", "GUI".green());
                self.topology(&drones, &clients, &servers);
            }

            GUIEvents::ClientList(client, client_list) => {
                info!(
                    "[ {} ]: Received ClientList from [ Client {} ]",
                    "GUI".green(),
                    client
                );
                self.nodes
                    .get_mut(&client)
                    .unwrap()
                    .chat_params
                    .client_list_value = Some(client_list);
            }

            // show message
            GUIEvents::MessageReceived(src, dest, msg) => {
                info!(
                    "[ {} ]: [ Client {} ] received the message {:?} from [ Client {} ]",
                    "GUI".green(),
                    dest,
                    msg,
                    src
                );
                self.nodes
                    .get_mut(&dest)
                    .unwrap()
                    .chat_params
                    .recv_message_client_value = Some(msg);
            }
            GUIEvents::FileList(server, _, items) => {
                info!(
                    "[ {} ]: Received FileList from [ Client {} ]",
                    "GUI".green(),
                    server
                );
                self.file_list.insert(server, items);
            }
        }
    }

    pub fn handle_commands(&mut self) {
        for (_, instance) in self.nodes.clone() {
            if let Some(command) = &instance.command {
                info!("[ {} ] Handling {:?}", "GUI".green(), command);
                #[allow(clippy::match_wildcard_for_single_variants)]
                match command {
                    GUICommands::Crash(drone) => {
                        self.crash(*drone);
                    }
                    GUICommands::RemoveSender(drone, to_remove) => {
                        self.remove_sender(*drone, *to_remove);
                    }
                    GUICommands::AddSender(drone, to_add) => {
                        self.add_sender(*drone, *to_add);
                    }
                    GUICommands::SetPDR(drone, pdr) => {
                        self.set_pdr(*drone, *pdr);
                    }
                    GUICommands::SendMessageTo(src, dest, msg) => {
                        self.send_message(*src, *dest, msg);
                    }
                    GUICommands::RegisterTo(client, server) => {
                        self.register(*client, *server);
                    }
                    GUICommands::GetClientList(client) => {
                        self.get_list(*client);
                    }
                    GUICommands::LogOut(client, server) => {
                        self.logout(*client, *server);
                    }
                    GUICommands::AskForFileList(client, server) => {
                        self.ask_for_file_list(*client, *server);
                    }
                    GUICommands::GetFile(client, server, title) => {
                        self.get_file(*client, *server, title);
                    }
                    _ => error!("[ {} ] Not supposed to handle {:?}", "GUI".red(), command),
                }
            }
        }

        if let Some(command) = &self.spawn_command.clone() {
            info!("[ {} ] Handling {:?}", "GUI".green(), command);
            if let GUICommands::Spawn(id, neighbor, pdr) = command {
                self.spawn(*id, &neighbor.clone(), *pdr);
            } else {
                error!("[ {} ] Not supposed to handle {:?}", "GUI".red(), command);
            }
        }
    }
}
