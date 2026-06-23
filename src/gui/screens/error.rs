use super::*;

#[derive(Default)]
pub struct ErrorScreen {
    should_fade: bool,
}

impl ErrorScreen {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut Arc<Mutex<UiState>>,
        err: auth::LoginError,
    ) {
        match err {
            auth::LoginError::NoAccountActive => {
                if let Ok(mut state) = state.lock() {
                    *state = UiState::Login;
                }
            }
            _ => self.display_error(ui, err),
        }
        self.should_fade = true;
    }

    fn display_error(&self, ui: &mut egui::Ui, err: auth::LoginError) {
        egui::Panel::bottom("login_bottom_panel")
            .resizable(false)
            .exact_size(50.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(env!("CARGO_PKG_VERSION"));
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
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

            egui::Area::new("error_area".into())
                .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
                .show(ui, |ui| {
                    let opacity = ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id("error_fade_animation"),
                        self.should_fade,
                        0.25,
                    );

                    ui.set_opacity(opacity);

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

                        widgets::add_icon(ui, egui::Vec2 { x: 256.0, y: 256.0 });
                    });
                });
        });
    }
}
