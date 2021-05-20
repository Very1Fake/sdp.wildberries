use iced::{
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
    // Widgets
    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn button(&self) -> Box<dyn button::StyleSheet> {
        match *self {
            Theme::Light => light::ButtonPrimary.into(),
        }
    }

    pub fn text_input(&self) -> Box<dyn text_input::StyleSheet> {
        match *self {
            Theme::Light => light::TextInput.into(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Light
    }
}
