use colored::Colorize;
use log::{error, info};

use wg_2024::{network::NodeId, packet::NodeType};

use messages::gui_commands::GUICommands;

use crate::{
    logic::state::GUIState,
};

pub fn crash(state: &mut GUIState, drone: NodeId) {
    if state.nodes.contains_key(&drone) {
        for node in state.nodes.get(&drone).unwrap().neighbor.clone() {
            let instance = state.nodes.get(&node).unwrap();
            if (instance.node_type == NodeType::Client && instance.neighbor.len() == 1)
                || (instance.node_type == NodeType::Server && instance.neighbor.len() == 2)
            {
                error!(
                    "[ {} ]: failed to crash a [ Drone {} ] due to Client/Server connection rules",
                    "GUI".red(),
                    instance.id
                );
                state.nodes.get_mut(&drone).unwrap().command = None;
                return;
            }
        }
    }

    let instance = state.nodes.get_mut(&drone).unwrap();
    match state.sender.send(GUICommands::Crash(instance.id)) {
        Ok(()) => {
            info!(
                "[ {} ] Successfully sent GUICommand::Crash from GUI to Simulation Controller",
                "GUI".green()
            );
            // remove from edge hashmap
            state.edges.remove(&instance.id);

            // remove edges starting from neighbor
            for neighbor_id in &instance.neighbor {
                // get edges starting from neighbor
                if let Some(neighbor_drone) = state.edges.get_mut(neighbor_id) {
                    neighbor_drone.0.retain(|drone| *drone != instance.id);
                }
            }

            instance.command = None;

            let neighbors = state.nodes.get(&drone).unwrap().neighbor.clone();
            let id = state.nodes.get(&drone).unwrap().id;
            for node in neighbors {
                let a = state.nodes.get_mut(&node).unwrap();
                a.neighbor.retain(|&x| x != id);
            }

            let id = state.nodes.get(&drone).unwrap().id;
            state.nodes.remove(&id);
        }
        Err(e) => {
            error!(
                "[ {} ] Unable to send GUICommand::Crash from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            );
        }
    }
}

pub fn set_pdr(state: &mut GUIState, drone: NodeId, pdr: f32) {
    let instance = state.nodes.get_mut(&drone).unwrap();
    match state.sender.send(GUICommands::SetPDR(instance.id, pdr)) {
        Ok(()) => {
            info!("[ {} ] Successfully sent GUICommand::SetPDR({}, {}) from GUI to Simulation Controller", "GUI".green(), instance.id, pdr);
            instance.pdr = pdr;
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::SetPDR from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }

    instance.command = None;
}

pub fn spawn(state: &mut GUIState, id: NodeId, neighbors: &Vec<NodeId>, pdr: f32) {
    match state.sender.send(GUICommands::Spawn(id, neighbors.clone(), pdr)) {
        Ok(()) => info!(
            "[ {} ] Successfully sent GUICommand::Spawn({}, {:?}, {}) from GUI to Simulation Controller",
            "GUI".green(),
            id,
            neighbors,
            pdr
        ),
        Err(e) => {
            error!("[ {} ] Unable to send GUICommand::Spawn({}, {:?}, {}) from GUI to Simulation Controller: {}",
                "GUI".red(),
                id,
                neighbors,
                pdr,
                e
            );
        },
    }
    state.spawn.command = None;
}
