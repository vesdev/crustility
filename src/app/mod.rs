#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{
    config::Config,
    device::{self, Device},
};

mod layout;

pub fn run() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Crustility",
        options,
        Box::new(|_cc| Box::<Crustility>::default()),
    )
}

struct Crustility {
    config: Config,
    device: Option<Device>,
    available_devices: Option<Vec<Device>>,
    time: std::time::Instant,
    error: Option<device::Error>,
    selected_key: Option<usize>,
}

impl Default for Crustility {
    fn default() -> Self {
        Self {
            time: std::time::Instant::now(),
            config: Config::default(),
            device: None,
            available_devices: None,
            error: None,
            selected_key: None,
        }
    }
}

impl Crustility {
    fn consume_result<'a, T>(
        &mut self,
        result: &'a Result<T, device::Error>,
    ) -> &'a Result<T, device::Error> {
        if let Err(e) = &result {
            self.error = Some(e.clone());
        }
        result
    }
}

impl eframe::App for Crustility {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);

            self.menu_bar(ctx, ui);
            self.status_bar(ctx, ui);
            if self.device.is_some() {
                self.key_panel(ctx, ui);
                // self.main_panel(ctx, ui);
            } else {
                ui.centered_and_justified(|ui| ui.heading("Select a Device..."));
            }
        });
    }
}
