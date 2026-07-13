mod models;
mod db;
mod ui;
mod ndi_out;

use gtk::prelude::*;
use gtk::Application;

const APP_ID: &str = "org.thruqe.gtkapp";

fn main() {
    unsafe {
        std::env::set_var("GTK_CSD", "0");
    }
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run();
}
