mod alert;
mod app;
mod audio;
mod config;
mod monitor;
mod types;
mod ui;

use app::App;

fn main() -> iced::Result {
    App::start()
}
