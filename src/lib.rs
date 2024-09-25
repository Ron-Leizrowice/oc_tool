// src/lib.rs

pub mod actions;
pub mod constants;
pub mod errors;
pub mod models;
pub mod tweaks;
pub mod ui;
pub mod utils;

use druid::{AppLauncher, LocalizedString, WindowDesc};
use models::AppState;
use ui::build_root_widget;

pub fn run() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(LocalizedString::new("OC Tool"))
        .window_size((500.0, 400.0));

    let initial_state = AppState::default();

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("launch failed");
}
