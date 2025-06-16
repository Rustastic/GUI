use eframe::egui::Color32;

/// Main window height
pub const HEIGHT: f32 = 900.0;

/// Main window width  
pub const WIDTH: f32 = 900.0;

/// Node visualization radius
pub const NODE_RADIUS: f32 = 10.0;

/// Node type colors for visualization
pub const DRONE_COLOR: Color32 = Color32::LIGHT_BLUE;
pub const COMMUNICATION_SERVER_COLOR: Color32 = Color32::GREEN;
pub const TEXT_CONTENT_SERVER_COLOR: Color32 = Color32::PURPLE;
pub const MEDIA_CONTENT_SERVER_COLOR: Color32 = Color32::RED;
pub const CHAT_CLIENT_COLOR: Color32 = Color32::YELLOW;
pub const MEDIA_CLIENT_COLOR: Color32 = Color32::ORANGE;

/// UI spacing and positioning
pub const LEGEND_Y_POS: f32 = 40.0;
pub const LEGEND_X_START: f32 = 10.0;
pub const LEGEND_SPACING: f32 = 5.0;

/// Animation timing
pub const PACKET_ANIMATION_DURATION_SECS: f32 = 0.005;
