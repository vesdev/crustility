#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::device::{self, DeviceHandle, Devices};

mod combobox;
mod layout;

pub fn run() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    let mut devices = Devices::default();
    devices.refresh();

    eframe::run_native(
        "Crustility",
        options,
        Box::new(|_cc| Box::new(Crustility::new(devices))),
    )
}

struct Crustility {
    device: Option<DeviceHandle>,
    time: std::time::Instant,
    error: Option<device::Error>,
    selected_key: Option<usize>,
    devices: Devices,
}

impl Crustility {
    fn new(devices: Devices) -> Self {
        Self {
            time: std::time::Instant::now(),
            device: None,
            error: None,
            selected_key: None,
            devices,
        }
    }

    /// gracefully consume error and log it
    /// you can use crustility.error to display it on the gui
    fn consume_error<T>(&mut self, result: Result<T, device::Error>) {
        if let Err(e) = &result {
            self.error = Some(e.clone());
            log::error!("{e}");
        }
    }
}

impl eframe::App for Crustility {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);
            if self.device.is_some() {
                self.menu_bar(ctx, ui);
                self.device_panel(ctx, ui);
            } else {
                self.default_panel(ctx, ui);
            }
        });
    }
}
