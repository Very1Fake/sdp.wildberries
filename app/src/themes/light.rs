use iced::{button, container, rule, text_input, Background, Color, Vector};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Colors
////////////////////////////////////////////////////////////////////////////////////////////////////

// Text colors

pub const COLOR_TEXT: Color = Color::WHITE;
pub const COLOR_TEXT_LIGHT: Color = Color::from_rgba(0.4, 0.4, 0.4, 1.0); // #666
pub const COLOR_TEXT_MUTED: Color = Color::from_rgba(0.6, 0.6, 0.6, 1.0); // #999

// General colors

pub const OPPOSITE: Color = Color::BLACK;
pub const COLOR_PRIMARY: Color = Color::from_rgba(0.118, 0.529, 0.941, 1.0); // #1E87F0
pub const COLOR_PRIMARY_DARK: Color = Color::from_rgba(0.059, 0.478, 0.898, 1.0); // #0F7AE5
pub const COLOR_PRIMARY_DARKER: Color = Color::from_rgba(0.055, 0.427, 0.804, 1.0); // #0E6DCD
pub const COLOR_DANGER: Color = Color::from_rgba(0.827, 0.184, 0.184, 1.0); // #D32F2F
pub const COLOR_DANGER_DARK: Color = Color::from_rgba(0.776, 0.157, 0.157, 1.0); // #C62828
pub const COLOR_DANGER_DARKER: Color = Color::from_rgba(0.718, 0.11, 0.11, 1.0); // #B71C1C
pub const COLOR_SUCCESS: Color = Color::from_rgba(0.0, 0.475, 0.42, 1.0); // #00796B
pub const COLOR_SUCCESS_DARK: Color = Color::from_rgba(0.0, 0.412, 0.361, 1.0); // #C62828
pub const COLOR_SUCCESS_DARKER: Color = Color::from_rgba(0.0, 0.302, 0.251, 1.0); // #004D40
pub const COLOR_GREY_INACTIVE: Color = Color::from_rgba(0.741, 0.741, 0.741, 1.0); // #BDBDBD
pub const COLOR_GREY_ACTIVE: Color = Color::from_rgba(0.62, 0.62, 0.62, 1.0); // # 9E9E9E
pub const COLOR_BACKGROUND_GREY_INACTIVE: Color = Color::WHITE;
pub const COLOR_BACKGROUND_GREY_ACTIVE: Color = Color::from_rgba(0.878, 0.878, 0.878, 1.0); // #E0E0E0

// Other colors

pub const COLOR_BORDER: Color = Color::from_rgba(0.898, 0.898, 0.898, 1.0); // #E5E5E5
pub const COLOR_SELECTION: Color = Color::from_rgba(0.675, 0.808, 0.969, 1.0); // #ACCEF7

////////////////////////////////////////////////////////////////////////////////////////////////////
// Misc
////////////////////////////////////////////////////////////////////////////////////////////////////

pub const SECTION_DIVIDER_SPACING: u16 = 7;
pub const TASK_DIVIDER_SPACING: u16 = 5;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Button Primary
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct PrimaryButton;

impl button::StyleSheet for PrimaryButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_PRIMARY)),
            border_radius: 8.0,
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            text_color: COLOR_TEXT,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_PRIMARY_DARK)),
            shadow_offset: Vector::new(0.2, 0.2),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_PRIMARY_DARKER)),
            ..self.hovered()
        }
    }

    fn disabled(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::WHITE)),
            border_color: COLOR_BORDER,
            text_color: COLOR_TEXT_MUTED,
            ..self.active()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Button Danger
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DangerButton;

impl button::StyleSheet for DangerButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_DANGER)),
            border_radius: 8.0,
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            text_color: COLOR_TEXT,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_DANGER_DARK)),
            shadow_offset: Vector::new(0.2, 0.2),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_DANGER_DARKER)),
            ..self.hovered()
        }
    }

    fn disabled(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::WHITE)),
            border_color: COLOR_BORDER,
            text_color: COLOR_TEXT_MUTED,
            ..self.active()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Button Danger
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SuccessButton;

impl button::StyleSheet for SuccessButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_SUCCESS)),
            border_radius: 8.0,
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            text_color: COLOR_TEXT,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_SUCCESS_DARK)),
            shadow_offset: Vector::new(0.2, 0.2),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_SUCCESS_DARKER)),
            ..self.hovered()
        }
    }

    fn disabled(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::WHITE)),
            border_color: COLOR_BORDER,
            text_color: COLOR_TEXT_MUTED,
            ..self.active()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Icon button
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct IconButton;

impl button::StyleSheet for IconButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_BACKGROUND_GREY_INACTIVE)),
            text_color: COLOR_GREY_INACTIVE,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: COLOR_GREY_ACTIVE,
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_BACKGROUND_GREY_ACTIVE)),
            ..self.hovered()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Text Input
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TextInput;

impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            border_width: 1.0,
            border_color: COLOR_BORDER,
            ..Default::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: COLOR_PRIMARY,
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        COLOR_TEXT_MUTED
    }

    fn value_color(&self) -> Color {
        COLOR_TEXT_LIGHT
    }

    fn selection_color(&self) -> Color {
        COLOR_SELECTION
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Text Input Danger
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TextInputDanger;

impl text_input::StyleSheet for TextInputDanger {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            border_width: 2.0,
            border_color: COLOR_DANGER,
            ..Default::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: COLOR_DANGER_DARK,
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        COLOR_TEXT_MUTED
    }

    fn value_color(&self) -> Color {
        COLOR_TEXT_LIGHT
    }

    fn selection_color(&self) -> Color {
        COLOR_SELECTION
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Section divider
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SectionDivider;

impl rule::StyleSheet for SectionDivider {
    fn style(&self) -> rule::Style {
        rule::Style {
            color: Color::BLACK,
            width: 1,
            radius: 0.0,
            fill_mode: rule::FillMode::Full,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Task divider
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TaskDivider;

impl rule::StyleSheet for TaskDivider {
    fn style(&self) -> rule::Style {
        rule::Style {
            color: COLOR_GREY_INACTIVE,
            width: 1,
            radius: 0.0,
            fill_mode: rule::FillMode::Padded(8),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Card
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Card;

impl container::StyleSheet for Card {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 8.0,
            border_width: 0.4,
            border_color: [0.7, 0.75, 0.7, 1.0].into(),
            ..Default::default()
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Alert Box
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AlertBox;

impl container::StyleSheet for AlertBox {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 8.0,
            border_width: 2.0,
            border_color: COLOR_DANGER_DARK,
            text_color: Some(COLOR_DANGER),
            ..Default::default()
        }
    }
}
