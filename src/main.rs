// On Windows: use the GUI subsystem so no console window appears when the
// app is launched. Has no effect on Linux / macOS builds.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod db;
mod models;
mod ndi_out;
mod ui;

use gtk::Application;
use gtk::prelude::*;

const APP_ID: &str = "org.thruqe.gtkapp";

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
    };
}

fn main() {
    unsafe {
        std::env::set_var("GTK_CSD", "0");
    }

    // Initialize simplelog logger
    let saves_dir = crate::db::get_saves_directory();
    let log_path = saves_dir.join("logs.txt");
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
    {
        use simplelog::*;
        let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), file);
    }

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run();
}
