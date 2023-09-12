use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use egui::{
    emath::format_with_decimals_in_range, pos2, text::LayoutJob, vec2, widget_text::WidgetTextJob,
    Color32, RichText, WidgetText,
};

use crate::{
    app::combobox,
    config::{self, HKey},
    device,
};

use super::Crustility;

impl Crustility {
    pub fn device_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::panel::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Keys");
            // ui.separator();
            self.key_options(ctx, ui);

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(device) = self.device {
                    if let Some(device) = self.devices.get_mut(&device) {
                        let key_count = device.key_count() as usize;
                        ui.horizontal_top(|ui| {
                            self.keys(ctx, ui, key_count, 60., 10.);
                        });
                    }
                }
            });
        });
    }

    pub fn key_options(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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

        //key options
        if let Some(key) = self.selected_key {
            egui::panel::SidePanel::left("Key options")
                .frame(egui::Frame::central_panel(ui.style()))
                .resizable(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(device) = self.device {
                            if let Some(device) = self.devices.get_mut(&device) {
                                if let Some(cfg) = device.config_mut() {
                                    ui.heading("Key ".to_string() + &(key + 1).to_string());
                                    ui.separator();

                                    let key = &mut cfg.hkeys[key];

                                    rt_section(ui, key);
                                    hysterisis_section(ui, key);
                                    hid_section(ui, key);
                                }
                            } else {
                                ui.label("Device Disconnected");
                            }
                        }
                    });
                });
        } else {
            egui::panel::SidePanel::left("Key options")
                .frame(egui::Frame::central_panel(ui.style()))
                .resizable(false)
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.heading("Select Key");
                    });
                });
        }
    }

    pub fn menu_bar(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("Menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // if ui.button("File").clicked() {}
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
            // ui.style_mut().visuals.window_fill = Color32::TRANSPARENT;
            // ui.style_mut().visuals.panel_fill = Color32::TRANSPARENT;
            // ui.style_mut().visuals.button_frame = false;
            // ui.style_mut().override_text_style = Color32::TRANSPARENT;
            // egui::containers::con
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;
            ui.visuals_mut().widgets.active.weak_bg_fill = Color32::TRANSPARENT;
            ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::NONE;
            ui.visuals_mut().widgets.open.bg_stroke = egui::Stroke::NONE;

            let response = combobox::ComboBox::from_id_source(id)
                .selected_text(
                    RichText::new(if let Some(d) = self.device {
                        if let Some(d) = self.devices.get(&d) {
                            d.name().as_str()
                        } else {
                            "Disconnected"
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
                        self.devices.refresh();
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
                                    "Disconnected"
                                })
                                .size(16.),
                            );
                        }
                    });
                });
            // ui.set_min_height()
            // ui.style_mut().interact(response).weak_bg_fill

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
    pub fn status_bar(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::TopBottomPanel::bottom("Status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(e) = &self.error {
                    ui.label(egui::RichText::new(format!("ERROR: {:?}", &e)).color(Color32::RED));
                }
            });
        });
    }

    pub fn keys(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        key_count: usize,
        key_width: f32,
        key_gap: f32,
    ) {
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
                        ui.heading(format!("{}", i + 1));
                        ui.separator();
                        let key_visualizer_rect = egui::Rect::from_two_pos(
                            pos2(key_rect.min.x, ui.cursor().min.y),
                            key_rect.max,
                        );

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
                            egui::Rounding::ZERO,
                            if is_selected {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().widgets.hovered.bg_fill
                            },
                            egui::Stroke::NONE,
                        );

                        if ui
                            .add_sized(
                                vec2(key_rect.width(), key_rect.height()),
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
                    })
                });
            }
        });
    }
}
