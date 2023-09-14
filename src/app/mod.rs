#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::device::{self, DeviceHandle, Devices};

mod combobox;
mod layout;
mod theme;

pub fn run() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    let mut devices = Devices::default();
    devices.rescan();

    eframe::run_native(
        "Crustility",
        options,
        Box::new(|_cc| Box::new(Crustility::new(devices))),
    )
}

struct Crustility {
    device: Option<DeviceHandle>,
    error: Option<device::Error>,
    selected_key: Option<usize>,
    devices: Devices,
    theme: egui::Visuals,
}

impl Crustility {
    fn new(devices: Devices) -> Self {
        Self {
            device: None,
            error: None,
            selected_key: None,
            devices,
            theme: theme::horizon_dark(),
        }
    }

    /// gracefully consume error and log it
    /// you can use crustility.error to display it on the gui
    fn consume_error<T>(&mut self, result: Result<T, device::Error>) {
        if let Err(e) = result {
            log::error!("{e}");
            self.error = Some(e);
        }
    }
}

impl eframe::App for Crustility {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        ctx.set_visuals(self.theme.clone());
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.device.is_some() {
                self.menu_bar(ctx, ui);
                self.device_panel(ctx, ui);
            } else {
                self.default_panel(ctx, ui);
            }
        });
    }
}
