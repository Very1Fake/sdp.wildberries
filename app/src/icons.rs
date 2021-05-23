use iced::{Font, HorizontalAlignment, Length, Text};

const FONT: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../assets/fonts/icons.ttf"),
};

pub fn icon(unicode: &str) -> Text {
    Text::new(unicode)
        .font(FONT)
        .size(20)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
}
