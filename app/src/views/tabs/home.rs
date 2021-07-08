use chrono::{DateTime, Local, NaiveDateTime, Utc};
use iced::{Align, Element, Length, Row, Text, VerticalAlignment};

use crate::{layout::Message, logic::activation::Activation, themes::Theme};

use super::{section, tab};

#[derive(Default)]
pub struct HomeTab;

impl HomeTab {
    pub fn view<'a>(&mut self, theme: &Theme, activation: &Activation) -> Element<'a, Message> {
        tab(&format!(
            "Welcome, {}",
            activation.name.split(" ").collect::<Vec<&str>>()[0]
        ))
        .push(
            section("Account details", theme)
                .push(
                    Row::new()
                        .push(
                            Text::new("Full name")
                                .width(Length::FillPortion(1))
                                .vertical_alignment(VerticalAlignment::Center),
                        )
                        .push(
                            Text::new(&activation.name)
                                .width(Length::FillPortion(2))
                                .vertical_alignment(VerticalAlignment::Center),
                        )
                        .height(Length::Units(32))
                        .align_items(Align::Center),
                )
                .push(
                    Row::new()
                        .push(
                            Text::new("Email")
                                .width(Length::FillPortion(1))
                                .vertical_alignment(VerticalAlignment::Center),
                        )
                        .push(
                            Text::new(&activation.email)
                                .width(Length::FillPortion(2))
                                .vertical_alignment(VerticalAlignment::Center),
                        )
                        .height(Length::Units(32))
                        .align_items(Align::Center),
                )
                .push(
                    Row::new()
                        .push(
                            Text::new("License expires at")
                                .width(Length::FillPortion(1))
                                .vertical_alignment(VerticalAlignment::Center),
                        )
                        .push(
                            Text::new(
                                DateTime::<Utc>::from_utc(
                                    NaiveDateTime::from_timestamp(activation.exp as i64, 0),
                                    Utc,
                                )
                                .with_timezone(&Local)
                                .format("%x %X")
                                .to_string(),
                            )
                            .width(Length::FillPortion(2))
                            .vertical_alignment(VerticalAlignment::Center),
                        )
                        .height(Length::Units(32))
                        .align_items(Align::Center),
                ),
        )
        .into()
    }
}
