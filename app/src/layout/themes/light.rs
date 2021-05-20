use iced::{
    widget::{button, text_input},
    Background, Color, Vector,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Colors
////////////////////////////////////////////////////////////////////////////////////////////////////

// Text colors

const COLOR_TEXT: Color = Color::WHITE;
const COLOR_TEXT_VALUE: Color = Color::from_rgba(
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
// Button Primary
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ButtonPrimary;

impl button::StyleSheet for ButtonPrimary {
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
        COLOR_TEXT_VALUE
    }

    fn selection_color(&self) -> Color {
        COLOR_SELECTION
    }
}
