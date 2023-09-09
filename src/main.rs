mod app;
mod config;
mod device;

fn main() {
    // let mut devices = Keypad::available_devices();

    // println!("select your device:");
    // for (i, device) in devices.iter().enumerate() {
    //     println!("{i}) {}", device.name());
    // }

    // let mut input = String::new();
    // std::io::stdin()
    //     .read_line(&mut input)
    //     .expect("error reading input");

    // let keypad = input
    //     .trim()
    //     .parse::<usize>()
    //     .expect("invalid device number");

    // let mut keypad = Keypad::new(devices.remove(keypad));
    // println!("{:?}", keypad.get_info());
    app::run().unwrap();
}
