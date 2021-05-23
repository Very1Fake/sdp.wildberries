use iced::{
    button, text_input, Align, Column, Command, Container, Element, Length, Row, Space, Text,
};

use super::{View, ViewMsg};
use crate::{layout::Message, themes::Theme};

#[derive(Clone, Debug)]
pub enum AuthMsg {
    KeyInput(String),
    Submit,
    Failed,
}

impl Into<Message> for AuthMsg {
    #[inline(always)]
    fn into(self) -> Message {
        Message::ViewMsg(ViewMsg::Auth(self))
    }
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

#[derive(Default)]
pub struct AuthViewState {
    pub key_state: text_input::State,
    pub key: String,
    pub button: button::State,
    pub stage: Stage,
}

impl AuthViewState {
    pub fn update(&mut self, msg: AuthMsg) -> Command<Message> {
        let mut result: Command<Message> = Command::none();

        match msg {
            AuthMsg::Failed => self.stage = Stage::Failed(1),
            AuthMsg::KeyInput(val) => {
                if val.len() <= 19 {
                    self.key = val
                }
            }
            AuthMsg::Submit => {
                use std::time::Duration;
                use tokio::time::sleep;

                let key = self.key.clone();
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
                                    Message::View(View::Main)
                                } else {
                                    AuthMsg::Failed.into()
                                }
                            },
                            |m| m,
                        );
                        self.stage = Stage::Checking;
                        return result;
                    }
                }
                self.stage = Stage::Failed(0)
            }
        }

        result
    }

    pub fn view(&mut self, theme: &Theme) -> Element<Message> {
        let mut header = Row::new()
            .push(Text::new("Activation").size(24))
            .align_items(Align::Center)
            .spacing(16);

        if let Stage::Failed(err) = self.stage {
            header = header.push(
                match err {
                    0 => Text::new("Failed: Incorrect key format #0"),
                    1 => Text::new("Failed: Wrong key #1"),
                    code => Text::new(format!("Failed: #{}", code)),
                }
                .color(theme.color_danger()),
            )
        }

        let mut button = button::Button::new(&mut self.button, Text::new("Activate")).padding(8);

        if self.stage != Stage::Checking {
            button = button.on_press(AuthMsg::Submit.into());
        }

        Container::new(
            Column::new()
                .max_width(800)
                .padding(32)
                .push(header)
                .push(Space::with_height(Length::Units(16)))
                .push(
                    text_input::TextInput::new(
                        &mut self.key_state,
                        "Enter your activation key",
                        &self.key,
                        if self.stage == Stage::Checking {
                            |_| Message::None
                        } else {
                            |val| AuthMsg::KeyInput(val).into()
                        },
                    )
                    .on_submit(AuthMsg::Submit.into())
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
}
