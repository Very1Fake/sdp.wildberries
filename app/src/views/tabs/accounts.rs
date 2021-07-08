use iced::{
    button, scrollable, text_input, Align, Button, Checkbox, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Text, TextInput,
};
use serde::{Deserialize, Serialize};

use crate::{
    icons::{icon, Icon},
    layout::Message,
    logic::validator::{email, phone},
    themes::Theme,
};

use super::tab;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Account
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(default)]
pub struct Account {
    pub login: String,
    pub password: String,
    pub active: bool,

    #[serde(skip)]
    pub state: AccountState,
}

impl Account {
    pub fn new() -> Account {
        Account {
            login: String::new(),
            password: String::new(),
            active: true,
            state: AccountState::Edit {
                login_error: false,
                password_error: false,
                login_edit: Default::default(),
                password_edit: Default::default(),
                save_btn: Default::default(),
                delete_btn: Default::default(),
            },
        }
    }

    pub fn update(&mut self, msg: AccountMsg) {
        match msg {
            AccountMsg::Edit => {
                self.state = AccountState::Edit {
                    login_error: false,
                    password_error: false,
                    login_edit: text_input::State::focused(),
                    password_edit: text_input::State::new(),
                    save_btn: button::State::new(),
                    delete_btn: button::State::new(),
                }
            }
            AccountMsg::LoginChanged(login) => self.login = login,
            AccountMsg::PasswordChanged(password) => self.password = password,
            AccountMsg::SwitchStatus(active) => self.active = active,
            AccountMsg::Save => {
                if let AccountState::Edit {
                    ref mut login_error,
                    ref mut password_error,
                    ..
                } = self.state
                {
                    if email(&self.login) || phone(&self.login) {
                        *login_error = false;
                        if self.password.len() != 0 {
                            self.state = AccountState::default();
                        } else {
                            *password_error = true;
                        }
                    } else {
                        *login_error = true;
                    }
                }
            }
            AccountMsg::Delete => (),
        }
    }

    fn view(&mut self, theme: &Theme) -> Element<AccountMsg> {
        Container::new(match self.state {
            AccountState::View { ref mut edit_btn } => Row::new()
                .push(
                    Checkbox::new(self.active, "", AccountMsg::SwitchStatus).width(Length::Shrink),
                )
                .push(Text::new(&self.login).width(Length::FillPortion(5)))
                .push(Text::new("*".repeat(self.password.len())).width(Length::FillPortion(5)))
                .push(
                    Button::new(edit_btn, icon(Icon::Edit))
                        .on_press(AccountMsg::Edit)
                        .width(Length::Shrink)
                        .padding(8)
                        .style(theme.primary_btn()),
                )
                .align_items(Align::Center)
                .padding(8)
                .spacing(8),
            AccountState::Edit {
                login_error,
                password_error,
                ref mut login_edit,
                ref mut password_edit,
                ref mut save_btn,
                ref mut delete_btn,
            } => Row::new()
                .push(
                    TextInput::new(
                        login_edit,
                        if login_error {
                            "Example login format: \"+7(XXX)XXX-XX-XX\""
                        } else {
                            "Login (email or phone)"
                        },
                        &self.login,
                        AccountMsg::LoginChanged,
                    )
                    .padding(8)
                    .width(Length::FillPortion(5))
                    .style(if login_error {
                        theme.text_input_danger()
                    } else {
                        theme.text_input()
                    }),
                )
                .push(
                    TextInput::new(
                        password_edit,
                        if password_error {
                            "Password must be specified"
                        } else {
                            "Password"
                        },
                        &self.password,
                        AccountMsg::PasswordChanged,
                    )
                    .password()
                    .padding(8)
                    .width(Length::FillPortion(5))
                    .style(if password_error {
                        theme.text_input_danger()
                    } else {
                        theme.text_input()
                    }),
                )
                .push(
                    Button::new(save_btn, icon(Icon::Accept))
                        .on_press(AccountMsg::Save)
                        .width(Length::Shrink)
                        .padding(8)
                        .style(theme.success_btn()),
                )
                .push(
                    Button::new(delete_btn, icon(Icon::Delete))
                        .on_press(AccountMsg::Delete)
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

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.login == other.login && self.password == other.password && self.active == other.active
    }
}

#[derive(Debug)]
pub enum AccountState {
    View {
        edit_btn: button::State,
    },
    Edit {
        login_error: bool,
        password_error: bool,

        login_edit: text_input::State,
        password_edit: text_input::State,
        save_btn: button::State,
        delete_btn: button::State,
    },
}

impl Default for AccountState {
    fn default() -> Self {
        AccountState::View {
            edit_btn: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AccountMsg {
    Edit,
    LoginChanged(String),
    PasswordChanged(String),
    SwitchStatus(bool),
    Save,
    Delete,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// DataStore
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct AccountsTab {
    table_scroll: scrollable::State,
    new_btn: button::State,
}

impl AccountsTab {
    pub fn view<'a>(
        &'a mut self,
        theme: &Theme,
        accounts: &'a mut Vec<Account>,
    ) -> Element<'a, Message> {
        tab(&String::from("Data Store"))
            .push(
                Button::new(
                    &mut self.new_btn,
                    Text::new("Add account")
                        .width(Length::Fill)
                        .horizontal_alignment(HorizontalAlignment::Center),
                )
                .on_press(Message::NewAccount)
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
                        list.push(
                            account
                                .view(theme)
                                .map(move |msg| Message::Account(id, msg)),
                        )
                    },
                ),
            )
            .into()
    }
}
