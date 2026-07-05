use super::egui;

#[derive(Default)]
pub struct MainScreen;

impl MainScreen {
    pub fn show(&self, ui: &mut egui::Ui) {
        egui::Panel::left("room_list_panel")
            .resizable(false)
            .exact_size(75.0)
            .show_inside(ui, |ui| {
                ui.add_space(5.0);
                // all icons have now been normed to 50 px
                let home_button = ui.add(
                    egui::widgets::Button::image(
                        egui::Image::new(egui::include_image!("../../../assets/home.png"))
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
}
