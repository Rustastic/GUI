#[derive(Debug, Clone)]
pub struct DroneParams {
    pub crashed: bool,
    pub set_pdr: bool,
    pub pdr_value: Option<String>,
}

impl DroneParams {
    pub fn new() -> Self {
        Self {
            crashed: false,
            set_pdr: false,
            pdr_value: None,
        }
    }
}

impl Default for DroneParams {
    fn default() -> Self {
        Self::new()
    }
}