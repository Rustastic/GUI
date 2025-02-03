use std::f32::consts::PI;

use colored::Colorize;
use log::{error, info};
use wg_2024::{config::Drone as ConfigDrone, network::NodeId};

use crate::{commands::GUICommands, DroneGUI, SimCtrlGUI};

pub fn topology(sim_ctrl: &mut SimCtrlGUI, topology: Vec<ConfigDrone>) {
    let radius = 200.0;
    let center_x = 400.0;
    let center_y = 400.0;

    for (i, drone) in topology.iter().enumerate() {
        let angle = i as f32 * (2.0 * PI / 10.0);
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();

        let new_drone = DroneGUI::new(drone.clone(), x, y);

        for drone in new_drone.neighbor.clone() {
            if !sim_ctrl.edges.contains_key(&drone) {
                let vec = sim_ctrl.edges.entry(new_drone.id).or_insert_with(Vec::new);
                vec.push(drone);
            }
        }

        sim_ctrl.nodes.insert(new_drone.id, new_drone);
    }

    info!("[ {} ] Successfully composed the topology", "GUI".green());
    sim_ctrl.initialized = true;
}

pub fn crash(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl.sender.send(GUICommands::Crash(instance.id)) {
        Ok(()) => {
            info!(
                "[ {} ] Successfully sent GUICommand::Crash from GUI to Simulation Controller",
                "GUI".green()
            );
            // remove from edge hashmap
            sim_ctrl.edges.remove(&instance.id);

            // remove edges starting from neighbor
            for neighbor_id in instance.neighbor.iter() {
                // get edges starting from neighbor
                if let Some(neighbor_drone) = sim_ctrl.edges.get_mut(neighbor_id) {
                    neighbor_drone.retain(|drone| *drone != instance.id);
                }
            }

            instance.command = None;

            let neighbors = sim_ctrl.nodes.get(drone).unwrap().neighbor.clone();
            let id = sim_ctrl.nodes.get(drone).unwrap().id.clone();
            for node in neighbors {
                let a = sim_ctrl.nodes.get_mut(&node).unwrap();
                a.neighbor.retain(|&x| x != id);
            }

            let id = sim_ctrl.nodes.get(drone).unwrap().id;
            sim_ctrl.nodes.remove(&id);
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::Crash from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }
}

pub fn remove_sender(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId, to_remove: &NodeId) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl
        .sender
        .send(GUICommands::RemoveSender(instance.id, *to_remove))
    {
        Ok(_) => {
            info!(
                "[ {} ] Successfully sent GUICommand::RemoveSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                instance.id,
                to_remove
            );

            if let Some(edge) = sim_ctrl.edges.get_mut(&instance.id) {
                if edge.contains(to_remove) {
                    edge.retain(|&node| node != *to_remove);
                }
            }
            if let Some(edge) = sim_ctrl.edges.get_mut(to_remove) {
                if edge.contains(&instance.id) {
                    edge.retain(|&node| node != instance.id);
                }
            }

            // Remove neighbor from the current instance.
            instance.neighbor.retain(|&x| x != *to_remove);
            instance.command = None;

            // Remove neighbor from to_remove
            let id = sim_ctrl.nodes.get(drone).unwrap().id.clone();
            let neighbor = sim_ctrl.nodes.get_mut(to_remove).unwrap();
            neighbor.neighbor.retain(|&x| x != id);
        }
        Err(e) => {
            error!(
                "[ {} ] Unable to send GUICommand::RemoveSender from GUI to Simulation Controller: {}",
                "GUI".red(),
                e
            )
        }
    }
}

pub fn add_sender(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId, to_add: NodeId) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl
        .sender
        .send(GUICommands::AddSender(instance.id, to_add))
    {
        Ok(_) => {
            info!(
                "[ {} ] Successfully sent GUICommand::AddSender({}, {}) from GUI to Simulation Controller",
                "GUI".green(),
                instance.id,
                to_add
            );

            instance.neighbor.push(to_add);
            sim_ctrl
                .edges
                .entry(*drone)
                .or_insert_with(Vec::new)
                .push(to_add);

            let neighbor = sim_ctrl.nodes.get_mut(&to_add).unwrap();
            neighbor.neighbor.push(*drone);
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::AddSender from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }

    sim_ctrl.nodes.get_mut(drone).unwrap().command = None;
}

pub fn set_pdr(sim_ctrl: &mut SimCtrlGUI, drone: &NodeId, pdr: &f32) {
    let instance = sim_ctrl.nodes.get_mut(drone).unwrap();
    match sim_ctrl.sender.send(GUICommands::SetPDR(instance.id, *pdr)) {
        Ok(_) => {
            info!("[ {} ] Successfully sent GUICommand::SetPDR({}, {}) from GUI to Simulation Controller", "GUI".green(), instance.id, pdr);
            instance.pdr = *pdr;
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::SetPDR from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    }

    instance.command = None;
}

pub fn spawn(sim_ctrl: &mut SimCtrlGUI, id: &NodeId, neighbors: &Vec<NodeId>, pdr: f32) {
    match sim_ctrl
        .sender
        .send(GUICommands::Spawn(*id, neighbors.clone(), pdr))
    {
        Ok(()) => {
            sim_ctrl.spawn_id = None;
            sim_ctrl.spawn_neighbors.clear();
            sim_ctrl.spawn_pdr = None;

            let drone = ConfigDrone {
                id: *id,
                connected_node_ids: neighbors.clone(),
                pdr,
            };

            // add to nodes
            let new_drone = DroneGUI::new(drone, 400.0, 400.0);

            // ad to various instances neighbors
            for drone in neighbors {
                sim_ctrl
                    .nodes
                    .get_mut(drone)
                    .unwrap()
                    .neighbor
                    .push(new_drone.id);
            }

            sim_ctrl.nodes.insert(*id, new_drone);

            // add edges
            sim_ctrl.edges.insert(*id, neighbors.clone());

            sim_ctrl.spawn_command = None;
        }
        Err(e) => error!(
            "[ {} ] Unable to send GUICommand::Spawn from GUI to Simulation Controller: {}",
            "GUI".red(),
            e
        ),
    };
}
