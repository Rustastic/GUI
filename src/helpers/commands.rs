use wg_2024::{config::Drone as ConfigDrone, network::NodeId, packet::Packet};

// From SimCtrl to GUI
#[derive(Debug, Clone)]
pub enum GUIEvents {
    PacketSent(NodeId, NodeId, Packet),
    PacketDropped(NodeId, Packet),
    Topology(Vec<ConfigDrone>),

    CommunicationServerList(Vec<NodeId>),
    MessageReceived(NodeId, String),
    ClientList(Vec<NodeId>),
    UnreachableClient(NodeId),
    ErrorNotRunning,
    ErrorNotRegistered,
}

// From GUI to SimCtrl
#[derive(Debug, Clone)]
pub enum GUICommands {
    Spawn(NodeId, Vec<NodeId>, f32),
    Crash(NodeId),
    RemoveSender(NodeId, NodeId),
    AddSender(NodeId, NodeId),
    SetPDR(NodeId, f32),
}
