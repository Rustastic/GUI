use wg_2024::{drone::Drone, network::NodeId, packet::Packet};

// From SimCtrl to GUI
pub enum GUIEvents {
    PacketSent(NodeId, NodeId),
    PacketDropped(NodeId, Packet),
}

// From GUI to SimCtrl
pub enum GUICommands {
    Spawn(Box<dyn Drone>),
    Crash(NodeId),
    RemoveSender(NodeId, NodeId),
    AddSender(NodeId, NodeId),
    SetPDR(NodeId, f32),
}