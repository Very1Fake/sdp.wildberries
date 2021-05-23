use iced::{button, pick_list, slider, Container, Element, Length, Row, Text};

use crate::{layout::Message, themes::Theme, VERSION};

use super::{section, tab};

#[derive(Default)]
pub struct SettingsTab {
    pub theme_pick: pick_list::State<Theme>,
    pub scale_slider: slider::State,
    pub reset_button: button::State,
}

impl SettingsTab {
    pub fn render(&mut self, theme: &Theme, scale: f64) -> Element<Message> {
        tab(&String::from("Settings"))
            .push(
                section("Appearance", theme)
                    .push(
                        Row::new()
                            .push(Text::new("Theme").width(Length::FillPortion(1)))
                            .push(
                                pick_list::PickList::new(
                                    &mut self.theme_pick,
                                    &Theme::ALL[..],
                                    Some(*theme),
                                    |new| Message::Theme(new),
                                )
                                .width(Length::FillPortion(2)),
                            ),
                    )
                    .push(
                        Row::new()
                            .push(
                                Text::new(format!("Scale (x{})", scale))
                                    .width(Length::FillPortion(1)),
                            )
                            .push(
                                slider::Slider::new(
                                    &mut self.scale_slider,
                                    50.0f64..=300.0,
                                    scale * 100.0,
                                    |scale| Message::Scale(scale / 100.0),
                                )
                                .width(Length::FillPortion(2)),
                            ),
                    )
                    .push(
                        Container::new(
                            button::Button::new(&mut self.reset_button, Text::new("Reset"))
                                .on_press(Message::ResetAppearance)
                                .padding(8)
                                .style(theme.button()),
                        )
                        .width(Length::Fill)
                        .center_x(),
                    ),
            )
            .push(
                section("About", theme)
                    .push(
                        Row::new()
                            .push(Text::new("Name").width(Length::FillPortion(1)))
                            .push(Text::new("SDP Alpha").width(Length::FillPortion(2))),
                    )
                    .push(
                        Row::new()
                            .push(Text::new("Version").width(Length::FillPortion(1)))
                            .push(Text::new(VERSION).width(Length::FillPortion(2))),
                    ),
            )
            .into()
    }
}
