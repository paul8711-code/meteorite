use super::*;

#[derive(Default)]
pub struct LoadingScreen {
    login_started: bool,
}

impl LoadingScreen {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut Arc<Mutex<UiState>>) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(36, 36, 36)))
            .show_inside(ui, |ui| {
                let screen_rect = ui.max_rect();
                let rect_size = egui::vec2(100.0, 100.0);
                let screen_center = screen_rect.center();
                let centered_rect = egui::Rect::from_center_size(screen_center, rect_size);

                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(50.0);
                    ui.add(
                        egui::Image::from_bytes("bytes://icon.png", ICON)
                            .corner_radius(15.0)
                            .fit_to_exact_size(egui::Vec2 { x: 256.0, y: 256.0 }),
                    );
                });

                egui::widgets::Spinner::new().paint_at(ui, centered_rect);
                self.try_login(ui.ctx().clone(), state);
            });
    }

    fn try_login(&mut self, ctx: egui::Context, state: &mut Arc<Mutex<UiState>>) {
        if !self.login_started {
            self.login_started = true;
            let state_clone = std::sync::Arc::clone(state);
            tokio::spawn(async move {
                match auth::login().await {
                    Ok(_client) => {
                        if let Ok(mut state) = state_clone.lock() {
                            *state = UiState::Main;
                            // for instant repaint after
                            ctx.request_repaint();
                        }
                    }
                    Err(e) => {
                        if let Ok(mut state) = state_clone.lock() {
                            *state = UiState::Error(e);
                            ctx.request_repaint();
                        }
                    }
                }
            });
        }
    }
}
