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
