use super::Message;

use auth::{AuthMessage, AuthViewState};
use splash::SplashMessage;

pub mod auth;
pub mod splash;

pub enum View {
    Splash,
    Auth(AuthViewState),
}

#[derive(Clone, Debug)]
pub enum ViewMessage {
    Splash(SplashMessage),
    Auth(AuthMessage),
}
