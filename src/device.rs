use std::io::Read;
use std::{collections::HashMap, time::Duration};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct Device {
    port: String,
    name: String,
}

impl Device {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn port(&self) -> &String {
        &self.port
    }

    pub fn dummy() -> Self {
        Self {
            port: "/dev/null".to_string(),
            name: "<dummy>".to_string(),
        }
    }

    fn discard_dummy(&mut self) -> Result<(), Error> {
        if self.name == "<dummy>" {
            return Err(Error::Dummy);
        }

        Ok(())
    }

    fn open_port(&mut self) -> Result<Box<dyn serialport::SerialPort>, Error> {
        self.discard_dummy()?;

        // #[cfg(unix)]
        // std::os::unix::fs::PermissionsExt::set_mode(, )

        Ok(Box::new(serialport::new(self.port.clone(), 115_200))
            .timeout(Duration::from_millis(200))
            .flow_control(serialport::FlowControl::Hardware)
            .parity(serialport::Parity::Even)
            .stop_bits(serialport::StopBits::One)
            .open()?)
    }

    pub fn get_info(&mut self) -> Result<HashMap<String, String>, Error> {
        self.discard_dummy()?;
        let mut port = self.open_port()?;
        port.write_all("get\n".as_bytes()).expect("write failed");

        let mut values = HashMap::new();

        let mut line = String::new();
        let mut bytes = port.bytes();
        while let Some(Ok(chr)) = bytes.next() {
            let chr = chr as char;
            if matches!(chr, '\n' | '\r') {
                if line == "GET END" {
                    return Ok(values);
                }

                if let Some(line) = line.strip_prefix("GET ") {
                    if let Some((key, value)) = line.split_once('=') {
                        values.insert(key.to_string(), value.to_string());
                    }
                }

                line.clear();
            } else {
                line.push(chr);
            }
        }

        Err(Error::Read)
    }
}

pub fn available_devices() -> Vec<Device> {
    let ports = serialport::available_ports().expect("No ports found!");
    ports
        .iter()
        .filter_map(|p| {
            if let serialport::SerialPortType::UsbPort(info) = &p.port_type {
                Some(Device {
                    port: p.port_name.clone(),
                    name: info.product.as_ref().map_or("", String::as_str).to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    Serial(#[from] serialport::Error),

    #[error("error parsing minipad info")]
    Parse(#[from] std::string::FromUtf8Error),

    #[error("could not read from the serial port")]
    Read,

    #[error("cannot operate on a dummy device")]
    Dummy,
}
