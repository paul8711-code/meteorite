use super::{Arc, Client, ErrorKind, LoginStage, Mutex, UiState, auth, egui, widgets};
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Default)]
pub struct LoginScreen {
    show_validation_errors: bool,
    current_stage: LoginStage,
    target_stage: LoginStage,
    login_started: bool,
    login_handle: Option<JoinHandle<()>>,
    login_tx: Option<mpsc::UnboundedSender<String>>,
    login_rx: Option<mpsc::UnboundedReceiver<String>>,
    error_rx: Option<mpsc::UnboundedReceiver<String>>,
    client_rx: Option<mpsc::UnboundedReceiver<Client>>,
    sso_link: Option<String>,
    login_error: Option<String>,
    homeserver: String,
    username: String,
    password: String,
    visible: bool,
}

impl LoginScreen {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut Arc<Mutex<UiState>>,
        client: &mut Option<Client>,
    ) {
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

            self.login_loading(ui);

            self.display_error(ui);

            ui.scope(|ui| {
                let opacity = ui.ctx().animate_bool_with_time(
                    ui.make_persistent_id("login_screen_fade_animation"),
                    self.visible,
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
                                            self.credentials(ui, state, client);
                                        }
                                    }
                                }
                            });
                    });
            });
        });
        if !self.visible {
            self.visible = true;
        }
    }

    fn display_error(&self, ui: &mut egui::Ui) {
        let screen_width = ui.ctx().viewport_rect().width();
        let toast_width = (screen_width * 0.6).clamp(250.0, 600.0);

        egui::Area::new("error_area".into())
            .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
            .show(ui, |ui| {
                let opacity = {
                    let fade = ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id("error_fade_animation"),
                        self.login_error.is_some(),
                        0.25,
                    );
                    if self.login_error.is_some() {
                        fade
                    } else {
                        0.0
                    }
                };

                ui.set_width(toast_width);

                ui.set_opacity(opacity);

                egui::Frame::window(&ui.global_style())
                    .corner_radius(10.0)
                    .fill(egui::Color32::from_rgb(255, 120, 120))
                    .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)))
                    .show(ui, |ui| {
                        if let Some(login_error) = &self.login_error {
                            ui.label(
                                egui::RichText::new(login_error)
                                    .color(egui::Color32::from_rgb(20, 20, 20)),
                            );
                        }
                    });
            });
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

    fn credentials(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut Arc<Mutex<UiState>>,
        client: &mut Option<Client>,
    ) {
        ui.vertical_centered(|ui| {
            widgets::add_icon(ui, egui::Vec2 { x: 64.0, y: 64.0 });
        });
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label("Username");
        });
        ui.vertical_centered(|ui| {
            ui.add_enabled(
                !self.login_started,
                egui::TextEdit::singleline(&mut self.username),
            );
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
            ui.add_enabled(
                !self.login_started,
                egui::TextEdit::singleline(&mut self.password).password(true),
            );
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
                .add_enabled(
                    !self.login_started,
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

            {
                if !self.login_started {
                    if ui
                        .add(
                            egui::Button::new("Login with Homeserver")
                                .min_size(egui::Vec2 { x: 280.0, y: 40.0 })
                                .corner_radius(10.0),
                        )
                        .clicked()
                    {
                        self.login_error = None;
                        self.start_login_homeserver(ui.ctx().clone(), state);
                    }
                } else if ui
                    .add(
                        egui::Button::new("Cancel")
                            .min_size(egui::Vec2 { x: 280.0, y: 40.0 })
                            .corner_radius(10.0),
                    )
                    .clicked()
                {
                    // must be some at this point
                    self.login_handle.as_mut().unwrap().abort();
                    self.login_started = false;
                    self.sso_link = None;
                }

                self.login_recv(state, client);
            }

            if ui
                .add_enabled(
                    !self.login_started,
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

    fn start_login_homeserver(&mut self, ctx: egui::Context, state: &mut Arc<Mutex<UiState>>) {
        let (login_tx, login_rx) = mpsc::unbounded_channel();
        self.login_tx = Some(login_tx.clone());
        self.login_rx = Some(login_rx);

        let (client_tx, client_rx) = mpsc::unbounded_channel();
        self.client_rx = Some(client_rx);

        let (error_tx, error_rx) = mpsc::unbounded_channel();
        self.error_rx = Some(error_rx);

        let state_clone = Arc::clone(state);
        let homeserver_clone = self.homeserver.clone();

        self.login_started = true;
        let login_handle = tokio::spawn(async move {
            let login_result = auth::login_sso(&homeserver_clone, login_tx).await;
            loop {
                match login_result {
                    Ok(client) => {
                        client_tx.send(client).ok();
                        break;
                    }
                    Err(ref e) => {
                        if let Ok(mut state) = state_clone.lock() {
                            match e {
                                auth::LoginError::NoAccountActive => {
                                    *state = UiState::Error {
                                        kind: ErrorKind::NoAccountActive,
                                        message: e.to_string(),
                                    }
                                }
                                auth::LoginError::Other(_) => {
                                    error_tx.send(e.to_string()).ok();
                                }
                            }
                            ctx.request_repaint();
                            break;
                        }
                    }
                }
            }
        });
        self.login_handle = Some(login_handle);
    }

    fn login_recv(&mut self, state: &mut Arc<Mutex<UiState>>, client: &mut Option<Client>) {
        if let Some(error_rx) = self.error_rx.as_mut()
            && let Ok(error_msg) = error_rx.try_recv()
        {
            self.login_error = Some(error_msg);
            self.login_started = false;
        }

        if let Some(client_rx) = self.client_rx.as_mut()
            && let Ok(recv_client) = client_rx.try_recv()
        {
            *client = Some(recv_client);

            // clear login screen struct "for better security"
            *self = LoginScreen::default();
            // should always return Ok
            if let Ok(mut state) = state.lock() {
                *state = UiState::Main;
            }
        }

        if let Some(login_rx) = self.login_rx.as_mut()
            && let Ok(sso_link) = login_rx.try_recv()
        {
            self.sso_link = Some(sso_link);
        }
    }

    fn login_loading(&self, ui: &mut egui::Ui) {
        if !self.login_started {
            return;
        }

        let size = 50.0;
        let screen_rect = ui.ctx().viewport_rect();

        let spinner_rect =
            egui::Rect::from_center_size(screen_rect.center(), egui::Vec2::splat(size));

        egui::Area::new(egui::Id::new("loading_spinner"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.put(spinner_rect, egui::Spinner::new().size(size));
            });

        egui::Area::new(egui::Id::new("sso_text"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, size * 1.2))
            .show(ui.ctx(), |ui| {
                let opacity = {
                    let fade = ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id("sso_text_fade_animation"),
                        self.sso_link.is_some(),
                        0.25,
                    );
                    if self.sso_link.is_some() { fade } else { 0.0 }
                };

                ui.set_opacity(opacity);

                egui::Frame::window(&ui.global_style())
                    .corner_radius(10.0)
                    .fill(egui::Color32::from_rgb(27, 27, 27))
                    .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(60, 60, 60)))
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            if let Some(text) = &self.sso_link {
                                ui.label(text);
                            }
                        });
                    });
            });
    }
}
