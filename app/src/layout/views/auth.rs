use iced::{
    button, text_input, Align, Column, Command, Container, Element, Length, Row, Space, Text,
};

use super::{Message, View, ViewMessage};
use crate::layout::views::main::MainViewState;
use crate::layout::{themes::Theme, Layout};

#[derive(Clone, Debug)]
pub enum AuthMessage {
    Success,
    Failed,
    KeyInput(String),
    Submit,
}

impl Into<Message> for AuthMessage {
    #[inline(always)]
    fn into(self) -> Message {
        Message::View(ViewMessage::Auth(self))
    }
}

#[derive(Default)]
pub struct AuthViewState {
    pub key_state: text_input::State,
    pub key: String,
    pub button: button::State,
    pub stage: Stage,
}

#[derive(PartialEq)]
pub enum Stage {
    Waiting,
    Checking,
    Failed(u8),
}

impl Default for Stage {
    fn default() -> Self {
        Stage::Waiting
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// View rendering & processing
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn update(layout: &mut Layout, msg: AuthMessage) -> Command<Message> {
    let mut result: Command<Message> = Command::none();

    if let View::Auth(ref mut state) = layout.view {
        match msg {
            AuthMessage::Success => {
                layout.view = View::Main(MainViewState {
                    tabs: vec![(Default::default(), "i", "Module 1".to_string())],
                    ..Default::default()
                })
            }
            AuthMessage::Failed => state.stage = Stage::Failed(1),
            AuthMessage::KeyInput(val) => {
                if val.len() <= 19 {
                    if let View::Auth(ref mut state) = layout.view {
                        state.key = val
                    }
                }
            }
            AuthMessage::Submit => {
                use std::time::Duration;
                use tokio::time::sleep;

                let key = state.key.clone();
                if key.len() == 19 {
                    let mut count = 0;

                    for i in key.split("-") {
                        if i.len() == 4 {
                            if i.chars().all(|c| c.is_ascii_alphanumeric()) {
                                count += 1;
                            }
                        }
                    }

                    if count == 4 {
                        result = Command::perform(
                            async move {
                                sleep(Duration::from_millis(750)).await;
                                if key == "1234-1234-7777-9900" {
                                    ViewMessage::Auth(AuthMessage::Success)
                                } else {
                                    ViewMessage::Auth(AuthMessage::Failed)
                                }
                            },
                            Message::View,
                        );
                        state.stage = Stage::Checking;
                        return result;
                    }
                }
                println!("Error");
                state.stage = Stage::Failed(0)
            }
        }
    }

    result
}

pub fn view<'a>(state: &'a mut AuthViewState, theme: &Theme) -> Element<'a, Message> {
    let mut header = Row::new()
        .push(Text::new("Activation").size(24))
        .align_items(Align::Center)
        .spacing(16);

    if let Stage::Failed(err) = state.stage {
        header = header.push(
            match err {
                0 => Text::new("Failed: Incorrect key format #0"),
                1 => Text::new("Failed: Wrong key #1"),
                code => Text::new(format!("Failed: #{}", code)),
            }
            .color(theme.color_danger()),
        )
    }

    let mut button = button::Button::new(&mut state.button, Text::new("Activate")).padding(8);

    if state.stage != Stage::Checking {
        button = button.on_press(AuthMessage::Submit.into());
    }

    Container::new(
        Column::new()
            .max_width(800)
            .padding(32)
            .push(header)
            .push(Space::with_height(Length::Units(16)))
            .push(
                text_input::TextInput::new(
                    &mut state.key_state,
                    "Enter your activation key",
                    &state.key,
                    if state.stage == Stage::Checking {
                        |_| Message::None
                    } else {
                        |val| AuthMessage::KeyInput(val).into()
                    },
                )
                .on_submit(AuthMessage::Submit.into())
                .padding(8)
                .style(theme.text_input()),
            )
            .push(Space::with_height(Length::Units(8)))
            .push(
                Row::new()
                    .push(Container::new(
                        Text::new("Key format: XXXX-XXXX-XXXX-XXXX")
                            .color(theme.color_text_muted()),
                    ))
                    .push(
                        Container::new(button.style(theme.button()))
                            .width(Length::Fill)
                            .align_x(Align::End),
                    ),
            ),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
