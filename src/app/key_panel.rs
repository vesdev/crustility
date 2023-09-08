use std::time::Duration;

use egui::{pos2, vec2};

use super::Crustility;

impl Crustility {
    pub fn keypanel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let frame = egui::Frame::central_panel(ui.style());
        egui::panel::SidePanel::right("Key Panel")
            .frame(frame)
            .show(ctx, |ui| {
                ui.heading("Keys");
                ui.separator();

                self.key_visualizer(ctx, ui);
            });
    }

    fn key_visualizer(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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
                        ui.heading(format!("{}", i));

                        let key_visualizer_rect = egui::Rect::from_two_pos(
                            pos2(key_rect.min.x, ui.cursor().min.y),
                            key_rect.max,
                        );

                        let key_range = 100.;
                        let key_position =
                            ((self.time.elapsed().as_secs_f32() * 2. + (i as f32 + 1.)).sin()
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
                                        key_position / key_range * key_visualizer_rect.height(),
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
    }
}
