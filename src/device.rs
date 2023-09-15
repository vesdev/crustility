use egui::load::Result;
use indexmap::IndexMap;
use std::sync::Mutex;
use std::time::Duration;
use std::{io::Read, sync::Arc};
use thiserror::Error;

use crate::config::{self, Config, HKey, Millimeter};

macro_rules! write_serial {
    ( $string:ident; $($key:literal, $idx:expr, $field:literal, $value:expr);+) => {
        $(
            $string += &format!("{}{}.{} {}\n", $key, $idx, $field, &$value.to_string());
        )*
    };

}

/// Serial Port
#[derive(Debug)]
struct Port {
    port: Option<Box<dyn serialport::SerialPort>>,
    port_name: String,
}

impl Port {
    fn new(port_name: String) -> Self {
        Self {
            port: None,
            port_name,
        }
    }
    /// Operate on a serial port
    ///
    /// Opens a new port if its not already open
    fn open(&mut self, parity: serialport::Parity) -> Result<(), Error> {
        if self.port.is_none() {
            let port = Box::new(serialport::new(self.port_name.clone(), 115_200))
                .timeout(Duration::from_millis(200))
                .flow_control(serialport::FlowControl::Hardware)
                .parity(parity)
                .data_bits(serialport::DataBits::Eight)
                .stop_bits(serialport::StopBits::One)
                .open();

            if port.is_err() && cfg!(unix) {
                let _ = std::process::Command::new("pkexec")
                    .arg("chmod")
                    .arg("666")
                    .arg(self.port_name.as_str())
                    .spawn();
            }

            self.port = Some(port?);
        }

        Ok(())
    }

    fn write(&mut self, data: impl Into<String>) -> Result<(), Error> {
        if let Some(port) = &mut self.port {
            port.write_all(data.into().as_bytes())?;
        }
        Ok(())
    }

    fn read(&mut self) -> Result<String, Error> {
        if let Some(port) = &mut self.port {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => return Ok(String::from_utf8(serial_buf[..t].to_vec()).unwrap()),
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                    Err(_) => break,
                }
            }
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
    port: Arc<Mutex<Port>>,
    name: String,
    key_count: u16,
    config: Option<Config>,
    is_dummy: bool,
    data_receiver: Option<std::sync::mpsc::Receiver<Event>>,
    event_sender: Option<std::sync::mpsc::Sender<SendEvent>>,
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
    pub fn port_name(&self) -> Result<String, Error> {
        Ok(self.port.lock().map_err(|_| Error::Read)?.port_name.clone())
    }
    pub fn config_mut(&mut self) -> Option<&mut Config> {
        self.config.as_mut()
    }
    pub fn set_config(&mut self, config: Config) {
        self.config = Some(config);
    }

    #[allow(unused)]
    pub fn config(&mut self) -> Option<&Config> {
        self.config.as_ref()
    }

    pub fn serialize_config(&mut self) -> Result<String, Error> {
        if self.is_dummy {
            return Err(Error::Parse);
        }

        if let Some(config) = &self.config {
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
            return Ok(commands);
        }
        Err(Error::Parse)
    }

    fn parse_config(raw_config: String) -> Result<Config, Error> {
        // the get ouput looks something like this
        // GET key=value
        // GET END

        // key -> prefix ?("." suffix)
        // prefix -> (("h" | "d") "key" number) | string
        // suffix & value -> string

        // let mut port = self.port.lock().map_err(|_| Error::Read)?;

        // port.open(serialport::Parity::Even)?;
        // port.write("get\n")?;

        let mut config = Config::default();

        for line in raw_config.lines() {
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
                    }
                    "dkeys" => {
                        //TODO
                    }
                    _ => (),
                };
            }
        }

        Ok(config)
    }

    fn read_sensors(port: &mut Port) -> Result<Vec<SensorData>, Error> {
        port.write("out\n")?;
        let mut result = Vec::new();
        for line in port.read()?.lines() {
            //sleep for per line or it will lag the device
            std::thread::sleep(Duration::from_millis(20));

            let Some(line) = line.strip_prefix("OUT ") else {
                continue;
            };
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let Some((raw, mapped)) = value.split_once(' ') else {
                continue;
            };

            let key_index = key[4..].parse::<usize>();
            let raw = raw.parse::<usize>();
            let mapped = mapped.parse::<usize>();
            if let (Ok(key_index), Ok(raw), Ok(mapped)) = (key_index, raw, mapped) {
                result.push(SensorData {
                    raw,
                    mapped: Millimeter::from_serial(mapped),
                    key: key_index - 1,
                });
            }
        }
        Ok(result)
    }

    pub fn recv_data(&mut self) -> Result<Event, Error> {
        let Some(data_receiver) = &mut self.data_receiver else {
            return Err(Error::Read);
        };
        let Ok(data) = data_receiver.try_recv() else {
            return Err(Error::Read);
        };

        Ok(data)
    }

    pub fn send_event(&mut self, event: SendEvent) -> Result<(), Error> {
        let Some(event_sender) = &mut self.event_sender else {
            return Err(Error::Send);
        };
        event_sender.send(event).map_err(|_| Error::Send)?;
        Ok(())
    }

    pub fn spawn_event_loop(&mut self) -> Result<(), Error> {
        if self.data_receiver.is_some() || self.event_sender.is_some() {
            return Ok(());
        }
        let (data_sender, data_receiver) = std::sync::mpsc::channel::<Event>();
        let (event_sender, event_receiver) = std::sync::mpsc::channel::<SendEvent>();
        self.data_receiver = Some(data_receiver);
        self.event_sender = Some(event_sender);

        if self.is_dummy {
            if let Some(config) = self.config.as_ref() {
                for (i, _) in config.hkeys.iter().enumerate() {
                    data_sender
                        .send(Event::Sensor(SensorData {
                            raw: 500 * i,
                            mapped: Millimeter::from(2. * i as f32),
                            key: i,
                        }))
                        .map_err(|_| Error::Send)?;
                }
                return Ok(());
            }
        }
        {
            let port = self.port.clone();

            #[allow(unreachable_code)]
            std::thread::spawn(move || {
                let mut port = port.lock().map_err(|_| Error::Read)?;

                let mut read_sensors = false;
                port.open(serialport::Parity::Even)?;
                data_sender.send(Event::Init).map_err(|_| Error::Send)?;

                loop {
                    if read_sensors {
                        let sensor_data = Self::read_sensors(&mut port)?;

                        for data in sensor_data {
                            let _ = data_sender
                                .send(Event::Sensor(data))
                                .map_err(|_| Error::Send);
                        }
                    }

                    if let Ok(event) = event_receiver.try_recv() {
                        match event {
                            SendEvent::SendCommands(cmds) => {
                                port.write(cmds).map_err(|_| Error::Send)?;
                            }
                            SendEvent::ReadSensorsBegin => read_sensors = true,
                            SendEvent::ReadSensorsEnd => read_sensors = false,
                            SendEvent::ReadConfig => {
                                port.write("get\n")?;
                                let mut raw_config = String::new();
                                loop {
                                    let line = port.read()?;
                                    if line.contains("GET END") {
                                        break;
                                    }

                                    log::debug!("{line}");
                                    raw_config += &line;
                                }
                                let config = Self::parse_config(raw_config)?;
                                data_sender
                                    .send(Event::Config(config))
                                    .map_err(|_| Error::Send)?;
                            }
                        }
                    }
                }
                Ok::<(), Error>(())
            });
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SendEvent {
    /// send commands without a return value
    SendCommands(String),
    ReadSensorsBegin,
    ReadSensorsEnd,
    ReadConfig,
}

#[derive(Debug)]
pub enum Event {
    Init,
    Sensor(SensorData),
    Config(Config),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SensorData {
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

    pub fn scan(&mut self) {
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
                        device.port = Arc::new(Mutex::new(Port::new(p.port_name.clone())));
                        Some((handle, device))
                    } else {
                        Some((
                            handle,
                            Device {
                                port: Arc::new(Mutex::new(Port::new(p.port_name.clone()))),
                                name: info.product.as_ref().map_or("", String::as_str).to_string(),
                                config: None,
                                key_count: 0,
                                is_dummy: false,
                                data_receiver: None,
                                event_sender: None,
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
                port: Arc::new(Mutex::new(Port::new("/dev/null".to_string()))),
                name: "<dummy>".to_string(),
                key_count: 3,
                config: Some(Config {
                    hkeys: vec![HKey::default(), HKey::default(), HKey::default()],
                    dkeys: Vec::new(),
                }),
                is_dummy: true,
                data_receiver: None,
                event_sender: None,
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
    #[error("serial port io")]
    Io(#[from] std::io::Error),

    #[error("could not read from the serial port")]
    Read,

    #[error("could not send the value")]
    Send,

    #[error("error parsing config")]
    Parse,
}
