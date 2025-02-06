use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone},
    network::NodeId,
    packet::Packet,
};

// From SimCtrl to GUI
#[derive(Debug, Clone)]
pub enum GUIEvents {
    PacketSent(NodeId, NodeId, Packet),
    PacketDropped(NodeId, Packet),
    Topology(Vec<ConfigDrone>, Vec<ConfigClient>),

    CommunicationServerList(Vec<NodeId>),
    MessageReceived(NodeId, String),
    ClientList(Vec<NodeId>),
}

// From GUI to SimCtrl
#[derive(Debug, Clone)]
pub enum GUICommands {
    Spawn(NodeId, Vec<NodeId>, f32),
    Crash(NodeId),
    RemoveSender(NodeId, NodeId),
    AddSender(NodeId, NodeId),
    SetPDR(NodeId, f32),

    SendMessageTo(NodeId, NodeId, String),
    RegisterTo(NodeId, NodeId),
    LogOut(NodeId, NodeId),
}
