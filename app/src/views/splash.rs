use iced::{Align, Column, Container, Element, Length, Text};

use crate::{icons::icon, layout::Message, themes::Theme};

////////////////////////////////////////////////////////////////////////////////////////////////////
// View rendering & processing
////////////////////////////////////////////////////////////////////////////////////////////////////

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
