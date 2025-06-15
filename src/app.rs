#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use colored::Colorize;
use eframe::egui;
use log::warn;

use crate::{
    logic::{
        handlers::{CommandHandler, EventHandler},
        state::GUIState,
    },
    ui::MainUI,
};

/// Main GUI application struct
pub struct SimCtrlGUI {
    pub state: GUIState,
    event_handler: EventHandler,
    command_handler: CommandHandler,
    main_ui: MainUI,
}

impl SimCtrlGUI {
    #[must_use]
    pub fn new(
        sender: crossbeam_channel::Sender<messages::gui_commands::GUICommands>,
        receiver: crossbeam_channel::Receiver<messages::gui_commands::GUIEvents>,
    ) -> Self {
        let state = GUIState::new(sender, receiver);

        Self {
            event_handler: EventHandler::new(),
            command_handler: CommandHandler::new(),
            main_ui: MainUI::new(),
            state,
        }
    }
}

impl eframe::App for SimCtrlGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.initialized {
            // Handle incoming events
            self.event_handler.handle_events(&mut self.state, ctx);

            // Render main UI
            self.main_ui.render(&mut self.state, ctx);
        } else {
            warn!("[ {} ] Waiting for initialization", "GUI".green());
            // Handle initialization
            self.event_handler
                .handle_initialization(&mut self.state, ctx);
        }

        let mut tserver_list = Vec::<wg_2024::network::NodeId>::new();
        for (id, instance) in &self.state.nodes {
            if instance.node_type == wg_2024::packet::NodeType::Server && instance.server_type.unwrap() == messages::high_level_messages::ServerType::Text {
                tserver_list.push(*id);
            }
        }

        println!("{:?}", tserver_list);


        // Process pending commands
        self.command_handler.handle_commands(&mut self.state);

        // Request continuous repainting for animations
        ctx.request_repaint();
    }
}
