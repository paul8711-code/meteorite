use super::{Arc, LoginStage, Mutex, UiState, egui, widgets};

#[derive(Default)]
pub struct LoginScreen {
    show_validation_errors: bool,
    current_stage: LoginStage,
    target_stage: LoginStage,
    homeserver: String,
    username: String,
    password: String,
    opacity: bool,
}

impl LoginScreen {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut Arc<Mutex<UiState>>) {
        egui::Panel::bottom("login_bottom_panel")
            .resizable(false)
            .exact_size(50.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(env!("CARGO_PKG_VERSION"));
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
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
        });

        ui.scope(|ui| {
            let opacity = ui.ctx().animate_bool_with_time(
                ui.make_persistent_id("login_screen_fade_animation"),
                self.opacity,
                0.25,
            );

            ui.set_opacity(opacity);

            let target_height = match self.current_stage {
                LoginStage::Homeserver => 200.0,
                LoginStage::Credentials => 390.0,
            };
            let (login_height_animation, render_opacity) =
                if self.current_stage == self.target_stage {
                    let login_height_animation = ui.ctx().animate_value_with_time(
                        ui.make_persistent_id("login_height_animation"),
                        target_height,
                        0.1,
                    );

                    let is_height_ready = (login_height_animation - target_height).abs() < 1.0;

                    let fade_in_opacity = ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id("login_fade_animation"),
                        is_height_ready,
                        0.15,
                    );

                    let render_opacity = if is_height_ready && opacity >= 1.0 {
                        fade_in_opacity
                    } else if opacity < 1.0 {
                        opacity
                    } else {
                        0.0
                    };

                    (login_height_animation, render_opacity)
                } else {
                    let fade_out_opacity = ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id("login_fade_animation"),
                        false,
                        0.15,
                    );

                    if fade_out_opacity <= 0.001 {
                        self.current_stage = self.target_stage;
                    }

                    let render_opacity = fade_out_opacity;

                    let login_height_animation = ui.ctx().animate_value_with_time(
                        ui.make_persistent_id("login_height_animation"),
                        target_height,
                        0.1,
                    );
                    (login_height_animation, render_opacity)
                };

            egui::Area::new("login_area".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui, |ui| {
                    egui::Frame::window(&ui.global_style())
                        .multiply_with_opacity(opacity)
                        .corner_radius(15.0)
                        .show(ui, |ui| {
                            ui.set_min_width(300.0);
                            ui.set_height(login_height_animation);

                            ui.set_opacity(render_opacity);

                            if login_height_animation >= target_height && render_opacity > 0.0 {
                                match self.current_stage {
                                    LoginStage::Homeserver => {
                                        self.homeserver(ui);
                                    }
                                    LoginStage::Credentials => {
                                        self.credentials(ui, state);
                                    }
                                }
                            }
                        });
                });
        });
        if !self.opacity {
            self.opacity = true;
        }
    }

    fn homeserver(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            widgets::add_icon(ui, egui::Vec2 { x: 64.0, y: 64.0 });
        });
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Homeserver");
        });
        ui.vertical_centered(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.homeserver).prefix("https://"));
        });
        ui.horizontal(|ui| {
            if self.show_validation_errors && self.homeserver.is_empty() {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("This field is required")
                        .color(egui::Color32::LIGHT_RED)
                        .small(),
                );
            }
        });

        if !self.homeserver.is_empty() || !self.show_validation_errors {
            ui.add_space(9.0);
        }
        ui.separator();

        ui.vertical_centered(|ui| {
            if ui
                .add(
                    egui::Button::new("Check")
                        .min_size(egui::Vec2 { x: 280.0, y: 40.0 })
                        .corner_radius(10.0),
                )
                .clicked()
            {
                if self.homeserver.is_empty() {
                    self.show_validation_errors = true;
                } else {
                    self.show_validation_errors = false;
                    self.target_stage = LoginStage::Credentials;
                }
            }
        });
    }

    fn credentials(&mut self, ui: &mut egui::Ui, state: &mut Arc<Mutex<UiState>>) {
        ui.vertical_centered(|ui| {
            widgets::add_icon(ui, egui::Vec2 { x: 64.0, y: 64.0 });
        });
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Username");
        });
        ui.vertical_centered(|ui| {
            ui.text_edit_singleline(&mut self.username);
        });

        if !self.username.is_empty() || !self.show_validation_errors {
            ui.add_space(9.0);
        }

        ui.horizontal(|ui| {
            if self.show_validation_errors && self.username.is_empty() {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("This field is required")
                        .color(egui::Color32::LIGHT_RED)
                        .small(),
                );
            }
        });

        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Password");
        });
        ui.vertical_centered(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
        });

        if !self.password.is_empty() || !self.show_validation_errors {
            ui.add_space(9.0);
        }

        ui.horizontal(|ui| {
            if self.show_validation_errors && self.password.is_empty() {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("This field is required")
                        .color(egui::Color32::LIGHT_RED)
                        .small(),
                );
            }
        });

        ui.separator();

        ui.vertical_centered(|ui| {
            if ui
                .add(
                    egui::Button::new("Login")
                        .min_size(egui::Vec2 { x: 280.0, y: 40.0 })
                        .corner_radius(10.0),
                )
                .clicked()
            {
                if self.username.is_empty() || self.password.is_empty() {
                    self.show_validation_errors = true;
                } else {
                    // clear login screen struct "for better security"
                    *self = LoginScreen::default();
                    // should always return Ok
                    if let Ok(mut state) = state.lock() {
                        *state = UiState::Main;
                    }
                }
            }

            ui.label("or");

            if ui
                .add(
                    egui::Button::new("Login with Homeserver")
                        .min_size(egui::Vec2 { x: 280.0, y: 40.0 })
                        .corner_radius(10.0),
                )
                .clicked()
            {
                println!("login with hs");
            }

            if ui
                .add(
                    egui::Button::new("Back")
                        .min_size(egui::Vec2 { x: 280.0, y: 40.0 })
                        .corner_radius(10.0),
                )
                .clicked()
            {
                self.show_validation_errors = false;
                self.target_stage = LoginStage::Homeserver;
            }
        });
    }
}
