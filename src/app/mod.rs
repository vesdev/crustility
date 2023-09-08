#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Duration;

use egui::{pos2, vec2};

mod key_panel;

pub fn run() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Cristility",
        options,
        Box::new(|_cc| Box::<Crustility>::default()),
    )
}

struct Crustility {
    time: std::time::Instant,
    rapid_trigger: bool,
}

impl Default for Crustility {
    fn default() -> Self {
        Self {
            time: std::time::Instant::now(),
            rapid_trigger: false,
        }
    }
}

impl eframe::App for Crustility {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);
            egui::TopBottomPanel::top("Crustility").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    if ui.button("File").clicked() {
                        println!("forsen");
                    }
                });
            });
            ctx.set_debug_on_hover(true);

            self.keypanel(ctx, ui);

            egui::panel::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Global Configuration");
                ui.separator();

                ui.add_space(15.);
                ui.heading("Rapid Trigger:");
                ui.checkbox(&mut self.rapid_trigger, "Enabled");
                ui.checkbox(&mut self.rapid_trigger, "Continuos Rapid Trigger");
                ui.add(egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Up Sensitivity"));
                ui.add(egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Down Sensitivity"));

                ui.add_space(15.);
                ui.heading("General:");
                ui.add(egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Upper Hysterisis"));
                ui.add(egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Lower Hysterisis"));
            });
        });
    }
}
