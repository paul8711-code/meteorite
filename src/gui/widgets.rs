use super::*;

// reusable components
pub fn add_icon(ui: &mut egui::Ui, size: egui::Vec2) {
    ui.add(
        egui::Image::from_bytes("bytes://icon.png", ICON)
            .corner_radius(15.0)
            .fit_to_exact_size(size),
    );
}
