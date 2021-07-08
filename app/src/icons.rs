use iced::{Font, HorizontalAlignment, Length, Text};

const FONT: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../assets/fonts/icons.ttf"),
};

pub enum Icon {
    Logo,
    Home,
    Settings,
    Edit,
    Accept,
    Add,
    Delete,
    List,
    Account,
    ArrowDown,
    ArrowUp,
    Reload,
    Server,
}

impl Icon {
    fn to_str(&self) -> &str {
        match *self {
            Icon::Logo => "\u{0100}",
            Icon::Home => "\u{0101}",
            Icon::Settings => "\u{0102}",
            Icon::Edit => "\u{0103}",
            Icon::Accept => "\u{0104}",
            Icon::Add => "\u{0105}",
            Icon::Delete => "\u{0106}",
            Icon::List => "\u{0107}",
            Icon::Account => "\u{0108}",
            Icon::ArrowDown => "\u{0109}",
            Icon::ArrowUp => "\u{010A}",
            Icon::Reload => "\u{010B}",
            Icon::Server => "\u{010C}",
        }
    }
}

pub fn icon(i: Icon) -> Text {
    Text::new(i.to_str())
        .font(FONT)
        .size(20)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
}
