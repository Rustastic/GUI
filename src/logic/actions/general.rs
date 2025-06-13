use eframe::egui::Color32;

use colored::Colorize;
use log::{error, info};

use wg_2024::{
    network::NodeId,
    packet::NodeType,
};

use messages::gui_commands::GUICommands;

use crate::logic::state::GUIState;

pub fn remove_sender(state: &mut GUIState, node_id: NodeId, to_remove: NodeId) {
    if state.nodes.get(&node_id).unwrap().node_type == NodeType::Client
        && state.nodes.get(&node_id).unwrap().neighbor.len() == 1
    {
        state.nodes.get_mut(&node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender to [ Client {} ] from GUI to Simulation Controller: {}",
            "GUI".red(),
            node_id,
            "Each client must be connected to at least one and at most two drones"
        );
        return;
    } else if state.nodes.get(&to_remove).unwrap().node_type == NodeType::Client
        && state.nodes.get(&to_remove).unwrap().neighbor.len() == 1
    {
        state.nodes.get_mut(&node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender to [ Client {} ] from GUI to Simulation Controller: {}",
            "GUI".red(),
            to_remove,
            "Each client must be connected to at least one and at most two drones"
        );
        return;
    }

    if state.nodes.get(&node_id).unwrap().node_type == NodeType::Server
        && state.nodes.get(&node_id).unwrap().neighbor.len() == 2
    {
        state.nodes.get_mut(&node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender to [ Server {} ] from GUI to Simulation Controller: {}",
            "GUI".red(),
            node_id,
            "Each server must be connected to least two drones"
        );
        return;
    } else if state.nodes.get(&to_remove).unwrap().node_type == NodeType::Server
        && state.nodes.get(&to_remove).unwrap().neighbor.len() == 2
    {
        state.nodes.get_mut(&node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::RemoveSender to [ Server {} ] from GUI to Simulation Controller: {}",
            "GUI".red(),
            to_remove,
            "Each server must be connected to least two drones"
        );
        return;
    }

    let instance = state.nodes.get_mut(&node_id).unwrap();
    match state
        .sender
        .send(GUICommands::RemoveSender(instance.id, to_remove))
    {
        Ok(()) => {
            info!(
                "[ {} ] Successfully sent GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                instance.id,
                to_remove
            );

            if let Some(edge) = state.edges.get_mut(&instance.id) {
                if edge.0.contains(&to_remove) {
                    edge.0.retain(|&node| node != to_remove);
                }
            }
            if let Some(edge) = state.edges.get_mut(&to_remove) {
                if edge.0.contains(&instance.id) {
                    edge.0.retain(|&node| node != instance.id);
                }
            }

            // Remove neighbor from the current instance.
            instance.neighbor.retain(|&x| x != to_remove);
            instance.command = None;

            // Remove neighbor from to_remove
            let id = state.nodes.get(&node_id).unwrap().id;
            let neighbor = state.nodes.get_mut(&to_remove).unwrap();
            neighbor.neighbor.retain(|&x| x != id);
        }
        Err(e) => {
            error!(
                "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            );
        }
    }
}

pub fn add_sender(state: &mut GUIState, node_id: NodeId, to_add: NodeId) {
    if state.nodes.get(&node_id).unwrap().node_type == NodeType::Client
        && state.nodes.get(&to_add).unwrap().node_type == NodeType::Client
    {
        state.nodes.get_mut(&node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            "Two clients can be connected between eachother"
        );
        return;
    } else if state.nodes.get(&node_id).unwrap().node_type == NodeType::Client {
        if state.nodes.get(&node_id).unwrap().neighbor.len() == 2 {
            state.nodes.get_mut(&node_id).unwrap().command = None;
            error!(
                "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                "Each client must be connected to at most two drones"
            );
            return;
        }
    } else if state.nodes.get(&to_add).unwrap().node_type == NodeType::Client
        && state.nodes.get(&to_add).unwrap().neighbor.len() == 2
    {
        state.nodes.get_mut(&node_id).unwrap().command = None;
        error!(
            "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            "Each client must be connected to at most two drones"
        );
        return;
    }

    let instance = state.nodes.get_mut(&node_id).unwrap();
    match state
        .sender
        .send(GUICommands::AddSender(instance.id, to_add))
    {
        Ok(()) => {
            info!(
                "[ {} ] Successfully sent GUICommand::AddSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                instance.id,
                to_add
            );

            instance.neighbor.push(to_add);
            let (vec, _) = state
                .edges
                .entry(node_id)
                .or_insert_with(|| (Vec::new(), Color32::GRAY));
            vec.push(to_add);

            let neighbor = state.nodes.get_mut(&to_add).unwrap();
            neighbor.neighbor.push(node_id);
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }

    state.nodes.get_mut(&node_id).unwrap().command = None;
}
