use std::fmt;

use iced::{
    container, rule,
    widget::{button, text_input},
    Color,
};

mod light;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Theme {
    Light,
}

impl Theme {
    pub const ALL: [Theme; 1] = [Theme::Light];

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

    pub fn primary_btn(&self) -> Box<dyn button::StyleSheet> {
        match *self {
            Theme::Light => light::PrimaryButton.into(),
        }
    }

    pub fn danger_btn(&self) -> Box<dyn button::StyleSheet> {
        match *self {
            Theme::Light => light::DangerButton.into(),
        }
    }

    pub fn success_btn(&self) -> Box<dyn button::StyleSheet> {
        match *self {
            Theme::Light => light::SuccessButton.into(),
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

    pub fn text_input_danger(&self) -> Box<dyn text_input::StyleSheet> {
        match *self {
            Theme::Light => light::TextInputDanger.into(),
        }
    }

    pub fn section_divider(&self) -> Box<dyn rule::StyleSheet> {
        match *self {
            Theme::Light => light::SectionDivider.into(),
        }
    }

    pub fn task_divider(&self) -> Box<dyn rule::StyleSheet> {
        match *self {
            Theme::Light => light::TaskDivider.into(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Containers
    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn card(&self) -> Box<dyn container::StyleSheet> {
        match *self {
            Theme::Light => light::Card.into(),
        }
    }

    pub fn alert_box(&self) -> Box<dyn container::StyleSheet> {
        match *self {
            Theme::Light => light::AlertBox.into(),
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

    pub fn task_divider_spacing(&self) -> u16 {
        match *self {
            Theme::Light => light::TASK_DIVIDER_SPACING,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Light
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Theme::Light => "Light",
            }
        )
    }
}
