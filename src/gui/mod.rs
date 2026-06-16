use eframe::egui;
use native_dialog::{DialogBuilder, MessageLevel};

#[derive(PartialEq, Clone, Copy)]
enum LoginStage {
    Homeserver,
    Credentials,
}

pub fn main() {
    let native_options = eframe::NativeOptions::default();
    match eframe::run_native(
        "meteorite",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    ) {
        Ok(_) => {}
        Err(e) => {
            DialogBuilder::message()
                .set_title("Launch Error")
                .set_text(format!(
                    "The application failed to set up a graphics context.\n\nDetails: {}",
                    e
                ))
                .set_level(MessageLevel::Error)
                .alert()
                .show()
                .unwrap();
        }
    }
}

struct App {
    authenticated: bool,
    show_validation_errors: bool,
    homeserver: String,
    current_stage: LoginStage,
    target_stage: LoginStage,
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
            authenticated: false,
            show_validation_errors: false,
            homeserver: String::new(),
            current_stage: LoginStage::Homeserver,
            target_stage: LoginStage::Homeserver,
            username: String::new(),
            password: String::new(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.authenticated {
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
        } else {
            egui::Panel::bottom("login_bottom_panel")
                .resizable(false)
                .exact_size(50.0)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(env!("CARGO_PKG_VERSION"));
                    });
                });

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(egui::Color32::from_hex("#202020").unwrap()))
                .show_inside(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.painter().add(egui::Shape::gradient_rect(
                            ui.ctx().viewport_rect(),
                            egui::Direction::TopDown,
                            [
                                egui::Color32::from_hex("#003020").unwrap(),
                                egui::Color32::from_hex("#013300").unwrap(),
                            ],
                        ));
                    });
                });

            let target_height = match self.current_stage {
                LoginStage::Homeserver => 135.0,
                LoginStage::Credentials => 325.0,
            };
            let (login_height_animation, render_opacity) =
                if self.current_stage != self.target_stage {
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
                                match self.current_stage {
                                    LoginStage::Homeserver => {
                                        ui.horizontal(|ui| {
                                            ui.add_space(10.0);
                                            ui.label("Homeserver");
                                        });
                                        ui.vertical_centered(|ui| {
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.homeserver)
                                                    .prefix("https://"),
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            if self.show_validation_errors
                                                && self.homeserver.is_empty()
                                            {
                                                ui.add_space(10.0);
                                                ui.label(
                                                    egui::RichText::new("This field is required")
                                                        .color(egui::Color32::LIGHT_RED)
                                                        .small(),
                                                );
                                            }
                                        });

                                        if !self.homeserver.is_empty()
                                            || !self.show_validation_errors
                                        {
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
                                                    println!("{}", self.homeserver);
                                                    self.target_stage = LoginStage::Credentials;
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
                                            ui.text_edit_singleline(&mut self.username);
                                        });

                                        if !self.username.is_empty() || !self.show_validation_errors
                                        {
                                            ui.add_space(9.0);
                                        }

                                        ui.horizontal(|ui| {
                                            if self.show_validation_errors
                                                && self.username.is_empty()
                                            {
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
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.password)
                                                    .password(true),
                                            );
                                        });

                                        if !self.password.is_empty() || !self.show_validation_errors
                                        {
                                            ui.add_space(9.0);
                                        }

                                        ui.horizontal(|ui| {
                                            if self.show_validation_errors
                                                && self.password.is_empty()
                                            {
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
                                                if self.username.is_empty()
                                                    || self.password.is_empty()
                                                {
                                                    self.show_validation_errors = true;
                                                } else {
                                                    self.show_validation_errors = false;
                                                    self.authenticated = true;
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
                            }
                        });
                });
        }
    }
}
