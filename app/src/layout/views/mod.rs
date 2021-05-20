use super::Message;

use auth::{AuthMessage, AuthViewState};
use main::{MainMessage, MainViewState};
use splash::SplashMessage;

pub mod auth;
pub mod main;
pub mod splash;

pub enum View {
    Splash,
    Auth(AuthViewState),
    Main(MainViewState),
}

#[derive(Clone, Debug)]
pub enum ViewMessage {
    Splash(SplashMessage),
    Auth(AuthMessage),
    Main(MainMessage),
}
