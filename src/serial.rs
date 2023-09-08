use std::io::Read;
use std::{collections::HashMap, time::Duration};

pub fn get_minipad_info() -> HashMap<String, String> {
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
    }

    println!("specify port");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("no port provided");

    let mut port = serialport::new("/dev/ttyACM0", 115_200)
        .timeout(Duration::from_millis(200))
        .flow_control(serialport::FlowControl::Hardware)
        .parity(serialport::Parity::Even)
        .stop_bits(serialport::StopBits::One)
        .open()
        .expect("Failed to open port");

    port.write_all("get\n".as_bytes()).expect("Write failed!");

    let mut values = HashMap::new();

    let mut bytes = port.bytes();
    let mut line = String::new();
    while let Some(Ok(byte)) = bytes.next() {
        let byte = byte as char;
        if byte != '\n' || byte != '\r' {
            line.push(byte);
        } else {
            println!("{line}");

            if line == "GET END" {
                break;
            }

            if let Some(line) = line.strip_prefix("GET ") {
                if let Some((key, value)) = line.split_once('=') {
                    values.insert(key.to_string(), value.to_string());
                }
            }

            line.clear();
        }
    }

    values
}
