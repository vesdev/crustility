#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod config;
mod device;

fn main() {
    app::run().unwrap();
}
