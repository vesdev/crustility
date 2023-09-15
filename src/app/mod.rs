use crate::device::{self, DeviceHandle, Devices};

mod combobox;
mod layout;
mod theme;

pub fn run() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(720.0, 480.0)),
        ..Default::default()
    };

    let mut devices = Devices::default();
    devices.scan();

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
        self.handle_data();
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

impl Crustility {
    fn handle_data(&mut self) {
        if let Some(device) = self.device {
            if let Some(device) = self.devices.get_mut(&device) {
                let _ = device.spawn_event_loop(); //TODO handle this error
                let Ok(data) = device.recv_data() else {
                    return;
                };

                match data {
                    device::Event::Init => {
                        let _ = device.send_event(device::SendEvent::ReadSensorsBegin);
                    }
                    device::Event::Sensor(v) => {
                        let Some(config) = device.config_mut() else {
                            return;
                        };
                        config.hkeys[v.key].target_position = v.mapped
                    }
                    device::Event::Config(v) => {
                        device.set_config(v);
                    }
                }
            };
        };
    }
}
