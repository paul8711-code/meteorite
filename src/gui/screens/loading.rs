use super::{Arc, Mutex, UiState, auth, egui, widgets};

#[derive(Default)]
pub struct LoadingScreen {
    login_started: bool,
    should_fade: Arc<Mutex<bool>>,
    opacity: Arc<Mutex<f32>>,
}

impl LoadingScreen {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut Arc<Mutex<UiState>>) {
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

            ui.scope(|ui| {
                if let Ok(mut opacity) = self.opacity.lock() {
                    if let Ok(should_fade) = self.should_fade.lock() {
                        *opacity = ui.ctx().animate_bool_with_time(
                            ui.make_persistent_id("loading_fade_animation"),
                            !*should_fade,
                            0.25,
                        );
                    }

                    ui.set_opacity(*opacity);
                }

                let screen_rect = ui.max_rect();
                let rect_size = egui::vec2(100.0, 100.0);
                let screen_center = screen_rect.center();
                let centered_rect = egui::Rect::from_center_size(screen_center, rect_size);

                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(50.0);
                    widgets::add_icon(ui, egui::Vec2 { x: 256.0, y: 256.0 });
                });

                egui::widgets::Spinner::new().paint_at(ui, centered_rect);
                self.try_login(ui.ctx().clone(), state);
            });
        });
    }

    fn try_login(&mut self, ctx: egui::Context, state: &mut Arc<Mutex<UiState>>) {
        let state_clone = Arc::clone(state);
        let should_fade_clone = Arc::clone(&self.should_fade);
        let opacity_clone = Arc::clone(&self.opacity);

        if self.login_started {
            return;
        }

        self.login_started = true;
        tokio::spawn(async move {
            let login_result = auth::login().await;
            if let Ok(mut should_fade) = should_fade_clone.lock() {
                *should_fade = true;
            }
            loop {
                if let Ok(opacity) = opacity_clone.lock() {
                    match login_result {
                        Ok(ref _client) => {
                            if let Ok(mut state) = state_clone.lock()
                                && *opacity <= 0.0
                            {
                                *state = UiState::Main;
                                // for instant repaint after
                                ctx.request_repaint();
                                break;
                            }
                        }
                        Err(ref e) => {
                            if let Ok(mut state) = state_clone.lock()
                                && *opacity <= 0.0
                            {
                                *state = UiState::Error(e.clone());
                                ctx.request_repaint();
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
