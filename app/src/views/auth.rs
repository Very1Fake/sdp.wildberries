use iced::{
    button, text_input, Align, Column, Command, Container, Element, Length, Row, Space, Text,
};

use crate::{
    layout::Message,
    logic::activation::{Activation, ActivationError},
    themes::Theme,
};

use super::ViewMsg;

#[derive(Clone, Debug)]
pub enum AuthMsg {
    KeyInput(String),
    Submit,
    Failed(ActivationError),
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
    Failed(ActivationError),
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
    pub key: String,
    pub key_state: text_input::State,
    pub button: button::State,
    pub stage: Stage,
}

impl AuthViewState {
    pub fn update(&mut self, msg: AuthMsg) -> Command<Message> {
        match msg {
            AuthMsg::Failed(err) => self.stage = Stage::Failed(err),
            AuthMsg::KeyInput(val) => {
                self.key = if val.len() > 41 {
                    val[0..41].to_string()
                } else {
                    val
                }
            }
            AuthMsg::Submit => {
                let key = self.key.clone();
                if Activation::validate_key(&key) {
                    self.stage = Stage::Checking;
                    return Command::perform(
                        async move {
                            match Activation::activate(key).await {
                                Ok((activation, token)) => {
                                    Message::Activation { activation, token }
                                }
                                Err(err) => Message::ViewMsg(ViewMsg::Auth(AuthMsg::Failed(err))),
                            }
                        },
                        |msg| msg,
                    );
                }
                self.stage = Stage::Failed(ActivationError::InvalidKeyFormat)
            }
        }

        Command::none()
    }

    pub fn view(&mut self, theme: &Theme) -> Element<Message> {
        let mut header = Row::new()
            .push(Text::new("Activation").size(24))
            .align_items(Align::Center)
            .spacing(16);

        if let Stage::Failed(ref err) = self.stage {
            header = header.push(
                Text::new(format!("Failed: {} #{}", err.as_str(), err.code()))
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
                    .style(
                        if let Stage::Failed(ActivationError::InvalidKeyFormat) = self.stage {
                            theme.text_input_danger()
                        } else {
                            theme.text_input()
                        },
                    ),
                )
                .push(Space::with_height(Length::Units(8)))
                .push(
                    Row::new()
                        .push(Container::new(
                            Text::new("Key format: XXXXXX-XXXXXX-XXXXXX-XXXXXX-XXXXXX-XXXXXX")
                                .color(theme.color_text_muted()),
                        ))
                        .push(
                            Container::new(button.style(theme.primary_btn()))
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
