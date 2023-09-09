use std::{rc::Rc, sync::atomic::AtomicUsize, time::Duration};

use egui::{pos2, vec2, Color32};

use crate::device;

use super::Crustility;

impl Crustility {
    pub fn key_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        //key options
        if let Some(key) = self.selected_key {
            egui::panel::SidePanel::left("Key options").show(ctx, |ui| {
                ui.checkbox(&mut true, "Rapid Trigger");
                ui.checkbox(&mut false, "Continuos Rapid Trigger");
            });
        }

        egui::panel::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Keys");
            ui.separator();

            self.keys(ctx, ui);
        });
    }

    pub fn menu_bar(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("Menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("File").clicked() {
                    println!("forsen");
                }
                egui::containers::ComboBox::from_id_source("Device")
                    .selected_text(
                        self.device
                            .as_ref()
                            .map_or(&"<Select Device>".to_string(), |d| d.name()),
                    )
                    .show_ui(ui, |ui| {
                        //scan for devices only once when the combo box is opened
                        let devices = self.available_devices.take();
                        let devices = if let Some(devices) = devices {
                            devices
                        } else {
                            device::available_devices()
                        };

                        for device in devices.iter() {
                            ui.selectable_value(
                                &mut self.device,
                                Some(device.to_owned()),
                                device.name(),
                            );
                        }

                        #[cfg(debug_assertions)]
                        ui.selectable_value(
                            &mut self.device,
                            Some(device::Device::dummy()),
                            "<dummy>",
                        );

                        self.available_devices = Some(devices);
                    })
                    .inner
                    .is_none()
                    .then(|| self.available_devices = None);

                if let Some(device) = &mut self.device {
                    if ui.button("Debug Device Info").clicked() {
                        println!("{:?}", device);
                        let info = device.get_info();
                        println!("{:#?}", self.consume_result(&info));
                    };
                };
            });
        });
    }

    pub fn status_bar(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("Status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(e) = &self.error {
                    match e {
                        device::Error::Serial(e)
                            if e.kind()
                                == serialport::ErrorKind::Io(
                                    std::io::ErrorKind::PermissionDenied,
                                ) =>
                        {
                            if let Some(device) = &self.device {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "PERMISSION DENIED: for port {:?}",
                                        device.port()
                                    ))
                                    .color(Color32::RED),
                                );

                                ui.label(
                                    egui::RichText::new(format!(
                                        "forgot to set the permission? sudo chmod 666 {}",
                                        device.port()
                                    ))
                                    .color(Color32::GREEN),
                                );
                            }
                        }
                        _ => {
                            ui.label(
                                egui::RichText::new(format!("ERROR: {:?}", &e)).color(Color32::RED),
                            );
                        }
                    }
                }
            });
        });
    }

    fn keys(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        //visualizer
        let key_count = 3;
        let key_width = 50.;
        let key_gap = 10.;
        let current_rect = ui.max_rect();
        let cursor = ui.cursor().min;
        ui.horizontal(|ui| {
            for i in 0..key_count {
                let key_rect = egui::Rect::from_two_pos(
                    egui::pos2(
                        current_rect.min.x + i as f32 * (key_width + key_gap),
                        cursor.y,
                    ),
                    egui::pos2(
                        current_rect.min.x + i as f32 * (key_width + key_gap) + key_width,
                        current_rect.max.y,
                    ),
                );

                ui.allocate_ui_at_rect(key_rect, |ui| {
                    ui.vertical_centered(|ui| {
                        let key_visualizer_rect = egui::Rect::from_two_pos(
                            pos2(key_rect.min.x, ui.cursor().min.y),
                            key_rect.max,
                        );

                        // ui.add_space(20.);

                        let key_range = 100.;
                        //simulate key press
                        let key_position =
                            ((self.time.elapsed().as_secs_f32() * 2. + (i as f32 + 1.)).sin()
                                * key_range
                                + key_range)
                                / 2.;

                        ctx.request_repaint_after(Duration::from_millis(20));

                        let is_selected = if let Some(idx) = self.selected_key {
                            idx == i
                        } else {
                            false
                        };

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
                            if is_selected {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().widgets.hovered.bg_fill
                            },
                            // egui::Color32::WHITE,
                            egui::Stroke::NONE,
                        );

                        if ui
                            .add_sized(
                                vec2(key_rect.width(), key_rect.height()),
                                egui::Button::new(format!("{i}")).fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            if is_selected {
                                self.selected_key = None;
                            } else {
                                self.selected_key = Some(i);
                            }
                        };
                    })
                });
            }
        });
    }
}
