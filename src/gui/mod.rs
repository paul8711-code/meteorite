use crate::APP_NAME;
use crate::core::{auth, utils};
use eframe::egui;
use native_dialog::MessageLevel;
use std::sync::{Arc, Mutex};

mod screens;
mod widgets;

use screens::{error, loading, login, main};

#[derive(PartialEq, Clone, Copy, Default)]
enum LoginStage {
    #[default]
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
    login_screen: login::LoginScreen,
    loading_screen: loading::LoadingScreen,
    main_screen: main::MainScreen,
    error_screen: error::ErrorScreen,
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
            login_screen: login::LoginScreen::default(),
            loading_screen: loading::LoadingScreen::default(),
            main_screen: main::MainScreen,
            error_screen: error::ErrorScreen::default(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let state = {
            if let Ok(state) = self.current_state.lock() {
                state.clone()
            } else {
                UiState::Loading // fallback
            }
        };

        match state {
            UiState::Loading => self.loading_screen.show(ui, &mut self.current_state),
            UiState::Error(err) => self.error_screen.show(ui, &mut self.current_state, err),
            UiState::Main => self.main_screen.show(ui),
            UiState::Login => self.login_screen.show(ui, &mut self.current_state),
        }
    }
}
