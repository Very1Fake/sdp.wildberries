use iced::{
    widget::{text_input, Container, Text},
    Element, Length,
};

use super::{auth::AuthViewState, Message, View};
use crate::layout::Layout;

#[derive(Clone, Debug)]
pub enum SplashMessage {
    Done,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// View rendering & processing
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn update(layout: &mut Layout, msg: SplashMessage) {
    match msg {
        SplashMessage::Done => {
            layout.view = View::Auth(AuthViewState {
                key_state: text_input::State::new(),
                key: String::new(),
            })
        }
    }
}

pub fn view<'a>() -> Element<'a, Message> {
    Container::new(Text::new("Sellars Desktop Platform").size(42))
        .height(Length::Fill)
        .width(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
