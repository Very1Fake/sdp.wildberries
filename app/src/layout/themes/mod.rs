use iced::{
    rule,
    widget::{button, text_input},
    Color,
};

mod light;

pub enum Theme {
    Light,
}

impl Theme {
    // pub const ALL: [Theme; 1] = [Theme::Light];

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Colors
    ////////////////////////////////////////////////////////////////////////////////////////////////

    // Text

    pub fn color_text_muted(&self) -> Color {
        match *self {
            Theme::Light => light::COLOR_TEXT_MUTED,
        }
    }

    // General

    pub fn color_opposite(&self) -> Color {
        match *self {
            Theme::Light => light::OPPOSITE,
        }
    }

    pub fn color_primary(&self) -> Color {
        match *self {
            Theme::Light => light::COLOR_PRIMARY,
        }
    }

    pub fn color_danger(&self) -> Color {
        match *self {
            Theme::Light => light::COLOR_DANGER,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Widgets styling
    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn button(&self) -> Box<dyn button::StyleSheet> {
        match *self {
            Theme::Light => light::Button.into(),
        }
    }

    pub fn icon_button(&self) -> Box<dyn button::StyleSheet> {
        match *self {
            Theme::Light => light::IconButton.into(),
        }
    }

    pub fn text_input(&self) -> Box<dyn text_input::StyleSheet> {
        match *self {
            Theme::Light => light::TextInput.into(),
        }
    }

    pub fn section_divider(&self) -> Box<dyn rule::StyleSheet> {
        match *self {
            Theme::Light => light::SectionDivider.into(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Misc
    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn section_divider_spacing(&self) -> u16 {
        match *self {
            Theme::Light => light::SECTION_DIVIDER_SPACING,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Light
    }
}
