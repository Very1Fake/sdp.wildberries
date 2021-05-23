use auth::{AuthMsg, AuthViewState};
use tabs::TabsViewState;

pub mod auth;
pub mod splash;
pub mod tabs;

#[derive(Clone, Debug)]
pub enum View {
    Splash,
    Auth,
    Main,
}

impl View {
    pub fn state(&self) -> ViewState {
        match *self {
            View::Auth => ViewState::Auth(Default::default()),
            View::Main => ViewState::Main(Default::default()),
            _ => ViewState::None,
        }
    }
}

impl Default for View {
    fn default() -> Self {
        View::Splash
    }
}

pub enum ViewState {
    Auth(AuthViewState),
    Main(TabsViewState),
    None,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState::None
    }
}

#[derive(Clone, Debug)]
pub enum ViewMsg {
    Auth(AuthMsg),
}
