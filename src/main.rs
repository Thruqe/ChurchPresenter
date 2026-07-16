mod models;
mod db;
mod ui;
mod ndi_out;

use gtk::prelude::*;
use gtk::Application;

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
    let saves_dir = "/home/thruqe/Documents/Church-Presenter/saves";
    let _ = std::fs::create_dir_all(saves_dir);
    let log_path = "/home/thruqe/Documents/Church-Presenter/saves/logs.txt";
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
    {
        use simplelog::*;
        let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), file);
    }

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run();
}
