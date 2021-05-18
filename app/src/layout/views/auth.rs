use iced::{
    widget::{text_input, Column, Container, Text},
    Element,
};

use super::{Message, View, ViewMessage};
use crate::layout::Layout;

pub struct AuthViewState {
    pub key_state: text_input::State,
    pub key: String,
}

#[derive(Clone, Debug)]
pub enum AuthMessage {
    Success,
    Failed,
    KeyInput(String),
}

impl Into<Message> for AuthMessage {
    #[inline(always)]
    fn into(self) -> Message {
        Message::View(ViewMessage::Auth(self))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// View rendering & processing
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn update(layout: &mut Layout, msg: AuthMessage) {
    match msg {
        AuthMessage::Success => println!("Auth: success"),
        AuthMessage::Failed => println!("Auth: failed"),
        AuthMessage::KeyInput(val) => {
            if val.len() <= 12 {
                if let View::Auth(ref mut state) = layout.view {
                    state.key = val
                }
            }
        }
    }
}

pub fn view<'a>(state: &'a mut AuthViewState) -> Element<'a, Message> {
    Column::new()
        .push(Text::new("Activation"))
        .push(text_input::TextInput::new(
            &mut state.key_state,
            "Enter your activation key",
            &state.key,
            |val| AuthMessage::KeyInput(val).into(),
        ))
        .into()
}
