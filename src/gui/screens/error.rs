use super::{Arc, ErrorKind, Mutex, UiState, egui, widgets};

#[derive(Default)]
pub struct ErrorScreen;

impl ErrorScreen {
    pub fn show(
        ui: &mut egui::Ui,
        state: &mut Arc<Mutex<UiState>>,
        kind: &ErrorKind,
        message: &str,
    ) {
        match kind {
            ErrorKind::NoAccountActive => {
                if let Ok(mut state) = state.lock() {
                    *state = UiState::Login;
                }
            }
            ErrorKind::Other => Self::display_error(ui, message),
        }
    }

    fn display_error(ui: &mut egui::Ui, err: &str) {
        widgets::bottom_info_bar(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            widgets::draw_bg(ui);

            egui::Area::new("error_area".into())
                .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
                .show(ui, |ui| {
                    let opacity = ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id(("error_screen", "fade")),
                        true,
                        0.25,
                    );

                    ui.set_opacity(opacity);

                    ui.vertical_centered(|ui| {
                        egui::Frame::window(&ui.global_style())
                            .corner_radius(10.0)
                            .fill(egui::Color32::from_rgb(255, 120, 120))
                            .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(err)
                                        .color(egui::Color32::from_rgb(20, 20, 20)),
                                );
                            });

                        ui.add_space(50.0);

                        widgets::add_icon(ui, egui::Vec2::splat(256.0));
                    });
                });
        });
    }
}
