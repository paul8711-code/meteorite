use super::{ICON, egui};

// reusable components
pub fn add_icon(ui: &mut egui::Ui, size: egui::Vec2) {
    ui.add(
        egui::Image::from_bytes("bytes://icon.png", ICON)
            .corner_radius(15.0)
            .fit_to_exact_size(size),
    );
}

pub fn draw_bg(ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.painter().add(egui::Shape::gradient_rect(
            ui.ctx().viewport_rect(),
            egui::Direction::TopDown,
            [
                egui::Color32::from_rgb(20, 20, 20),
                egui::Color32::from_rgb(0, 0, 60),
            ],
        ));
    });
}

pub fn bottom_info_bar(ui: &mut egui::Ui) {
    egui::Panel::bottom("bottom_info_panel")
        .resizable(false)
        .exact_size(50.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(env!("CARGO_PKG_VERSION"));
            });
        });
}
