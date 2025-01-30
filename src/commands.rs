use wg_2024::{config::Drone as ConfigDrone, drone::Drone, network::NodeId, packet::Packet};

// From SimCtrl to GUI
#[derive(Debug, Clone)]
pub enum GUIEvents {
    PacketSent(NodeId, NodeId, Packet),
    PacketDropped(NodeId, Packet),
    Topology(Vec<ConfigDrone>),
}

// From GUI to SimCtrl
#[derive(Debug, Clone)]
pub enum GUICommands {
    Spawn,
    Crash(NodeId),
    RemoveSender(NodeId, NodeId),
    AddSender(NodeId, NodeId),
    SetPDR(NodeId, f32),
}
