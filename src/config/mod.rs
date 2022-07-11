mod app_info;
use std::{fs, path::Path};

use once_cell::sync::OnceCell;

use self::app_info::App;

#[allow(dead_code)]
static PUSHER_CONFIG: OnceCell<App> = OnceCell::new();

#[cfg(debug_assertions)]
pub(crate) fn load_from_test() {
    let vec = fs::read_to_string(Path::new("./config.toml")).expect("Config info not exist");
    let app = toml::from_str(&vec).expect("Parse to Toml Failure");

    PUSHER_CONFIG.set(app).expect("Config set")
}

pub fn set_config(cfg: App) {
    PUSHER_CONFIG.set(cfg).expect("Config Set")
}

pub(crate) fn get_config() -> &'static App {
    PUSHER_CONFIG.get().expect("Config Not Set")
}
