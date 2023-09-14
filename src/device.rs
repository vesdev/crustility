use indexmap::IndexMap;
use std::io::Read;
use std::time::Duration;
use thiserror::Error;

use crate::config::{self, Config, HKey, Millimeter};

macro_rules! write_serial {
    ( $string:ident; $($key:literal, $idx:expr, $field:literal, $value:expr);+) => {
        $(
            $string += &format!("{}{}.{} {}\n\r", $key, $idx, $field, &$value.to_string());
        )*
    };

}

/// Serial Port
#[derive(Debug)]
struct Port {
    port: Option<Box<dyn serialport::SerialPort>>,
    port_name: String,
    unix_permission_requested: bool,
}

impl Port {
    fn new(port_name: String) -> Self {
        Self {
            port: None,
            port_name,
            unix_permission_requested: false,
        }
    }
    /// Operate on a serial port
    ///
    /// Opens a new port if its not already open
    fn open(
        &mut self,
        parity: serialport::Parity,
        // callback: impl FnOnce(&mut Box<dyn serialport::SerialPort>) -> R,
    ) -> Result<(), Error> {
        if self.port.is_none() {
            let port = Box::new(serialport::new(self.port_name.clone(), 115_200))
                .timeout(Duration::from_millis(200))
                .flow_control(serialport::FlowControl::Hardware)
                .parity(parity)
                .data_bits(serialport::DataBits::Eight)
                .stop_bits(serialport::StopBits::One)
                .open();

            match port {
                Err(serialport::Error {
                    kind: serialport::ErrorKind::Io(e),
                    description: _,
                }) if e == std::io::ErrorKind::PermissionDenied && cfg!(unix) => {
                    if self.unix_permission_requested {
                        let _ = std::process::Command::new("pkexec")
                            .arg("chmod")
                            .arg("666")
                            .arg(self.port_name.as_str())
                            .spawn();
                        self.unix_permission_requested = true;
                    }
                }
                _ => (),
            }

            self.port = Some(port?);
        }

        Ok(())
        // port is guaranteed to be some
        // Ok(callback(unsafe { self.port.as_mut().unwrap_unchecked() }))
    }

    fn write(&mut self, data: impl Into<String>) -> Result<(), Error> {
        if let Some(port) = &mut self.port {
            port.write_all(data.into().as_bytes())?;
        }
        Ok(())
    }

    fn read(&mut self) -> Result<String, Error> {
        if let Some(port) = &mut self.port {
            let mut result = String::new();

            while let Some(Ok(chr)) = port.bytes().next() {
                result.push(chr as char);
            }
            return Ok(result);
        }
        Err(Error::Read)
    }

    #[allow(unused)]
    fn close(&mut self) {
        self.port = None;
    }
}

/// Device using the minipad serial protocol
#[derive(Debug)]
pub struct Device {
    port: Port,
    name: String,
    key_count: u16,
    config: Option<Config>,
    is_dummy: bool,
}

impl Device {
    pub fn name(&self) -> &String {
        &self.name
    }
    #[allow(unused)]
    pub fn key_count(&self) -> u16 {
        self.key_count
    }
    #[allow(unused)]
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
            self.port.open(serialport::Parity::Even)?;

            let mut commands = String::new();

            for (i, key) in config.hkeys.iter().enumerate() {
                let idx = i + 1;
                if let Some(rt) = &key.rt {
                    write_serial!(
                        commands;
                        "hkey", idx, "rt",   1;
                        "hkey", idx, "crt",  rt.continuos;
                        "hkey", idx, "rtus", rt.up_sensitivity.to_serial();
                        "hkey", idx, "rtds", rt.down_sensitivity.to_serial();
                        "hkey", idx, "lh",   key.hysterisis.lower.to_serial();
                        "hkey", idx, "rtuh", key.hysterisis.upper.to_serial();
                        "hkey", idx, "char", key.char.as_bytes()[0];
                        "hkey", idx, "hid",  key.hid
                    );
                } else {
                    write_serial!(
                        commands;
                        "hkey", idx, "rt", 0
                    );
                }
            }

            log::debug!("{}", commands);
            self.port.write(commands)?;
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

        self.port.open(serialport::Parity::Even)?;
        self.port.write("get\n")?;

        let mut config = Config::default();

        for line in self.port.read()?.lines() {
            if line == "GET END" {
                self.config = Some(config);
                return Ok(());
            }

            let Some(line) = line.strip_prefix("GET ") else {
                continue;
            };
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

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
                    _ => (),
                };
            }
        }

        Ok(())
    }

    pub fn read_sensors(&mut self) -> Result<Vec<Option<SensorValue>>, Error> {
        if self.is_dummy {
            if let Some(config) = &mut self.config {
                let mut result = vec![None; config.hkeys.len()];
                for (i, _) in config.hkeys.iter().enumerate() {
                    result[i] = Some(SensorValue {
                        raw: 500 * i,
                        mapped: Millimeter::from(2. * i as f32),
                        key: i,
                    });
                }
                return Ok(result);
            }
        }

        self.port.open(serialport::Parity::None)?;
        if let Some(config) = &mut self.config {
            self.port.write("out\n")?;
            let mut result = vec![None; config.hkeys.len()];
            for line in self.port.read()?.lines() {
                // log::debug!("{:?}", line);
                let Some(line) = line.strip_prefix("OUT ") else {
                    return Err(Error::Read);
                };
                let Some((key, value)) = line.split_once('=') else {
                    return Err(Error::Read);
                };
                let Some((raw, mapped)) = value.split_once(' ') else {
                    return Err(Error::Read);
                };

                let key_index = key[4..].parse::<usize>();
                let raw = raw.parse::<usize>();
                let mapped = mapped.parse::<usize>();
                if let (Ok(key_index), Ok(raw), Ok(mapped)) = (key_index, raw, mapped) {
                    result[key_index - 1] = Some(SensorValue {
                        raw,
                        mapped: Millimeter::from_serial(mapped),
                        key: key_index - 1,
                    });
                }
            }
            Ok(result)
        } else {
            Err(Error::Read)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SensorValue {
    pub raw: usize,
    pub mapped: Millimeter,
    pub key: usize,
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
                    rt.up_sensitivity = Millimeter::from_serial(value.parse::<usize>().unwrap())
                };
            }
            "rtds" => {
                if let Some(rt) = &mut config.hkeys[key_index].rt {
                    rt.down_sensitivity = Millimeter::from_serial(value.parse::<usize>().unwrap())
                };
            }
            "uh" => {
                config.hkeys[key_index].hysterisis.upper =
                    Millimeter::from_serial(value.parse::<usize>().unwrap())
            }
            "lh" => {
                config.hkeys[key_index].hysterisis.lower =
                    Millimeter::from_serial(value.parse::<usize>().unwrap())
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

    pub fn rescan(&mut self) {
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
                        device.port = Port::new(p.port_name.clone());
                        Some((handle, device))
                    } else {
                        Some((
                            handle,
                            Device {
                                port: Port::new(p.port_name.clone()),
                                name: info.product.as_ref().map_or("", String::as_str).to_string(),
                                config: None,
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

        #[cfg(debug_assertions)]
        self.device_map.insert(
            DeviceHandle { pid: 0, vid: 0 },
            Device {
                port: Port::new("/dev/null".to_string()),
                name: "<dummy>".to_string(),
                key_count: 3,
                config: Some(Config {
                    hkeys: vec![HKey::default(), HKey::default(), HKey::default()],
                    dkeys: Vec::new(),
                }),
                is_dummy: true,
            },
        );
    }
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

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Serial(#[from] serialport::Error),

    #[error("error parsing minipad info")]
    Parse(#[from] std::string::FromUtf8Error),

    #[error("serial port io")]
    Io(#[from] std::io::Error),

    #[error("could not read from the serial port")]
    Read,
}
