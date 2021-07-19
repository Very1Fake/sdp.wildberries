use iced::{
    button, pick_list, slider, text_input, Align, Button, Checkbox, Container, Element, Length,
    PickList, Row, Text, TextInput,
};

use crate::{layout::Message, settings::Settings, themes::Theme, VERSION};

use super::{proxy::ProxyMode, section, tab, TabMsg};

#[derive(Clone, Debug)]
pub enum SettingsMsg {
    IdChanged(String),
    TokenChanged(String),
    ScaleChange(f64),
    ScaleApply,
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
    pub scale_apply: button::State,
    pub scale: f64,

    pub reset_btn: button::State,
    pub logout_btn: button::State,
}

impl SettingsTab {
    pub fn update(&mut self, msg: SettingsMsg, settings: &mut Settings) {
        match msg {
            SettingsMsg::IdChanged(id) => match id.parse::<u64>() {
                Ok(id) => settings.webhook.id = id,
                Err(_) => (),
            },
            SettingsMsg::TokenChanged(token) => {
                settings.webhook.token = if token.len() > 68 {
                    token[0..=67].to_string()
                } else {
                    token
                }
            }
            SettingsMsg::ScaleChange(scale) => self.scale = scale,
            SettingsMsg::ScaleApply => settings.scale = self.scale,
        }
    }

    pub fn view(&mut self, settings: &Settings, key: &String) -> Element<Message> {
        if self.scale == 0.0 {
            self.scale = settings.scale;
        }

        let mut scale_apply = Button::new(&mut self.scale_apply, Text::new("Apply"));

        if self.scale != settings.scale {
            scale_apply = scale_apply.on_press(SettingsMsg::ScaleApply.into());
        }

        tab(&String::from("Settings"))
            .push(
                section("Discord webhook", &settings.theme)
                    .push(
                        Row::new()
                            .push(Text::new("ID").width(Length::FillPortion(1)))
                            .push(
                                TextInput::new(
                                    &mut self.id_input,
                                    "",
                                    &settings.webhook.id.to_string(),
                                    |id| SettingsMsg::IdChanged(id).into(),
                                )
                                .width(Length::FillPortion(2))
                                .padding(8)
                                .style(settings.theme.text_input()),
                            )
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Text::new("Token").width(Length::FillPortion(1)))
                            .push(
                                TextInput::new(
                                    &mut self.token_input,
                                    "",
                                    &settings.webhook.token,
                                    |token| SettingsMsg::TokenChanged(token).into(),
                                )
                                .width(Length::FillPortion(2))
                                .padding(8)
                                .style(settings.theme.text_input()),
                            )
                            .align_items(Align::Center),
                    ),
            )
            .push(
                section("Connectivity", &settings.theme).push(
                    Row::new()
                        .push(Text::new("Proxy mode: ").width(Length::FillPortion(1)))
                        .push(
                            PickList::new(
                                &mut self.proxy_mode,
                                &ProxyMode::ALL[..],
                                Some(settings.proxy_mode.clone()),
                                Message::ProxyMode,
                            )
                            .width(Length::FillPortion(2)),
                        )
                        .align_items(Align::Center),
                ),
            )
            .push(
                section("Appearance", &settings.theme)
                    .push(
                        Row::new()
                            .push(Text::new("Theme").width(Length::FillPortion(1)))
                            .push(
                                PickList::new(
                                    &mut self.theme_pick,
                                    &Theme::ALL[..],
                                    Some(settings.theme.clone()),
                                    |new| Message::Theme(new),
                                )
                                .width(Length::FillPortion(2)),
                            )
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(
                                Text::new(format!("Scale (x{})", self.scale))
                                    .width(Length::FillPortion(1)),
                            )
                            .push(
                                Row::new()
                                    .push(
                                        slider::Slider::new(
                                            &mut self.scale_slider,
                                            50..=300u16,
                                            (self.scale * 100.0) as u16,
                                            |scale| {
                                                SettingsMsg::ScaleChange(scale as f64 / 100.0)
                                                    .into()
                                            },
                                        )
                                        .width(Length::Fill),
                                    )
                                    .push(scale_apply.style(settings.theme.primary_btn()))
                                    .width(Length::FillPortion(2))
                                    .spacing(8)
                                    .align_items(Align::Center),
                            ),
                    )
                    .push(
                        Container::new(
                            Button::new(&mut self.reset_btn, Text::new("Reset"))
                                .on_press(Message::ResetAppearance)
                                .padding(8)
                                .style(settings.theme.danger_btn()),
                        )
                        .width(Length::Fill)
                        .center_x(),
                    ),
            )
            .push(
                section("Experimental", &settings.theme)
                    .push(
                        Row::new()
                            .push(Text::new("Limiter").width(Length::FillPortion(1)))
                            .push(
                                Container::new(Checkbox::new(settings.limiter, "", |is| {
                                    Message::Experimental(0, is)
                                }))
                                .width(Length::FillPortion(2))
                                .center_x(),
                            )
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Text::new("Checker").width(Length::FillPortion(1)))
                            .push(
                                Container::new(Checkbox::new(settings.checker, "", |is| {
                                    Message::Experimental(1, is)
                                }))
                                .width(Length::FillPortion(2))
                                .center_x(),
                            )
                            .align_items(Align::Center),
                    ),
            )
            .push(
                section("About", &settings.theme)
                    .push(
                        Row::new()
                            .push(Text::new("Code Name").width(Length::FillPortion(1)))
                            .push(Text::new("SDP Pre-alpha").width(Length::FillPortion(2)))
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
                                .style(settings.theme.danger_btn()),
                        )
                        .width(Length::Fill)
                        .center_x(),
                    ),
            )
            .into()
    }
}
