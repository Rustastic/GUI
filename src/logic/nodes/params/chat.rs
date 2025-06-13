use wg_2024::network::NodeId;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct ChatParam {
    pub send_message: bool,
    pub send_message_msg_value: Option<String>,
    pub send_message_client_value: Option<String>,
    pub recv_message_client_value: Option<String>,
    pub register_to: bool,
    pub register_value: Option<NodeId>,
    pub get_client_list: bool,
    pub client_list_value: Option<Vec<NodeId>>,
    pub logout: bool,
}

impl ChatParam {
    pub fn new() -> Self {
        Self {
            send_message: false,
            send_message_msg_value: None,
            send_message_client_value: None,
            recv_message_client_value: None,
            register_to: false,
            register_value: None,
            get_client_list: false,
            client_list_value: None,
            logout: false,
        }
    }
}

impl Default for ChatParam {
    fn default() -> Self {
        Self::new()
    }
}