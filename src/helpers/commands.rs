use wg_2024::{
    config::{Client as ConfigClient, Drone as ConfigDrone, Server as ConfigServer},
    network::NodeId,
    packet::Packet,
};

// From SimCtrl to GUI
#[derive(Debug, Clone)]
pub enum GUIEvents {
    Topology(Vec<ConfigDrone>, Vec<ConfigClient>, Vec<ConfigServer>),
    FileList(NodeId, Vec<String>),

    PacketSent(NodeId, NodeId, Packet),
    PacketDropped(NodeId, Packet),
    MessageReceived(NodeId, String),
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

    AskForFileList(NodeId, NodeId),
    GetFile(NodeId, NodeId, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientType {
    Chat,
    Media,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerType {
    Communication,
    Text,
    Image,
}
