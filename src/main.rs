#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Duration;

use egui::{pos2, vec2, Rect, Style};

fn main() -> Result<(), eframe::Error> {
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

            let frame = egui::Frame::central_panel(ui.style());

            egui::panel::SidePanel::right("Key Panel")
                .frame(frame)
                // .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Keys");
                    ui.separator();

                    let key_count = 3;
                    let key_width = 50.;
                    let current_rect = ui.max_rect();
                    ui.horizontal(|ui| {
                        let cursor = ui.cursor().min;
                        for i in 0..key_count {
                            let key_rect = egui::Rect::from_two_pos(
                                egui::pos2(current_rect.min.x + i as f32 * key_width, cursor.y),
                                egui::pos2(
                                    current_rect.min.x + i as f32 * key_width + key_width,
                                    current_rect.max.y,
                                ),
                            );

                            ui.allocate_ui_at_rect(key_rect, |ui| {
                                ui.vertical_centered(|ui| {
                                    // ui.heading(format!("{}", i));

                                    let key_visualizer_rect = egui::Rect::from_two_pos(
                                        pos2(key_rect.min.x, ui.cursor().min.y),
                                        key_rect.max,
                                    );

                                    let key_range = 100.;
                                    let key_position = ((self.time.elapsed().as_secs_f32() * 2.
                                        + (i as f32 + 1.))
                                        .sin()
                                        * key_range
                                        + key_range)
                                        / 2.;
                                    ctx.request_repaint_after(Duration::from_millis(20));
                                    //simulate key press

                                    ui.painter().rect(
                                        egui::Rect::from_two_pos(
                                            key_visualizer_rect.min
                                                + vec2(
                                                    0.,
                                                    key_position / key_range
                                                        * key_visualizer_rect.height(),
                                                ),
                                            key_visualizer_rect.max,
                                        ),
                                        egui::Rounding::none(),
                                        egui::Color32::WHITE,
                                        egui::Stroke::NONE,
                                    );

                                    ui.painter().rect(
                                        key_visualizer_rect,
                                        egui::Rounding::none(),
                                        egui::Color32::TRANSPARENT,
                                        egui::Stroke::new(2., egui::Color32::WHITE),
                                    );
                                })
                            });
                        }
                    });
                });

            egui::panel::CentralPanel::default()
                .frame(frame)
                .show(ctx, |ui| {
                    ui.heading("Global Configuration");
                    ui.separator();

                    ui.add_space(15.);
                    ui.heading("Rapid Trigger:");
                    ui.checkbox(&mut self.rapid_trigger, "Enabled");
                    ui.checkbox(&mut self.rapid_trigger, "Continuos Rapid Trigger");
                    ui.add(egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Up Sensitivity"));
                    ui.add(
                        egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Down Sensitivity"),
                    );

                    ui.add_space(15.);
                    ui.heading("General:");
                    ui.add(
                        egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Upper Hysterisis"),
                    );
                    ui.add(
                        egui::widgets::Slider::new(&mut 0., 0.0..=100.0).text("Lower Hysterisis"),
                    );
                });
        });
    }
}
