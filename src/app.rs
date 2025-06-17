#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use colored::Colorize;
use eframe::egui;
use log::warn;

use crate::{
    logic::{handlers::EventHandler, state::GUIState},
    ui::MainUI,
};

/// Main GUI application struct
pub struct SimCtrlGUI {
    pub state: GUIState,
    event_handler: EventHandler,
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

        // Request continuous repainting for animations
        ctx.request_repaint();
    }
}
