use indexmap::IndexMap;
use std::fmt::Write;
use std::io::Read;
use std::{collections::HashMap, time::Duration};
use thiserror::Error;

use crate::config::{self, Config, DKey, HKey, Millimeter};

#[derive(Debug)]
struct Port {
    port: Option<Box<dyn serialport::SerialPort>>,
    port_name: String,
}

impl Port {
    /// operate on a serial port
    /// opens a new port if its not already open
    fn open<R>(
        &mut self,
        operations: impl FnOnce(&mut Box<dyn serialport::SerialPort>) -> R,
    ) -> Result<R, Error> {
        if self.port.is_none() {
            println!(
                "{}",
                "pkexec chmod 644 ".to_string() + self.port_name.as_str()
            );

            let connect_port = || {
                Box::new(serialport::new(self.port_name.clone(), 115_200))
                    .timeout(Duration::from_millis(200))
                    .flow_control(serialport::FlowControl::Hardware)
                    .parity(serialport::Parity::Even)
                    .stop_bits(serialport::StopBits::One)
                    .open()
            };

            let port = connect_port();

            match port {
                Err(serialport::Error {
                    kind: serialport::ErrorKind::Io(e),
                    description: _,
                }) if e == std::io::ErrorKind::PermissionDenied && cfg!(unix) => {
                    let _ = std::process::Command::new("pkexec")
                        .arg("chmod")
                        .arg("666")
                        .arg(self.port_name.as_str())
                        .spawn();
                }
                _ => (),
            }

            self.port = Some(port?);
        }

        // port is guaranteed to be some
        Ok(operations(unsafe { self.port.as_mut().unwrap_unchecked() }))
    }

    fn close(&mut self) {
        self.port = None;
    }
}

#[derive(Debug)]
pub struct Device {
    port: Port,
    name: String,
    key_count: u16,
    config: Option<Config>,
    config_modified: bool,
    is_dummy: bool,
}

impl Device {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn key_count(&self) -> u16 {
        self.key_count
    }
    pub fn port_name(&self) -> &String {
        &self.port.port_name
    }
    pub fn config_mut(&mut self) -> &mut Option<Config> {
        &mut self.config
    }

    pub fn write_config(&mut self) -> Result<(), Error> {
        if self.is_dummy {
            return Ok(());
        }

        if let Some(config) = &self.config {
            self.port.open(|port| {
                let mut commands = String::new();

                for (i, key) in config.hkeys.iter().enumerate() {
                    let i = i + 1;
                    if let Some(rt) = &key.rt {
                        writeln!(commands, "hkey{i}.rt 1").unwrap();
                        writeln!(commands, "hkey{i}.crt {}", rt.continuos).unwrap();
                        writeln!(commands, "hkey{i}.rtus {}", rt.up_sensitivity.to_serial())
                            .unwrap();
                        writeln!(commands, "hkey{i}.rtds {}", rt.down_sensitivity.to_serial())
                            .unwrap();
                        writeln!(commands, "hkey{i}.lh {}", key.hysterisis.lower.to_serial())
                            .unwrap();
                        writeln!(commands, "hkey{i}.uh {}", key.hysterisis.upper.to_serial())
                            .unwrap();
                        writeln!(commands, "hkey{i}.char {}", key.char.as_bytes()[0]).unwrap();
                        writeln!(commands, "hkey{i}.hid {}", key.hid).unwrap();
                    } else {
                        writeln!(commands, "hkey{i}.rt 0").unwrap();
                    }
                }

                port.write_all(commands.as_bytes()).expect("write failed");
                log::debug!("{}", commands);
            })?;
        }
        Ok(())
    }

    pub fn load_config_from_serial(&mut self) -> Result<(), Error> {
        if self.is_dummy {
            return Ok(());
        }
        // the get ouput looks something like this
        // GET key=value
        // GET END

        // key -> prefix ?("." suffix)
        // prefix -> (("h" | "d") "key" number) | string
        // suffix & value -> string

        self.port.open(|port| {
            port.write_all("get\n".as_bytes()).expect("write failed");

            let mut config = Config::default();

            let mut line = String::new();
            let mut bytes = port.bytes();
            while let Some(Ok(chr)) = bytes.next() {
                let chr = chr as char;
                if matches!(chr, '\n' | '\r') {
                    log::debug!("{line}");
                    if line == "GET END" {
                        self.config = Some(config);
                        return Ok(());
                    }

                    if let Some(line) = line.strip_prefix("GET ") {
                        if let Some((key, value)) = line.split_once('=') {
                            if let Some((lhs, rhs)) = key.split_once('.') {
                                map_key_to_config(&mut config, lhs, rhs, value);
                            } else {
                                match key {
                                    "hkeys" => {
                                        let key_count = value.parse::<u16>().unwrap();
                                        for _ in 0..key_count {
                                            config.hkeys.push(HKey::default());
                                        }
                                        self.key_count = key_count;
                                    }
                                    "dkeys" => {
                                        //TODO
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    line.clear();
                } else {
                    line.push(chr);
                }
            }
            println!("{:?}", self.name);
            Err(Error::Read)
        })?
    }
}

fn map_key_to_config(config: &mut Config, key_prefix: &str, key_suffix: &str, value: &str) {
    //TODO: parse values gracefully
    let key_index = key_prefix[4..].parse::<usize>().unwrap() - 1;
    match &key_prefix[..1] {
        "h" => match key_suffix {
            "rt" => {
                config.hkeys[key_index].rt = if value.parse::<u32>().unwrap() == 1 {
                    Some(config::RapidTrigger::default())
                } else {
                    None
                }
            }
            "crt" => {
                if let Some(rt) = &mut config.hkeys[key_index].rt {
                    rt.continuos = value.parse::<u16>().unwrap() == 1;
                };
            }
            "rtus" => {
                if let Some(rt) = &mut config.hkeys[key_index].rt {
                    rt.up_sensitivity = Millimeter::from_serial(value.parse::<u16>().unwrap())
                };
            }
            "rtds" => {
                if let Some(rt) = &mut config.hkeys[key_index].rt {
                    rt.down_sensitivity = Millimeter::from_serial(value.parse::<u16>().unwrap())
                };
            }
            "uh" => {
                config.hkeys[key_index].hysterisis.upper =
                    Millimeter::from_serial(value.parse::<u16>().unwrap())
            }
            "lh" => {
                config.hkeys[key_index].hysterisis.lower =
                    Millimeter::from_serial(value.parse::<u16>().unwrap())
            }
            "char" => {
                config.hkeys[key_index].char = (value.parse::<u8>().unwrap() as char).to_string()
            }
            "rest" => config.hkeys[key_index].rest = value.parse::<usize>().unwrap(),
            "down" => config.hkeys[key_index].down = value.parse::<usize>().unwrap(),
            "hid" => config.hkeys[key_index].hid = value.parse::<usize>().unwrap() == 1,
            _ => {}
        },
        "d" => {
            //TODO
        }
        _ => {}
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceHandle {
    pid: u16,
    vid: u16,
}

#[derive(Debug, Default)]
pub struct Devices {
    device_map: IndexMap<DeviceHandle, Device>,
}

impl Devices {
    pub fn iter(&self) -> DevicesIterator {
        DevicesIterator {
            handles: self.device_map.keys(),
        }
    }

    pub fn get_mut(&mut self, handle: &DeviceHandle) -> Option<&mut Device> {
        self.device_map.get_mut(handle)
    }

    pub fn get(&self, handle: &DeviceHandle) -> Option<&Device> {
        self.device_map.get(handle)
    }

    pub fn refresh(&mut self) {
        let ports = serialport::available_ports().expect("No ports found!");
        let mut old_devices = std::mem::take(&mut self.device_map);

        self.device_map = ports
            .iter()
            .filter_map(|p| {
                if let serialport::SerialPortType::UsbPort(info) = &p.port_type {
                    let handle = DeviceHandle {
                        pid: info.pid,
                        vid: info.vid,
                    };

                    if let Some(mut device) = old_devices.remove(&handle) {
                        device.port = Port {
                            // invalidate current port since it might have changed
                            port: None,
                            port_name: p.port_name.clone(),
                        };
                        Some((handle, device))
                    } else {
                        Some((
                            handle,
                            Device {
                                port: Port {
                                    port: None,
                                    port_name: p.port_name.clone(),
                                },
                                name: info.product.as_ref().map_or("", String::as_str).to_string(),
                                config: None,
                                config_modified: false,
                                key_count: 0,
                                is_dummy: false,
                            },
                        ))
                    }
                } else {
                    None
                }
            })
            .collect();

        self.device_map.insert(
            DeviceHandle { pid: 0, vid: 0 },
            Device {
                port: Port {
                    port: None,
                    port_name: "/dev/null".to_string(),
                },
                name: "<dummy>".to_string(),
                key_count: 3,
                config: Some(Config {
                    hkeys: vec![HKey::default(), HKey::default(), HKey::default()],
                    dkeys: Vec::new(),
                }),
                config_modified: false,
                is_dummy: true,
            },
        );
    }

    // pub fn available() -> Vec<Device> {}
}

pub struct DevicesIterator<'a> {
    handles: indexmap::map::Keys<'a, DeviceHandle, Device>,
}

impl<'a> Iterator for DevicesIterator<'a> {
    type Item = &'a DeviceHandle;

    fn next(&mut self) -> Option<Self::Item> {
        self.handles.next()
    }
}

pub type DeviceInfo = HashMap<String, String>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    Serial(#[from] serialport::Error),

    #[error("error parsing minipad info")]
    Parse(#[from] std::string::FromUtf8Error),

    #[error("could not read from the serial port")]
    Read,

    #[error("device disconnected")]
    Disconnect,

    #[error("cannot operate on a dummy device")]
    Dummy,
}
