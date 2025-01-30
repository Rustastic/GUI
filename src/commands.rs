use wg_2024::{config::Drone as ConfigDrone, drone::Drone, network::NodeId, packet::Packet};

// From SimCtrl to GUI
pub enum GUIEvents {
    PacketSent(NodeId, NodeId, Packet),
    PacketDropped(NodeId, Packet),
    Topology(Vec<ConfigDrone>),
}

// From GUI to SimCtrl
pub enum GUICommands {
    Spawn(Box<dyn Drone>),
    Crash(NodeId),
    RemoveSender(NodeId, NodeId),
    AddSender(NodeId, NodeId),
    SetPDR(NodeId, f32),
}
