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
static EDITION: &str = "WILDBERRIES";
static SITE: &str = "Wildberries";

static ACCOUNTS_FILE: &str = "./accounts.json";
static PROXY_FILE: &str = "./proxy.json";
static SETTINGS_FILE: &str = "./settings.json";
static LICENSE_FILE: &str = "./license.jwt";

#[cfg(target_os = "linux")]
static H_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:91.0) Gecko/20100101 Firefox/91.0";
#[cfg(target_os = "windows")]
static H_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:91.0) Gecko/20100101 Firefox/91.0";

static H_HOST: &str = "www.wildberries.ru";
static H_ACCEPT: &str = "*/*";
static H_ACCEPT_LANGUAGE: &str = "ru-RU,ru;q=0.5";
static H_ACCEPT_ENCODING: &str = "gzip, deflate, br";
static H_X_REQUESTED_WITH: &str = "XMLHttpRequest";
static H_X_SPA_VERSION: &str = "8.0.4";
static H_SEC_FETCH_DEST: &str = "empty";
static H_SEC_FETCH_MODE: &str = "cors";
static H_SEC_FETCH_SITE: &str = "same-origin";
static H_PRAGMA: &str = "no-cache";
static H_CACHE_CONTROL: &str = "no-cache";
static H_ORIGIN: &str = "https://www.wildberries.ru";
static H_TE: &str = "trailers";

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
        default_font: Some(include_bytes!("../assets/fonts/roboto.ttf")),
        exit_on_close_request: false,
        ..Default::default()
    })
}
