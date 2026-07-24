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

        #[cfg(target_os = "windows")]
        {
            std::env::set_var("GSK_RENDERER", "cairo");
            std::env::set_var("GDK_WIN32_DISABLE_DIRECT2D", "1");
            std::env::set_var("GDK_DISABLE", "d2d");
            std::env::set_var("GDK_DEBUG", "no-d2d");
        }

        #[cfg(not(target_os = "windows"))]
        {
            if std::env::var("GSK_RENDERER").is_err() {
                std::env::set_var("GSK_RENDERER", "cairo");
            }
        }
    }

    // Initialize simplelog logger (logging to both saves/logs.txt and ./logs.txt)
    let saves_dir = crate::db::get_saves_directory();
    let _ = std::fs::create_dir_all(&saves_dir);
    let log_path = saves_dir.join("logs.txt");
    let local_log_path = std::path::PathBuf::from("logs.txt");

    let file1 = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
        .ok();

    let file2 = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&local_log_path)
        .ok();

    use simplelog::*;
    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
    if let Some(f1) = file1 {
        loggers.push(WriteLogger::new(LevelFilter::Debug, Config::default(), f1));
    }
    if let Some(f2) = file2 {
        loggers.push(WriteLogger::new(LevelFilter::Debug, Config::default(), f2));
    }
    let _ = CombinedLogger::init(loggers);

    println!("=== ChurchPresenter v{} Starting ===", env!("CARGO_PKG_VERSION"));
    println!("Saves Directory: {:?}", saves_dir);
    println!("Log File Path: {:?}", log_path);

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run();
}
