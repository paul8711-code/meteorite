use crate::APP_NAME;
use crate::core::{auth, utils};
use eframe::egui;
use native_dialog::MessageLevel;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
enum LoginStage {
    Homeserver,
    Credentials,
}

#[derive(PartialEq, Clone)]
enum UiState {
    Loading,
    Error(auth::LoginError),
    Login,
    Main,
}

static ICON: &[u8] = include_bytes!("../../assets/icon/icon.png");

pub fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::viewport::ViewportBuilder {
            app_id: Some(APP_NAME.to_owned()),
            icon: Some(Arc::new(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../../assets/icon/icon-rounded.png"
                ))
                .unwrap(),
            )),
            ..Default::default()
        },
        ..Default::default()
    };
    match eframe::run_native(
        "meteorite",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    ) {
        Ok(_) => {}
        Err(e) => {
            utils::show_dialog_window(
                "Launch Error",
                format!(
                    "The application failed to set up a graphics context.\n\nDetails: {}",
                    e
                ),
                MessageLevel::Error,
            );
        }
    }
}

struct App {
    current_state: Arc<Mutex<UiState>>,
    login_started: bool,
    login_screen: LoginScreen,
}

// this struct only contains values relevant on the login screen
struct LoginScreen {
    show_validation_errors: bool,
    current_stage: LoginStage,
    target_stage: LoginStage,
    homeserver: String,
    username: String,
    password: String,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        use FontFamily::{Monospace, Proportional};
        use egui::{FontFamily, FontId, TextStyle};
        use std::collections::BTreeMap;

        let text_styles: BTreeMap<TextStyle, FontId> = [
            (TextStyle::Heading, FontId::new(25.0, Proportional)),
            (TextStyle::Body, FontId::new(20.0, Proportional)),
            (TextStyle::Monospace, FontId::new(20.0, Monospace)),
            (TextStyle::Button, FontId::new(20.0, Proportional)),
            (TextStyle::Small, FontId::new(15.0, Proportional)),
        ]
        .into();
        cc.egui_ctx
            .all_styles_mut(move |style| style.text_styles = text_styles.clone());
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            current_state: Arc::new(Mutex::new(UiState::Loading)),
            login_started: false,
            login_screen: LoginScreen {
                show_validation_errors: false,
                current_stage: LoginStage::Homeserver,
                target_stage: LoginStage::Homeserver,
                homeserver: String::new(),
                username: String::new(),
                password: String::new(),
            },
        }
    }
}

// TODO:
// - make sub functions (to fix massive indentation)
// - move stuff to other files
// - make bg on loading screen & error screen
// - possibly animate after loading screen

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let current_state_value = {
            if let Ok(state) = self.current_state.lock() {
                state.clone()
            } else {
                UiState::Loading // fallback
            }
        };

        match current_state_value {
            UiState::Loading => {
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
                        if !self.login_started {
                            self.login_started = true;
                            let state_clone = std::sync::Arc::clone(&self.current_state);
                            let ctx = ui.ctx().clone();
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
                    });
            }
            UiState::Error(err) => match err {
                auth::LoginError::NoAccountActive => {
                    if let Ok(mut state) = self.current_state.lock() {
                        *state = UiState::Login;
                    }
                }
                _ => {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new().fill(egui::Color32::from_rgb(36, 36, 36)))
                        .show_inside(ui, |ui| {
                            egui::Area::new("error_area".into())
                                .anchor(egui::Align2::CENTER_TOP, [0.0, 50.0])
                                .show(ui, |ui| {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            egui::Frame::window(&ui.global_style())
                                                .corner_radius(10.0)
                                                .fill(egui::Color32::from_rgb(255, 120, 120))
                                                .stroke(egui::Stroke::new(
                                                    3.0,
                                                    egui::Color32::from_rgb(255, 0, 0),
                                                ))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!("{}", err))
                                                            .color(egui::Color32::from_rgb(
                                                                20, 20, 20,
                                                            )),
                                                    );
                                                });

                                            ui.add_space(50.0);

                                            ui.add(
                                                egui::Image::from_bytes("bytes://icon.png", ICON)
                                                    .corner_radius(15.0)
                                                    .fit_to_exact_size(egui::Vec2 {
                                                        x: 256.0,
                                                        y: 256.0,
                                                    }),
                                            );
                                        },
                                    );
                                });
                        });
                }
            },
            UiState::Main => {
                egui::Panel::left("room_list_panel")
                    .resizable(false)
                    .exact_size(75.0)
                    .show_inside(ui, |ui| {
                        ui.add_space(5.0);
                        // all icons have now been normed to 50 px
                        let home_button = ui.add(
                            egui::widgets::Button::image(
                                egui::Image::new(egui::include_image!("../../assets/home.png"))
                                    .fit_to_exact_size(egui::Vec2 { x: 50.0, y: 50.0 }),
                            )
                            .corner_radius(20.0), // animate to 15.0
                        );
                        if home_button.clicked() {
                            println!("home");
                        }

                        ui.add(egui::Separator::default().horizontal());
                    });
                egui::Panel::right("account_panel")
                    .resizable(false)
                    .exact_size(350.0)
                    .show_inside(ui, |ui| {
                        if ui.button("settings").clicked() {
                            println!("settings");
                        }
                    });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("meteorite");

                    if ui.button("test").clicked() {
                        println!("clicked button");
                    }
                });
            }
            UiState::Login => {
                egui::Panel::bottom("login_bottom_panel")
                    .resizable(false)
                    .exact_size(50.0)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(env!("CARGO_PKG_VERSION"));
                        });
                    });

                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(32, 32, 32)))
                    .show_inside(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.painter().add(egui::Shape::gradient_rect(
                                ui.ctx().viewport_rect(),
                                egui::Direction::TopDown,
                                [
                                    egui::Color32::from_rgb(0, 48, 32),
                                    egui::Color32::from_rgb(1, 51, 0),
                                ],
                            ));
                        });
                    });

                let target_height = match self.login_screen.current_stage {
                    LoginStage::Homeserver => 135.0,
                    LoginStage::Credentials => 325.0,
                };
                let (login_height_animation, render_opacity) =
                    if self.login_screen.current_stage != self.login_screen.target_stage {
                        let fade_out_opacity = ui.ctx().animate_bool_with_time(
                            ui.make_persistent_id("login_fade_animation"),
                            false,
                            0.15,
                        );

                        if fade_out_opacity <= 0.001 {
                            self.login_screen.current_stage = self.login_screen.target_stage;
                        }

                        let render_opacity = fade_out_opacity;

                        let login_height_animation = ui.ctx().animate_value_with_time(
                            ui.make_persistent_id("login_height_animation"),
                            target_height,
                            0.1,
                        );
                        (login_height_animation, render_opacity)
                    } else {
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

                        let render_opacity = if is_height_ready {
                            fade_in_opacity
                        } else {
                            0.0
                        };

                        (login_height_animation, render_opacity)
                    };

                egui::Area::new("login_area".into())
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui, |ui| {
                        egui::Frame::window(&ui.global_style())
                            .corner_radius(15.0)
                            .show(ui, |ui| {
                                ui.set_min_width(300.0);
                                ui.set_height(login_height_animation);

                                ui.set_opacity(render_opacity);

                                if login_height_animation == target_height && render_opacity > 0.0 {
                                    match self.login_screen.current_stage {
                                        LoginStage::Homeserver => {
                                            ui.horizontal(|ui| {
                                                ui.add_space(10.0);
                                                ui.label("Homeserver");
                                            });
                                            ui.vertical_centered(|ui| {
                                                ui.add(
                                                    egui::TextEdit::singleline(
                                                        &mut self.login_screen.homeserver,
                                                    )
                                                    .prefix("https://"),
                                                );
                                            });
                                            ui.horizontal(|ui| {
                                                if self.login_screen.show_validation_errors
                                                    && self.login_screen.homeserver.is_empty()
                                                {
                                                    ui.add_space(10.0);
                                                    ui.label(
                                                        egui::RichText::new(
                                                            "This field is required",
                                                        )
                                                        .color(egui::Color32::LIGHT_RED)
                                                        .small(),
                                                    );
                                                }
                                            });

                                            if !self.login_screen.homeserver.is_empty()
                                                || !self.login_screen.show_validation_errors
                                            {
                                                ui.add_space(9.0);
                                            }
                                            ui.separator();

                                            ui.vertical_centered(|ui| {
                                                if ui
                                                    .add(
                                                        egui::Button::new("Check")
                                                            .min_size(egui::Vec2 {
                                                                x: 280.0,
                                                                y: 40.0,
                                                            })
                                                            .corner_radius(10.0),
                                                    )
                                                    .clicked()
                                                {
                                                    if self.login_screen.homeserver.is_empty() {
                                                        self.login_screen.show_validation_errors =
                                                            true;
                                                    } else {
                                                        self.login_screen.show_validation_errors =
                                                            false;
                                                        println!(
                                                            "{}",
                                                            self.login_screen.homeserver
                                                        );
                                                        self.login_screen.target_stage =
                                                            LoginStage::Credentials;
                                                    }
                                                }
                                            });
                                        }
                                        LoginStage::Credentials => {
                                            ui.horizontal(|ui| {
                                                ui.add_space(10.0);
                                                ui.label("Username");
                                            });
                                            ui.vertical_centered(|ui| {
                                                ui.text_edit_singleline(
                                                    &mut self.login_screen.username,
                                                );
                                            });

                                            if !self.login_screen.username.is_empty()
                                                || !self.login_screen.show_validation_errors
                                            {
                                                ui.add_space(9.0);
                                            }

                                            ui.horizontal(|ui| {
                                                if self.login_screen.show_validation_errors
                                                    && self.login_screen.username.is_empty()
                                                {
                                                    ui.add_space(10.0);
                                                    ui.label(
                                                        egui::RichText::new(
                                                            "This field is required",
                                                        )
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
                                                ui.add(
                                                    egui::TextEdit::singleline(
                                                        &mut self.login_screen.password,
                                                    )
                                                    .password(true),
                                                );
                                            });

                                            if !self.login_screen.password.is_empty()
                                                || !self.login_screen.show_validation_errors
                                            {
                                                ui.add_space(9.0);
                                            }

                                            ui.horizontal(|ui| {
                                                if self.login_screen.show_validation_errors
                                                    && self.login_screen.password.is_empty()
                                                {
                                                    ui.add_space(10.0);
                                                    ui.label(
                                                        egui::RichText::new(
                                                            "This field is required",
                                                        )
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
                                                            .min_size(egui::Vec2 {
                                                                x: 280.0,
                                                                y: 40.0,
                                                            })
                                                            .corner_radius(10.0),
                                                    )
                                                    .clicked()
                                                {
                                                    if self.login_screen.username.is_empty()
                                                        || self.login_screen.password.is_empty()
                                                    {
                                                        self.login_screen.show_validation_errors =
                                                            true;
                                                    } else {
                                                        // clear login screen struct "for better security"
                                                        self.login_screen = LoginScreen {
                                                            show_validation_errors: false,
                                                            current_stage: LoginStage::Homeserver,
                                                            target_stage: LoginStage::Homeserver,
                                                            homeserver: String::new(),
                                                            username: String::new(),
                                                            password: String::new(),
                                                        };
                                                        // should always return Ok
                                                        if let Ok(mut state) =
                                                            self.current_state.lock()
                                                        {
                                                            *state = UiState::Main;
                                                        }
                                                    }
                                                }

                                                ui.label("or");

                                                if ui
                                                    .add(
                                                        egui::Button::new("Login with Homeserver")
                                                            .min_size(egui::Vec2 {
                                                                x: 280.0,
                                                                y: 40.0,
                                                            })
                                                            .corner_radius(10.0),
                                                    )
                                                    .clicked()
                                                {
                                                    println!("login with hs");
                                                }

                                                if ui
                                                    .add(
                                                        egui::Button::new("Back")
                                                            .min_size(egui::Vec2 {
                                                                x: 280.0,
                                                                y: 40.0,
                                                            })
                                                            .corner_radius(10.0),
                                                    )
                                                    .clicked()
                                                {
                                                    self.login_screen.show_validation_errors =
                                                        false;
                                                    self.login_screen.target_stage =
                                                        LoginStage::Homeserver;
                                                }
                                            });
                                        }
                                    }
                                }
                            });
                    });
            }
        }
    }
}
