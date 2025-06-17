use crate::constants::{
    CHAT_CLIENT_COLOR, COMMUNICATION_SERVER_COLOR, DRONE_COLOR, LEGEND_SPACING, LEGEND_X_START,
    LEGEND_Y_POS, MEDIA_CLIENT_COLOR, MEDIA_CONTENT_SERVER_COLOR, TEXT_CONTENT_SERVER_COLOR,
};
use eframe::egui::{self, Pos2};

pub struct Legend;

impl Default for Legend {
    fn default() -> Self {
        Self::new()
    }
}

impl Legend {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, ui: &mut egui::Ui) {
        let circles = [
            (LEGEND_X_START, DRONE_COLOR, "Drone"),
            (
                LEGEND_X_START + 65.0,
                COMMUNICATION_SERVER_COLOR,
                "CommunicationServer",
            ),
            (
                LEGEND_X_START + 220.0,
                TEXT_CONTENT_SERVER_COLOR,
                "TextContentServer",
            ),
            (
                LEGEND_X_START + 352.5,
                MEDIA_CONTENT_SERVER_COLOR,
                "MediaContentServer",
            ),
            (LEGEND_X_START + 495.0, CHAT_CLIENT_COLOR, "ChatClient"),
            (LEGEND_X_START + 582.5, MEDIA_CLIENT_COLOR, "MediaClient"),
        ];

        ui.horizontal(|ui| {
            for (x, color, label) in circles {
                ui.horizontal(|ui| {
                    let center = Pos2::new(x, LEGEND_Y_POS);
                    ui.painter()
                        .add(egui::Shape::circle_filled(center, 5.0, color));
                    ui.add_space(15.0);
                    ui.label(label);
                });
                ui.add_space(LEGEND_SPACING);
            }
        });
    }
}
