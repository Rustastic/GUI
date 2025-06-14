use eframe::egui::{self, Color32, Pos2, Vec2, Rect, Sense, Stroke};
use wg_2024::{network::NodeId, packet::NodeType};

use crate::{
    constants::*, logic::{nodes::{types::ClientType, NodeGUI}, state::GUIState}, ui::node::NodeDetails
};
use messages::high_level_messages::ServerType;

pub struct NetworkVisualization {
    node_details: NodeDetails,
}

impl NetworkVisualization {
    pub fn new(node_details: NodeDetails) -> Self {
        Self {
            node_details
        }
    }
    
    pub fn render(&mut self, state: &mut GUIState, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Allocate space for drawing
        let (response, painter) = ui.allocate_painter(
            Vec2::new(WIDTH, HEIGHT), 
            Sense::click_and_drag()
        );
        
        // Offset the painter to account for header space
        let painter = painter.with_clip_rect(Rect::from_min_max(
            egui::pos2(0.0, 100.0),
            egui::pos2(WIDTH, HEIGHT),
        ));
        
        // Draw network connections
        self.draw_connections(&painter, state);
        
        // Draw nodes and handle interactions
        self.draw_nodes_and_handle_interactions(ui, &painter, state, &response);
        
        // Update node colors based on type
        self.update_node_colors(state);

        self.node_details.render(state, ctx);
    }
    
    fn draw_connections(&self, painter: &egui::Painter, state: &GUIState) {
        for (start_id, (neighbors, color)) in &state.edges {
            if let Some(start_node) = state.nodes.get(start_id) {
                for end_id in neighbors {
                    if let Some(end_node) = state.nodes.get(end_id) {
                        painter.line_segment(
                            [
                                egui::pos2(start_node.x, start_node.y),
                                egui::pos2(end_node.x, end_node.y)
                            ],
                            Stroke::new(2.0, *color),
                        );
                    }
                }
            }
        }
    }
    
    fn draw_nodes_and_handle_interactions(
        &self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        state: &mut GUIState,
        response: &egui::Response,
    ) {
        let mut nodes_to_update = Vec::new();
        
        // Collect nodes that need interaction handling
        for (id, node) in &state.nodes {
            nodes_to_update.push((*id, node.x, node.y, node.color));
        }
        
        // Handle interactions and draw nodes
        for (node_id, x, y, color) in nodes_to_update {
            let screen_pos = egui::pos2(x, y);
            
            // Create interaction area for the node
            let node_rect = Rect::from_center_size(
                screen_pos, 
                Vec2::splat(NODE_RADIUS * 2.0)
            );
            
            let node_response = ui.allocate_rect(node_rect, Sense::click());
            
            // Handle node selection
            if node_response.clicked() {
                if let Some(node) = state.nodes.get_mut(&node_id) {
                    node.selected = true;
                    println!("Node {} selected = true", node.id);
                }
            }
            
            // Handle node dragging
            if response.dragged() {
                if let Some(hover_pos) = response.hover_pos() {
                    if node_rect.contains(hover_pos) {
                        if let Some(node) = state.nodes.get_mut(&node_id) {
                            let delta = response.drag_delta();
                            node.x += delta.x;
                            node.y += delta.y;
                            
                            // Keep nodes within bounds
                            node.x = node.x.clamp(NODE_RADIUS, WIDTH - NODE_RADIUS);
                            node.y = node.y.clamp(100.0 + NODE_RADIUS, HEIGHT - NODE_RADIUS);
                        }
                    }
                }
            }
            
            // Draw the node
            painter.circle_filled(screen_pos, NODE_RADIUS, color);
            
            // Optional: Draw node ID as text
            if NODE_RADIUS > 8.0 {
                painter.text(
                    screen_pos,
                    egui::Align2::CENTER_CENTER,
                    node_id.to_string(),
                    egui::FontId::proportional(10.0),
                    Color32::BLACK,
                );
            }
        }
    }
    
    fn update_node_colors(&self, state: &mut GUIState) {
        // Update colors based on node type
        for (_, node) in state.nodes.iter_mut() {
            // Only update if not in special state (e.g., packet animation)
            if !node.pending_reset {
                node.color = self.get_node_color(node);
            }
        }
    }
    
    fn get_node_color(&self, node: &NodeGUI) -> Color32 {
        match node.node_type {
            NodeType::Drone => DRONE_COLOR,
            NodeType::Server => {
                match node.server_type {
                    Some(ServerType::Chat) => COMMUNICATION_SERVER_COLOR,
                    Some(ServerType::Text) => TEXT_CONTENT_SERVER_COLOR,
                    Some(ServerType::Media) => MEDIA_CONTENT_SERVER_COLOR,
                    None => Color32::GRAY,
                }
            }
            NodeType::Client => {
                match node.client_type {
                    Some(ClientType::Chat) => CHAT_CLIENT_COLOR,
                    Some(ClientType::Media) => MEDIA_CLIENT_COLOR,
                    None => Color32::GRAY,
                }
            }
        }
    }
    
    fn categorize_nodes(&self, state: &GUIState) -> NodeCategories {
        let mut categories = NodeCategories::default();
        
        for (id, node) in &state.nodes {
            match node.node_type {
                NodeType::Client => {
                    match node.client_type {
                        Some(ClientType::Chat) => categories.chat_clients.push(*id),
                        Some(ClientType::Media) => categories.media_clients.push(*id),
                        None => {}
                    }
                }
                NodeType::Server => {
                    match node.server_type {
                        Some(ServerType::Chat) => categories.communication_servers.push(*id),
                        Some(ServerType::Text) => categories.text_servers.push(*id),
                        Some(ServerType::Media) => categories.media_servers.push(*id),
                        None => {}
                    }
                }
                NodeType::Drone => {
                    categories.drones.push(*id);
                }
            }
        }
        
        categories
    }
}

#[derive(Default)]
struct NodeCategories {
    pub chat_clients: Vec<NodeId>,
    pub media_clients: Vec<NodeId>,
    pub communication_servers: Vec<NodeId>,
    pub text_servers: Vec<NodeId>,
    pub media_servers: Vec<NodeId>,
    pub drones: Vec<NodeId>,
}

impl NetworkVisualization {
    /// Helper method to get communication servers for registration
    pub fn get_communication_servers(&self, state: &GUIState) -> Vec<NodeId> {
        let categories = self.categorize_nodes(state);
        categories.communication_servers
    }
    
    /// Helper method to get text content servers for file requests
    pub fn get_text_content_servers(&self, state: &GUIState) -> Vec<NodeId> {
        let categories = self.categorize_nodes(state);
        categories.text_servers
    }
    
    /// Helper method to get media content servers
    pub fn get_media_content_servers(&self, state: &GUIState) -> Vec<NodeId> {
        let categories = self.categorize_nodes(state);
        categories.media_servers
    }
    
    /// Set node position programmatically
    pub fn set_node_position(&self, state: &mut GUIState, node_id: NodeId, x: f32, y: f32) {
        if let Some(node) = state.nodes.get_mut(&node_id) {
            node.x = x.clamp(NODE_RADIUS, WIDTH - NODE_RADIUS);
            node.y = y.clamp(100.0 + NODE_RADIUS, HEIGHT - NODE_RADIUS);
        }
    }
    
    /// Auto-arrange nodes in a circle layout
    pub fn auto_arrange_nodes(&self, state: &mut GUIState) {
        let node_count = state.nodes.len();
        if node_count == 0 {
            return;
        }
        
        let center_x = WIDTH / 2.0;
        let center_y = (HEIGHT + 100.0) / 2.0;
        let radius = (WIDTH.min(HEIGHT - 100.0) / 2.0) - NODE_RADIUS * 2.0;
        
        for (i, (_, node)) in state.nodes.iter_mut().enumerate() {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / (node_count as f32);
            node.x = center_x + radius * angle.cos();
            node.y = center_y + radius * angle.sin();
        }
    }
    
    /// Get node at position (for hover detection)
    pub fn get_node_at_position(&self, state: &GUIState, pos: Pos2) -> Option<NodeId> {
        for (id, node) in &state.nodes {
            let node_pos = egui::pos2(node.x, node.y);
            let distance = (pos - node_pos).length();
            if distance <= NODE_RADIUS {
                return Some(*id);
            }
        }
        None
    }
}