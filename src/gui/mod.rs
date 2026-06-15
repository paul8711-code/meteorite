use eframe::egui;
use native_dialog::{DialogBuilder, MessageLevel};

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

#[derive(Default)]
struct App {
    authenticated: bool,
    homeserver: String,
    homeserver_set: bool, // treat this like a question (homeserver set?)
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
            (TextStyle::Body, FontId::new(16.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(12.0, Proportional)),
            (TextStyle::Small, FontId::new(8.0, Proportional)),
        ]
        .into();
        cc.egui_ctx
            .all_styles_mut(move |style| style.text_styles = text_styles.clone());
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            authenticated: false,
            homeserver: String::new(),
            homeserver_set: false,
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

            let target_height = if !self.homeserver_set { 100.0 } else { 200.0 };
            let animated_height = ui.ctx().animate_value_with_time(
                ui.make_persistent_id("login_height_animation"),
                target_height,
                0.1,
            );
            let animation_finished = (animated_height - target_height).abs() < 1.0;

            egui::Area::new("login_area".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui, |ui| {
                    egui::Frame::window(&ui.global_style())
                        .corner_radius(15.0)
                        .show(ui, |ui| {
                            ui.set_min_width(300.0);
                            ui.set_min_height(animated_height);

                            if !self.homeserver_set {
                                ui.scope(|ui| {
                                    ui.label("Homeserver");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.homeserver)
                                            .prefix("https://"),
                                    );

                                    ui.add_space(10.0);

                                    if ui.button("Check").clicked() {
                                        self.homeserver_set = true;
                                    }
                                });
                            } else {
                                if animation_finished {
                                    ui.scope(|ui| {
                                        ui.label("Username");
                                        ui.text_edit_singleline(&mut self.username);

                                        ui.label("Password");
                                        ui.add(
                                            egui::TextEdit::singleline(&mut self.password)
                                                .password(true),
                                        );

                                        ui.add_space(10.0);

                                        if ui.button("Back").clicked() {
                                            self.homeserver_set = false;
                                        }

                                        if ui.button("Login").clicked() {
                                            println!("Login pressed");
                                        }

                                        ui.label("or");

                                        if ui.button("Login with Homeserver").clicked() {
                                            println!("Login with hs");
                                        }
                                    });
                                }
                            }
                        });
                });
        }
    }
}
