use iced::{button, rule, text_input, Background, Color, Vector};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Colors
////////////////////////////////////////////////////////////////////////////////////////////////////

// Text colors

const COLOR_TEXT: Color = Color::WHITE;
const COLOR_TEXT_LIGHT: Color = Color::from_rgba(
    0x66 as f32 / 256.0,
    0x66 as f32 / 256.0,
    0x66 as f32 / 256.0,
    1.0,
);
pub const COLOR_TEXT_MUTED: Color = Color::from_rgba(
    0x99 as f32 / 256.0,
    0x99 as f32 / 256.0,
    0x99 as f32 / 256.0,
    1.0,
);

// General colors

pub const OPPOSITE: Color = Color::BLACK;
pub const COLOR_PRIMARY: Color = Color::from_rgba(
    0x1E as f32 / 256.0,
    0x87 as f32 / 256.0,
    0xF0 as f32 / 256.0,
    1.0,
);
const COLOR_PRIMARY_DARKER: Color = Color::from_rgba(
    0x0F as f32 / 256.0,
    0x7A as f32 / 256.0,
    0xE5 as f32 / 256.0,
    1.0,
);
const COLOR_PRIMARY_DARK: Color = Color::from_rgba(
    0x0E as f32 / 256.0,
    0x6D as f32 / 256.0,
    0xCD as f32 / 256.0,
    1.0,
);
pub const COLOR_DANGER: Color = Color::from_rgba(
    0xF0 as f32 / 256.0,
    0x50 as f32 / 256.0,
    0x6E as f32 / 256.0,
    1.0,
);
const COLOR_GREY_INACTIVE: Color = Color::from_rgba(
    0xBD as f32 / 256.0,
    0xBD as f32 / 256.0,
    0xBD as f32 / 256.0,
    1.0,
);
const COLOR_GREY_ACTIVE: Color = Color::from_rgba(
    0x9E as f32 / 256.0,
    0x9E as f32 / 256.0,
    0x9E as f32 / 256.0,
    1.0,
);
const COLOR_BACKGROUND_GREY_INACTIVE: Color = Color::WHITE;
const COLOR_BACKGROUND_GREY_ACTIVE: Color = Color::from_rgba(
    0xE0 as f32 / 256.0,
    0xE0 as f32 / 256.0,
    0xE0 as f32 / 256.0,
    1.0,
);

// Other colors

const COLOR_BORDER: Color = Color::from_rgba(
    0xE5 as f32 / 256.0,
    0xE5 as f32 / 256.0,
    0xE5 as f32 / 256.0,
    1.0,
);
const COLOR_SELECTION: Color = Color::from_rgba(
    0xAC as f32 / 256.0,
    0xCE as f32 / 256.0,
    0xF7 as f32 / 256.0,
    1.0,
);

////////////////////////////////////////////////////////////////////////////////////////////////////
// Misc
////////////////////////////////////////////////////////////////////////////////////////////////////

pub const SECTION_DIVIDER_SPACING: u16 = 7;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Button Primary
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Button;

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_PRIMARY)),
            border_radius: 6.0,
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            text_color: COLOR_TEXT,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_PRIMARY_DARKER)),
            shadow_offset: Vector::new(0.2, 0.2),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(COLOR_PRIMARY_DARK)),
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
