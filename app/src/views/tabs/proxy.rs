use std::fmt::{Display, Formatter, Result};

use iced::{
    button, scrollable, text_input, Align, Button, Checkbox, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Text, TextInput,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    icons::{icon, Icon},
    layout::Message,
    themes::Theme,
};

use super::tab;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Proxy
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub struct Proxy {
    pub address: String,
    pub active: bool,

    #[serde(skip)]
    pub state: ProxyState,
}

impl Proxy {
    pub fn new() -> Proxy {
        Proxy {
            address: String::new(),
            active: true,
            state: ProxyState::Edit {
                address_error: false,
                address_edit: Default::default(),
                save_btn: Default::default(),
                delete_btn: Default::default(),
            },
        }
    }

    pub fn update(&mut self, msg: ProxyMsg) {
        match msg {
            ProxyMsg::Edit => {
                self.state = ProxyState::Edit {
                    address_error: false,
                    address_edit: text_input::State::focused(),
                    save_btn: button::State::new(),
                    delete_btn: button::State::new(),
                }
            }
            ProxyMsg::AddressChanged(address) => self.address = address,
            ProxyMsg::SwitchStatus(active) => self.active = active,
            ProxyMsg::Save => {
                if let ProxyState::Edit {
                    ref mut address_error,
                    ..
                } = self.state
                {
                    if self.address.len() > 3 {
                        self.state = ProxyState::default();
                    } else {
                        *address_error = true;
                    }
                }
            }
            ProxyMsg::Delete => (),
        }
    }

    fn view(&mut self, theme: &Theme) -> Element<ProxyMsg> {
        Container::new(match self.state {
            ProxyState::View { ref mut edit_btn } => Row::new()
                .push(Checkbox::new(self.active, "", ProxyMsg::SwitchStatus).width(Length::Shrink))
                .push(Text::new(&self.address).width(Length::FillPortion(1)))
                .push(
                    Button::new(edit_btn, icon(Icon::Edit))
                        .on_press(ProxyMsg::Edit)
                        .width(Length::Shrink)
                        .padding(8)
                        .style(theme.primary_btn()),
                )
                .align_items(Align::Center)
                .padding(8)
                .spacing(8),
            ProxyState::Edit {
                address_error,
                ref mut address_edit,
                ref mut save_btn,
                ref mut delete_btn,
            } => Row::new()
                .push(Checkbox::new(self.active, "", ProxyMsg::SwitchStatus).width(Length::Shrink))
                .push(
                    TextInput::new(
                        address_edit,
                        if address_error {
                            "Example proxy format: \"login:pass@ip:port\""
                        } else {
                            "Proxy address (\"login:pass@ip:port\")"
                        },
                        &self.address,
                        ProxyMsg::AddressChanged,
                    )
                    .padding(8)
                    .width(Length::FillPortion(1))
                    .style(if address_error {
                        theme.text_input_danger()
                    } else {
                        theme.text_input()
                    }),
                )
                .push(
                    Button::new(save_btn, icon(Icon::Accept))
                        .on_press(ProxyMsg::Save)
                        .width(Length::Shrink)
                        .padding(8)
                        .style(theme.success_btn()),
                )
                .push(
                    Button::new(delete_btn, icon(Icon::Delete))
                        .on_press(ProxyMsg::Delete)
                        .width(Length::Shrink)
                        .padding(8)
                        .style(theme.danger_btn()),
                )
                .align_items(Align::Center)
                .padding(8)
                .spacing(8),
        })
        .padding(2)
        .style(theme.card())
        .into()
    }
}

impl PartialEq for Proxy {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.active == other.active
    }
}

#[derive(Debug)]
pub enum ProxyState {
    View {
        edit_btn: button::State,
    },
    Edit {
        address_error: bool,

        address_edit: text_input::State,
        save_btn: button::State,
        delete_btn: button::State,
    },
}

impl Default for ProxyState {
    fn default() -> Self {
        ProxyState::View {
            edit_btn: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProxyMsg {
    Edit,
    AddressChanged(String),
    SwitchStatus(bool),
    Save,
    Delete,
}

#[derive(Deserialize_repr, Serialize_repr, Eq, PartialEq, Clone, Debug)]
#[repr(u8)]
pub enum ProxyMode {
    Off = 0,
    Repeat,
    Moderate,
    Strict,
}

impl ProxyMode {
    pub const ALL: [ProxyMode; 4] = [
        ProxyMode::Off,
        ProxyMode::Repeat,
        ProxyMode::Moderate,
        ProxyMode::Strict,
    ];
}

impl Default for ProxyMode {
    fn default() -> Self {
        ProxyMode::Repeat
    }
}

impl Display for ProxyMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                ProxyMode::Off => "Off",
                ProxyMode::Repeat => "Repeat",
                ProxyMode::Moderate => "Moderate",
                ProxyMode::Strict => "Strict",
            }
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// DataStore
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ProxyTab {
    table_scroll: scrollable::State,
    new_btn: button::State,
}

impl ProxyTab {
    pub fn view<'a>(
        &'a mut self,
        theme: &Theme,
        accounts: &'a mut Vec<Proxy>,
    ) -> Element<'a, Message> {
        tab(&String::from("Proxy list"))
            .push(
                Button::new(
                    &mut self.new_btn,
                    Text::new("Add proxy")
                        .width(Length::Fill)
                        .horizontal_alignment(HorizontalAlignment::Center),
                )
                .on_press(Message::NewProxy)
                .width(Length::Fill)
                .padding(8)
                .style(theme.primary_btn()),
            )
            .push(
                accounts.iter_mut().enumerate().rev().fold(
                    Scrollable::new(&mut self.table_scroll)
                        .width(Length::Fill)
                        .spacing(8),
                    |list, (id, account)| {
                        list.push(account.view(theme).map(move |msg| Message::Proxy(id, msg)))
                    },
                ),
            )
            .into()
    }
}
