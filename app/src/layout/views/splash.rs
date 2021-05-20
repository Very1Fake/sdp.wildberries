use iced::{Align, Column, Container, Element, Length, Text};

use super::{Message, View};
use crate::layout::{icons::icon, themes::Theme, Layout};

#[derive(Clone, Debug)]
pub enum SplashMessage {
    Done,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// View rendering & processing
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn update(layout: &mut Layout, msg: SplashMessage) {
    match msg {
        SplashMessage::Done => layout.view = View::Auth(Default::default()),
    }
}

pub fn view<'a>(theme: &Theme) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .push(
                icon("b")
                    .size(128)
                    .width(Length::Shrink)
                    .color(theme.color_primary()),
            )
            .push(Text::new("SDP").size(64))
            .spacing(8)
            .align_items(Align::Center),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
