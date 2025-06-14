use eframe::egui;
use crate::{
    logic::state::GUIState,
    ui::{legend::Legend, network::NetworkVisualization, node::NodeDetails, spawn::SpawnPanel},
};

/// Main UI coordinator
pub struct MainUI {
    spawn_panel: SpawnPanel,
    legend: Legend,
    network_viz: NetworkVisualization,
}

impl MainUI {
    pub fn new() -> Self {
        Self {
            spawn_panel: SpawnPanel::new(),
            legend: Legend::new(),
            network_viz: NetworkVisualization::new(NodeDetails::new()),
        }
    }
    
    pub fn render(&mut self, state: &mut GUIState, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simulation Controller");
            
            // Render legend
            self.legend.render(ui);
            
            ui.add_space(10.0);
            
            // Render spawn controls
            self.spawn_panel.render(state, ui);
            
            // Render network visualization
            self.network_viz.render(state, ui, ctx);
        });
    }
}