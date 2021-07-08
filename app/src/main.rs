#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{window, Application, Result, Settings};

use layout::Layout;

mod icons;
mod layout;
mod logic;
mod themes;
mod views;

#[cfg(target_os = "windows")]
static OS: &str = "windows";
#[cfg(target_os = "linux")]
static OS: &str = "linux";
static VERSION: &str = env!("CARGO_PKG_VERSION");

static ACCOUNTS_FILE: &str = "./accounts.json";
static PROXY_FILE: &str = "./proxy.json";
static SETTINGS_FILE: &str = "./settings.json";
static LICENSE_FILE: &str = "./license.jwt";

#[cfg(target_os = "linux")]
static H_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0";
#[cfg(target_os = "windows")]
static H_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0";

static H_HOST: &str = "brandshop.ru";
static H_ACCEPT: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8";
static H_ACCEPT_LANGUAGE: &str = "ru-RU,ru;q=0.5";
static H_ACCEPT_ENCODING: &str = "gzip, deflate, br";
static H_ORIGIN: &str = "https://brandshop.ru";
static H_CONNECTION: &str = "keep-alive";
static H_UPGRADE_INSECURE_REQUESTS: &str = "1";

fn main() -> Result {
    #[cfg(debug_assertions)]
    let icon = include_bytes!("../assets/images/logo.rev").to_vec();
    #[cfg(not(debug_assertions))]
    let icon = include_bytes!("../assets/images/logo.raw").to_vec();

    Layout::run(Settings {
        window: window::Settings {
            size: (1280, 720),
            min_size: Some((1024, 720)),
            icon: Some(window::Icon::from_rgba(icon, 128, 128).unwrap()),
            ..window::Settings::default()
        },
        antialiasing: true,
        exit_on_close_request: false,
        ..Default::default()
    })
}
