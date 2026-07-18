use super::{Arc, Client, LoginStage, Mutex, UiState, auth, egui, widgets};
use tokio::{sync::mpsc, task::JoinHandle};

const BUTTON_SIZE: egui::Vec2 = egui::vec2(280.0, 40.0);
const RADIUS: f32 = 10.0;

#[derive(Default)]
pub struct LoginScreen {
    show_validation_errors: bool,
    current_stage: LoginStage,
    target_stage: LoginStage,
    login_task: Option<LoginTask>,
    sso_link: Option<String>,
    login_error: Option<String>,
    homeserver: String,
    username: String,
    password: String,
    visible: bool,
}

struct LoginTask {
    login_handle: JoinHandle<()>,
    login_rx: mpsc::UnboundedReceiver<String>,
    error_rx: mpsc::UnboundedReceiver<String>,
    client_rx: mpsc::UnboundedReceiver<Client>,
}

struct WindowAnimation {
    height: f32,
    opacity: f32,
    content_opacity: f32,
    ready: bool,
}

impl LoginScreen {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut Arc<Mutex<UiState>>,
        client: &mut Option<Client>,
    ) {
        widgets::bottom_info_bar(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            widgets::draw_bg(ui);

            self.login_loading(ui);

            self.display_error(ui);

            self.login_window(ui, state, client);
        });
        self.visible = true;
    }

    fn display_error(&self, ui: &mut egui::Ui) {
        let screen_width = ui.ctx().viewport_rect().width();
        let toast_width = (screen_width * 0.6).clamp(250.0, 600.0);

        egui::Area::new("error_area".into())
            .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
            .show(ui, |ui| {
                let opacity = {
                    let fade = ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id(("login_screen", "error", "fade")),
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
                    .corner_radius(RADIUS)
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

    fn login_window(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut Arc<Mutex<UiState>>,
        client: &mut Option<Client>,
    ) {
        ui.scope(|ui| {
            let anim = self.window_animation(ui);

            ui.set_opacity(anim.opacity);

            egui::Area::new("login_area".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui, |ui| {
                    egui::Frame::window(&ui.global_style())
                        .multiply_with_opacity(anim.opacity)
                        .corner_radius(RADIUS + 5.0)
                        .show(ui, |ui| {
                            ui.set_min_width(300.0);
                            ui.set_height(anim.height);

                            ui.set_opacity(anim.content_opacity);

                            if anim.ready && anim.content_opacity > 0.0 {
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
    }

    fn homeserver(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            widgets::add_icon(ui, egui::Vec2::splat(64.0));
        });

        text_field(
            ui,
            "Homeserver",
            &mut self.homeserver,
            "https://",
            false,
            true,
            self.show_validation_errors,
        );

        ui.separator();

        ui.vertical_centered(|ui| {
            if ui
                .add(
                    egui::Button::new("Check")
                        .min_size(BUTTON_SIZE)
                        .corner_radius(RADIUS),
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
            widgets::add_icon(ui, egui::Vec2::splat(64.0));
        });

        let enabled = !self.login_started();

        text_field(
            ui,
            "Username",
            &mut self.username,
            "",
            false,
            enabled,
            self.show_validation_errors,
        );

        text_field(
            ui,
            "Password",
            &mut self.password,
            "",
            true,
            enabled,
            self.show_validation_errors,
        );

        ui.separator();

        ui.vertical_centered(|ui| {
            if ui
                .add_enabled(
                    enabled,
                    egui::Button::new("Login")
                        .min_size(BUTTON_SIZE)
                        .corner_radius(RADIUS),
                )
                .clicked()
            {
                if self.username.is_empty() || self.password.is_empty() {
                    self.show_validation_errors = true;
                } else {
                    self.finish_login(state);
                }
            }

            ui.label("or");

            if enabled {
                if ui
                    .add(
                        egui::Button::new("Login with Homeserver")
                            .min_size(BUTTON_SIZE)
                            .corner_radius(RADIUS),
                    )
                    .clicked()
                {
                    self.login_error = None;
                    self.start_login_homeserver();
                }
            } else if ui
                .add(
                    egui::Button::new("Cancel")
                        .min_size(BUTTON_SIZE)
                        .corner_radius(RADIUS),
                )
                .clicked()
            {
                // must be some at this point
                self.login_task.take().unwrap().login_handle.abort();
                self.sso_link = None;
            }

            self.login_recv(state, client);

            if ui
                .add_enabled(
                    enabled,
                    egui::Button::new("Back")
                        .min_size(BUTTON_SIZE)
                        .corner_radius(RADIUS),
                )
                .clicked()
            {
                self.show_validation_errors = false;
                self.target_stage = LoginStage::Homeserver;
            }
        });
    }

    fn start_login_homeserver(&mut self) {
        let (login_tx, login_rx) = mpsc::unbounded_channel();
        let (client_tx, client_rx) = mpsc::unbounded_channel();
        let (error_tx, error_rx) = mpsc::unbounded_channel();

        let homeserver_clone = self.homeserver.clone();

        let login_handle = tokio::spawn(async move {
            let login_result = auth::login_sso(&homeserver_clone, login_tx).await;

            match login_result {
                Ok(client) => {
                    client_tx.send(client).ok();
                }
                Err(ref e) => {
                    error_tx.send(e.to_string()).ok();
                }
            }
        });
        self.login_task = Some(LoginTask {
            login_handle,
            login_rx,
            error_rx,
            client_rx,
        });
    }

    fn login_recv(&mut self, state: &mut Arc<Mutex<UiState>>, client: &mut Option<Client>) {
        let mut error = None;
        let mut client_recv = None;
        let mut sso_link = None;

        if let Some(task) = self.login_task.as_mut() {
            error = task.error_rx.try_recv().ok();
            client_recv = task.client_rx.try_recv().ok();
            sso_link = task.login_rx.try_recv().ok();
        }

        if let Some(error_msg) = error {
            self.login_error = Some(error_msg);
            self.login_task = None;
        }

        if let Some(recv_client) = client_recv {
            *client = Some(recv_client);

            self.finish_login(state);
        }

        self.sso_link = sso_link.or(self.sso_link.take());
    }

    fn login_loading(&self, ui: &mut egui::Ui) {
        if !self.login_started() {
            return;
        }

        let size = 50.0;
        let screen_rect = ui.ctx().viewport_rect();

        let spinner_rect =
            egui::Rect::from_center_size(screen_rect.center(), egui::Vec2::splat(size));

        egui::Area::new("loading_spinner".into())
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.put(spinner_rect, egui::Spinner::new().size(size));
            });

        egui::Area::new("sso_text".into())
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, size * 1.2))
            .show(ui.ctx(), |ui| {
                let opacity = {
                    ui.ctx().animate_bool_with_time(
                        ui.make_persistent_id(("login_screen", "sso_text", "fade")),
                        self.sso_link.is_some(),
                        0.25,
                    )
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

    fn window_animation(&mut self, ui: &mut egui::Ui) -> WindowAnimation {
        let opacity = ui.ctx().animate_bool_with_time(
            ui.make_persistent_id(("login_screen", "fade")),
            self.visible,
            0.25,
        );

        let target_height = match self.current_stage {
            LoginStage::Homeserver => 200.0,
            LoginStage::Credentials => 390.0,
        };
        let (height, content_opacity) = if self.current_stage == self.target_stage {
            let height = ui.ctx().animate_value_with_time(
                ui.make_persistent_id(("login_screen", "content", "height")),
                target_height,
                0.1,
            );

            let is_height_ready = (height - target_height).abs() < 1.0;

            let fade_in_opacity = ui.ctx().animate_bool_with_time(
                ui.make_persistent_id(("login_screen", "content", "fade")),
                is_height_ready,
                0.15,
            );

            let content_opacity = if is_height_ready && opacity >= 1.0 {
                fade_in_opacity
            } else if opacity < 1.0 {
                opacity
            } else {
                0.0
            };

            (height, content_opacity)
        } else {
            let fade_out_opacity = ui.ctx().animate_bool_with_time(
                ui.make_persistent_id(("login_screen", "content", "fade")),
                false,
                0.15,
            );

            if fade_out_opacity <= 0.001 {
                self.current_stage = self.target_stage;
            }

            let content_opacity = fade_out_opacity;

            let height = ui.ctx().animate_value_with_time(
                ui.make_persistent_id(("login_screen", "content", "height")),
                target_height,
                0.1,
            );
            (height, content_opacity)
        };

        let ready = (height - target_height).abs() < 1.0;

        WindowAnimation {
            height,
            opacity,
            content_opacity,
            ready,
        }
    }

    fn finish_login(&mut self, state: &mut Arc<Mutex<UiState>>) {
        *self = Self::default();

        if let Ok(mut s) = state.lock() {
            *s = UiState::Main;
        }
    }

    fn login_started(&self) -> bool {
        self.login_task.is_some()
    }
}

fn text_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    prefix: &str,
    password: bool,
    enabled: bool,
    show_validation_errors: bool,
) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(label);
    });
    ui.vertical_centered(|ui| {
        ui.add_enabled(
            enabled,
            egui::TextEdit::singleline(value)
                .password(password)
                .prefix(prefix),
        );
    });

    validation_error(ui, show_validation_errors, value);
}

fn validation_error(ui: &mut egui::Ui, show_validation_errors: bool, value: &str) {
    if !show_validation_errors || !value.is_empty() {
        ui.add_space(9.0);
    }

    ui.horizontal(|ui| {
        if show_validation_errors && value.is_empty() {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("This field is required")
                    .color(egui::Color32::LIGHT_RED)
                    .small(),
            );
        }
    });
}
