use wg_2024::network::NodeId;

#[derive(Debug, Clone)]
pub struct MediaParams {
    pub ask_for_file_list: bool,
    pub server_value: Option<NodeId>,
    pub get_file: bool,
}

impl MediaParams {
    pub fn new() -> Self {
        Self {
            ask_for_file_list: false,
            server_value: None,
            get_file: false,
        }
    }
}

impl Default for MediaParams {
    fn default() -> Self {
        Self::new()
    }
}
