use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use egui::{pos2, vec2, Color32, RichText};

use crate::{app::combobox, config::HKey};

use super::Crustility;

impl Crustility {
    pub fn device_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.key_options(ctx, ui);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    self.keys(ctx, ui, 60., 10.);
                });
            })
        });
    }

    pub fn key_options(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let Some(device) = self.device else {
            return;
        };
        let Some(device) = self.devices.get_mut(&device) else {
            return;
        };
        let Some(cfg) = device.config_mut() else {
            return;
        };

        let Some(key) = self.selected_key else {
            egui::panel::SidePanel::left("Key options")
                .frame(egui::Frame::central_panel(ui.style()))
                .resizable(false)
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.heading("Select Key");
                    });
                });
            return;
        };

        let rt_section = |ui: &mut egui::Ui, key: &mut HKey| {
            ui.add_space(10.);
            ui.group(|ui| {
                ui.heading("Rapid Trigger");
                let mut rt_enabled = key.rt.is_some();
                ui.checkbox(&mut rt_enabled, "Enable");

                if !rt_enabled {
                    key.rt = None;
                } else if key.rt.is_none() {
                    key.rt = Some(crate::config::RapidTrigger::default());
                }

                if let Some(rt) = &mut key.rt {
                    ui.checkbox(&mut rt.continuos, "Continuos");
                    ui.add(
                        egui::Slider::new((&mut rt.up_sensitivity).into(), 0.0..=4.)
                            .text("mm up sensitivity"),
                    );
                    ui.add(
                        egui::Slider::new((&mut rt.down_sensitivity).into(), 0.0..=4.)
                            .text("mm down sensitivity"),
                    );
                }
            });
        };

        let hysterisis_section = |ui: &mut egui::Ui, key: &mut HKey| {
            ui.add_space(20.);
            ui.group(|ui| {
                ui.heading("Hysterisis");
                ui.add(
                    egui::Slider::new((&mut key.hysterisis.upper).into(), 0.0..=4.)
                        .text("mm upper"),
                );
                ui.add(
                    egui::Slider::new((&mut key.hysterisis.lower).into(), 0.0..=4.)
                        .text("mm lower"),
                );
            });
        };

        let hid_section = |ui: &mut egui::Ui, key: &mut HKey| {
            ui.add_space(20.);
            ui.group(|ui| {
                ui.heading("HID");
                ui.checkbox(&mut key.hid, "Enable");
                ui.add(egui::TextEdit::singleline(&mut key.char).char_limit(1));
            });
        };

        egui::panel::SidePanel::left("Key options")
            .frame(egui::Frame::central_panel(ui.style()))
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Key ".to_string() + &(key + 1).to_string());
                    ui.separator();

                    let key = &mut cfg.hkeys[key];

                    rt_section(ui, key);
                    hysterisis_section(ui, key);
                    hid_section(ui, key);
                });
            });
    }

    pub fn menu_bar(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("Menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Apply Config").clicked() {
                    if let Some(device) = self.device {
                        if let Some(device) = self.devices.get_mut(&device) {
                            let err = device.write_config();
                            self.consume_error(err);
                        }
                    }
                };

                let current_rect = ui.max_rect();
                let combo_box_width = 80.;
                self.device_dropdown(
                    ui,
                    "menu bar devices",
                    egui::Rect::from_two_pos(
                        pos2(
                            current_rect.min.x + current_rect.width() / 2. - combo_box_width,
                            current_rect.min.y,
                        ),
                        pos2(
                            current_rect.min.x + current_rect.width() / 2. + combo_box_width,
                            current_rect.max.y,
                        ),
                    ),
                );
            });
        });
    }

    pub fn default_panel(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let current_rect = ui.max_rect();
                let combo_box_width = 80.;
                let combo_box_height = 10.;
                self.device_dropdown(
                    ui,
                    "no devices",
                    egui::Rect::from_two_pos(
                        pos2(
                            current_rect.min.x + current_rect.width() / 2. - combo_box_width,
                            current_rect.min.y + current_rect.height() / 2. - combo_box_height,
                        ),
                        pos2(
                            current_rect.min.x + current_rect.width() / 2. + combo_box_width,
                            current_rect.min.y + current_rect.height() / 2. + combo_box_height,
                        ),
                    ),
                );
            });
        });
    }

    pub fn device_dropdown(&mut self, ui: &mut egui::Ui, id: &str, rect: egui::Rect) {
        static REFRESH: AtomicBool = AtomicBool::new(true);
        ui.allocate_ui_at_rect(rect, |ui| {
            let response = combobox::ComboBox::from_id_source(id)
                .selected_text(
                    RichText::new(if let Some(d) = self.device {
                        if let Some(d) = self.devices.get(&d) {
                            d.name().as_str()
                        } else {
                            "<Select Device>" // device got disconnected
                        }
                    } else {
                        "<Select Device>"
                    })
                    .size(16.),
                )
                .width(rect.width())
                .show_ui(ui, |ui| {
                    //refresh devices when combo box is opened
                    if REFRESH.swap(false, Ordering::SeqCst) {
                        self.devices.rescan();
                    }
                    let devices = &self.devices;
                    ui.vertical_centered_justified(|ui| {
                        for device in devices.iter() {
                            ui.selectable_value(
                                &mut self.device,
                                Some(device.to_owned()),
                                RichText::new(if let Some(d) = devices.get(device) {
                                    d.name()
                                } else {
                                    "<Disconnected>"
                                })
                                .size(16.),
                            );
                        }
                    });
                });

            response.inner.is_none().then(|| {
                if !REFRESH.swap(true, Ordering::SeqCst) {
                    let mut result = Ok(());
                    if let Some(d) = self.device {
                        if let Some(d) = self.devices.get_mut(&d) {
                            result = d.load_config_from_serial();
                        }
                    }

                    self.consume_error(result);
                };
            });
        });
    }

    pub fn keys(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, key_width: f32, key_gap: f32) {
        let current_rect = ui.max_rect();
        let cursor = ui.cursor().min;
        let Some(device) = self.device else {
            return;
        };
        let Some(device) = self.devices.get_mut(&device) else {
            return;
        };
        let sensor_values = device.read_sensors();
        let Some(config) = &mut device.config_mut() else {
            return;
        };

        ctx.request_repaint_after(Duration::from_millis(30));

        let mut draw_visualizer = |ui: &mut egui::Ui, (i, key): (usize, &mut HKey)| {
            if let Ok(values) = &sensor_values {
                if let Some(value) = &values[i] {
                    key.current_position = value.mapped;
                }
            };

            // key.current_position = key.current_position
            //     + (key.target_position - key.current_position) / Millimeter::from(2.);

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
                    ui.heading(format!("{}", i + 1));
                    ui.separator();
                    let key_visualizer_rect = egui::Rect::from_two_pos(
                        pos2(key_rect.min.x, ui.cursor().min.y),
                        key_rect.max,
                    );

                    let is_selected = if let Some(idx) = self.selected_key {
                        idx == i
                    } else {
                        false
                    };

                    //background
                    ui.painter().rect(
                        key_visualizer_rect,
                        egui::Rounding::ZERO,
                        ui.visuals().widgets.inactive.bg_fill,
                        egui::Stroke::NONE,
                    );

                    //input
                    ui.painter().rect(
                        egui::Rect::from_two_pos(
                            key_visualizer_rect.min,
                            key_visualizer_rect.max
                                - vec2(
                                    0.,
                                    std::convert::Into::<f32>::into(key.current_position) / 4.
                                        * key_visualizer_rect.height(),
                                ),
                        ),
                        egui::Rounding::ZERO,
                        if is_selected {
                            ui.visuals().widgets.active.weak_bg_fill
                        } else {
                            ui.visuals().widgets.inactive.weak_bg_fill
                        },
                        egui::Stroke::NONE,
                    );

                    if ui
                        .add_sized(
                            vec2(key_rect.width(), key_visualizer_rect.height()),
                            egui::Button::new("").fill(Color32::TRANSPARENT),
                        )
                        .clicked()
                    {
                        if is_selected {
                            self.selected_key = None;
                        } else {
                            self.selected_key = Some(i);
                        }
                    };
                });
            });
        };

        ui.horizontal(|ui| {
            config
                .hkeys
                .iter_mut()
                .enumerate()
                .for_each(|key| draw_visualizer(ui, key));
        });
    }
}
