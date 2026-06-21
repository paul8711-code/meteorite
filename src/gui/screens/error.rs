use super::*;

#[derive(Default)]
pub struct ErrorScreen;

impl ErrorScreen {
    pub fn show(&self, ui: &mut egui::Ui, state: &mut Arc<Mutex<UiState>>, err: auth::LoginError) {
        match err {
            auth::LoginError::NoAccountActive => {
                if let Ok(mut state) = state.lock() {
                    *state = UiState::Login;
                }
            }
            _ => self.display_error(ui, err),
        }
    }

    fn display_error(&self, ui: &mut egui::Ui, err: auth::LoginError) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(36, 36, 36)))
            .show_inside(ui, |ui| {
                egui::Area::new("error_area".into())
                    .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            egui::Frame::window(&ui.global_style())
                                .corner_radius(10.0)
                                .fill(egui::Color32::from_rgb(255, 120, 120))
                                .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{}", err))
                                            .color(egui::Color32::from_rgb(20, 20, 20)),
                                    );
                                });

                            ui.add_space(50.0);

                            widgets::add_icon(ui);
                        });
                    });
            });
    }
}
