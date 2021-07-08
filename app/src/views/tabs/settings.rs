use iced::{
    button, pick_list, slider, text_input, Align, Button, Container, Element, Length, PickList,
    Row, Text, TextInput,
};

use crate::{layout::Message, themes::Theme, VERSION};

use super::{proxy::ProxyMode, section, tab, TabMsg};

#[derive(Clone, Debug)]
pub enum SettingsMsg {
    IdChanged(String),
    TokenChanged(String),
}

impl Into<Message> for SettingsMsg {
    fn into(self) -> Message {
        Message::TabMsg(TabMsg::SettingsMsg(self))
    }
}

#[derive(Default)]
pub struct SettingsTab {
    pub id_input: text_input::State,
    pub token_input: text_input::State,

    pub proxy_mode: pick_list::State<ProxyMode>,

    pub theme_pick: pick_list::State<Theme>,
    pub scale_slider: slider::State,

    pub reset_btn: button::State,
    pub logout_btn: button::State,
}

impl SettingsTab {
    pub fn update(&self, msg: SettingsMsg, w_id: &mut u128, w_token: &mut String) {
        match msg {
            SettingsMsg::IdChanged(id) => match id.parse::<u128>() {
                Ok(id) => *w_id = id,
                Err(_) => (),
            },
            SettingsMsg::TokenChanged(token) => {
                *w_token = if token.len() > 68 {
                    token[0..=67].to_string()
                } else {
                    token
                }
            }
        }
    }

    pub fn view(
        &mut self,
        theme: &Theme,
        scale: f64,
        proxy_mode: &ProxyMode,
        key: &String,
        w_id: u128,
        w_token: &String,
    ) -> Element<Message> {
        tab(&String::from("Settings"))
            .push(
                section("Discord webhook", theme)
                    .push(
                        Row::new()
                            .push(Text::new("ID").width(Length::FillPortion(1)))
                            .push(
                                TextInput::new(&mut self.id_input, "", &w_id.to_string(), |id| {
                                    SettingsMsg::IdChanged(id).into()
                                })
                                .width(Length::FillPortion(2))
                                .padding(8)
                                .style(theme.text_input()),
                            )
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Text::new("Token").width(Length::FillPortion(1)))
                            .push(
                                TextInput::new(&mut self.token_input, "", w_token, |token| {
                                    SettingsMsg::TokenChanged(token).into()
                                })
                                .width(Length::FillPortion(2))
                                .padding(8)
                                .style(theme.text_input()),
                            )
                            .align_items(Align::Center),
                    ),
            )
            .push(
                section("Connectivity", theme).push(
                    Row::new()
                        .push(Text::new("Proxy mode: ").width(Length::FillPortion(1)))
                        .push(
                            PickList::new(
                                &mut self.proxy_mode,
                                &ProxyMode::ALL[..],
                                Some(proxy_mode.clone()),
                                Message::ProxyMode,
                            )
                            .width(Length::FillPortion(2)),
                        )
                        .align_items(Align::Center),
                ),
            )
            .push(
                section("Appearance", theme)
                    .push(
                        Row::new()
                            .push(Text::new("Theme").width(Length::FillPortion(1)))
                            .push(
                                PickList::new(
                                    &mut self.theme_pick,
                                    &Theme::ALL[..],
                                    Some(theme.clone()),
                                    |new| Message::Theme(new),
                                )
                                .width(Length::FillPortion(2)),
                            )
                            .align_items(Align::Center),
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
                                    50..=300u16,
                                    (scale * 100.0) as u16,
                                    |scale| Message::Scale(scale as f64 / 100.0),
                                )
                                .width(Length::FillPortion(2)),
                            )
                            .align_items(Align::Center),
                    )
                    .push(
                        Container::new(
                            Button::new(&mut self.reset_btn, Text::new("Reset"))
                                .on_press(Message::ResetAppearance)
                                .padding(8)
                                .style(theme.primary_btn()),
                        )
                        .width(Length::Fill)
                        .center_x(),
                    ),
            )
            .push(
                section("About", theme)
                    .push(
                        Row::new()
                            .push(Text::new("Code Name").width(Length::FillPortion(1)))
                            .push(Text::new("SDP Alpha").width(Length::FillPortion(2)))
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Text::new("Version").width(Length::FillPortion(1)))
                            .push(Text::new(VERSION).width(Length::FillPortion(2)))
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Text::new("License key").width(Length::FillPortion(1)))
                            .push(Text::new(key).width(Length::FillPortion(2)))
                            .align_items(Align::Center),
                    )
                    .push(
                        Container::new(
                            Button::new(&mut self.logout_btn, Text::new("Logout"))
                                .on_press(Message::Logout)
                                .padding(8)
                                .style(theme.danger_btn()),
                        )
                        .width(Length::Fill)
                        .center_x(),
                    ),
            )
            .into()
    }
}
