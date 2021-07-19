use crate::{themes::Theme, views::tabs::proxy::ProxyMode};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Webhook
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Webhook {
    pub id: u64,
    pub token: String,
}

impl Default for Webhook {
    fn default() -> Self {
        Webhook {
            id: 0,
            token: String::new(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Settings
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(default)]
pub struct Settings {
    pub webhook: Webhook,
    pub proxy_mode: ProxyMode,

    // Appearance
    #[serde(skip)]
    pub theme: Theme,
    pub scale: f64,

    // Experimental flags
    pub limiter: bool,
    pub checker: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            webhook: Webhook::default(),
            proxy_mode: ProxyMode::default(),

            theme: Theme::Light,
            scale: 1.0,

            limiter: true,
            checker: true,
        }
    }
}
