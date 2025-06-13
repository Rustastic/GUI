use std::collections::HashMap;
use crossbeam_channel::{Receiver, Sender};

use eframe::egui::Color32;

use wg_2024::network::NodeId;

use messages::gui_commands::{GUICommands, GUIEvents};

use crate::logic::nodes::NodeGUI;

/// Main GUI state container
#[derive(Clone, Debug)]
pub struct GUIState {
    pub sender: Sender<GUICommands>,
    pub receiver: Receiver<GUIEvents>,
    
    // Core state
    pub initialized: bool,
    pub nodes: HashMap<NodeId, NodeGUI>,
    pub edges: HashMap<NodeId, (Vec<NodeId>, Color32)>,
    
    // Spawn drone state
    pub spawn: SpawnState,
    
    // File management
    pub file_list: HashMap<NodeId, Vec<String>>,
}

/// State for spawning new drones
#[derive(Clone, Debug, Default)]
pub struct SpawnState {
    pub button_visible: bool,
    pub panel_open: bool,
    pub id: Option<String>,
    pub neighbors: Vec<NodeId>,
    pub pdr: Option<String>,
    pub command: Option<GUICommands>,
}

impl GUIState {
    pub fn new(sender: Sender<GUICommands>, receiver: Receiver<GUIEvents>) -> Self {
        Self {
            sender,
            receiver,
            initialized: false,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            spawn: SpawnState {
                button_visible: true,
                panel_open: false,
                id: None,
                neighbors: Vec::new(),
                pdr: None,
                command: None,
            },
            file_list: HashMap::new(),
        }
    }
    
    /// Reset spawn state after successful spawn
    pub fn reset_spawn_state(&mut self) {
        self.spawn = SpawnState {
            button_visible: true,
            panel_open: false,
            ..Default::default()
        };
    }
}