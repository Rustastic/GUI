use colored::Colorize;
use eframe::egui;
use log::{error, info};

use crate::logic::state::GUIState;
use messages::gui_commands::GUICommands;

pub struct SpawnPanel;

impl SpawnPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&mut self, state: &mut GUIState, ui: &mut egui::Ui) {
        // Show spawn form if panel is open
        if state.spawn.panel_open {
            self.render_spawn_form(state, ui);
        }

        // Show spawn button if visible
        if state.spawn.button_visible && ui.button("Spawn Drone").clicked() {
            state.spawn.panel_open = true;
            state.spawn.button_visible = false;
            info!("[ {} ] Spawn button pressed", "GUI".green());
        }
    }

    fn render_spawn_form(&self, state: &mut GUIState, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Spawn a New Drone");

            // ID Input
            ui.horizontal(|ui| {
                ui.label("Enter Drone ID:");
                let text_id = state.spawn.id.clone().unwrap_or_default();
                let mut buffer_id = text_id;

                let text_edit = ui.text_edit_singleline(&mut buffer_id);
                if text_edit.changed() {
                    state.spawn.id = Some(buffer_id);
                }
            });

            // Neighbor Selection
            self.render_neighbor_selection(state, ui);

            // PDR Input
            ui.horizontal(|ui| {
                ui.label("Enter PDR:");
                let text_pdr = state.spawn.pdr.clone().unwrap_or_default();
                let mut buffer_pdr = text_pdr;

                let text_edit = ui.text_edit_singleline(&mut buffer_pdr);
                if text_edit.changed() {
                    state.spawn.pdr = Some(buffer_pdr);
                }
            });

            // Spawn Button
            if ui.button("Spawn").clicked() {
                self.handle_spawn_click(state);
            }
        });
    }

    fn render_neighbor_selection(&self, state: &mut GUIState, ui: &mut egui::Ui) {
        ui.label("Select Neighbors:");
        egui::ComboBox::from_label("Neighbors")
            .selected_text(format!("{:?}", state.spawn.neighbors))
            .show_ui(ui, |ui| {
                for &neighbor in state.nodes.keys() {
                    let label = format!("{neighbor}");
                    let is_selected = state.spawn.neighbors.contains(&neighbor);

                    if ui.selectable_label(is_selected, label).clicked() {
                        if is_selected {
                            state.spawn.neighbors.retain(|&n| n != neighbor);
                        } else {
                            state.spawn.neighbors.push(neighbor);
                        }
                    }
                }
            });
    }

    fn handle_spawn_click(&self, state: &mut GUIState) {
        if let (Some(id_str), Some(pdr_str)) = (&state.spawn.id, &state.spawn.pdr) {
            if let Ok(id) = id_str.parse::<u8>() {
                if let Ok(pdr) = pdr_str.parse::<f32>() {
                    let neighbors = state.spawn.neighbors.clone();
                    state.spawn.command = Some(GUICommands::Spawn(id, neighbors, pdr));
                    state.reset_spawn_state();
                    info!("[ {} ] Spawning new Drone: {}", "GUI".green(), id);
                } else {
                    error!("[ {} ] Invalid PDR value", "GUI".red());
                }
            } else {
                error!("[ {} ] Invalid ID value", "GUI".red());
            }
        }
    }
}
