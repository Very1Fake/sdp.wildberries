use iced::{window, Application, Result, Settings};

use layout::Layout;

mod layout;

fn main() -> Result {
    let icon = include_bytes!("../assets/images/logo.rev").to_vec();

    Layout::run(Settings {
        window: window::Settings {
            size: (1024, 720),
            min_size: Some((640, 480)),
            icon: Some(window::Icon::from_rgba(icon, 128, 128).unwrap()),
            ..window::Settings::default()
        },
        antialiasing: true,
        ..Default::default()
    })
}
