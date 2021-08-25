use iced::{
    button, scrollable, text_input, Align, Button, Checkbox, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Space, Text, TextInput,
};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::{
    icons::{icon, Icon},
    layout::Message,
    logic::{
        misc::{client, request, RequestMethod},
        models::{ResponseResult, ResponseValue, User},
    },
    themes::Theme,
};

use super::{tab, TabMsg};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Account
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Account {
    pub phone: String,
    pub token: String,
    pub active: bool,

    #[serde(skip)]
    delete_btn: button::State,
}

impl Account {
    pub fn new(phone: String, token: String) -> Account {
        Account {
            phone,
            token,
            active: true,
            delete_btn: Default::default(),
        }
    }

    pub async fn info(token: &String) -> Result<User, AccountError> {
        match request(
            &mut client(
                None,
                Some(&[(
                    String::from("WILDAUTHNEW_V3"),
                    token.clone(),
                    String::from("wildberries.ru"),
                )]),
            ),
            &String::from("https://wildberries.ru/lk/personalcabinet/data"),
            RequestMethod::GET,
            "https://wildberries.ru/lk/",
            0,
        )
        .await
        {
            Ok(resp) => {
                let result = from_str::<ResponseResult>(&resp.body).unwrap();
                if let ResponseValue::Value(value) = result.value {
                    Ok(value.user.unwrap())
                } else {
                    Err(AccountError::InvalidToken)
                }
            }
            Err(err) => {
                println!("Err: {:?}", err);
                Err(AccountError::Unknown)
            }
        }
    }

    pub fn view(&mut self, theme: &Theme, id: usize) -> Element<AccountsMsg> {
        Container::new(
            Row::new()
                .push(
                    Checkbox::new(self.active, "", move |val| AccountsMsg::Status(id, val))
                        .width(Length::Shrink),
                )
                .push(Text::new(&self.phone).width(Length::FillPortion(1)))
                .push(Text::new(&format!("{}...", &self.token[..12])).width(Length::FillPortion(1)))
                .push(
                    Button::new(&mut self.delete_btn, icon(Icon::Delete))
                        .on_press(AccountsMsg::Delete(id))
                        .width(Length::Shrink)
                        .padding(8)
                        .style(theme.primary_btn()),
                )
                .align_items(Align::Center)
                .padding(8)
                .spacing(8),
        )
        .padding(2)
        .style(theme.card())
        .into()
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.phone == other.phone && self.token == other.token && self.active == other.active
    }
}

#[derive(Clone, Debug)]
pub enum AccountError {
    InvalidToken,
    Unknown,
}

impl Into<String> for AccountError {
    fn into(self) -> String {
        match self {
            AccountError::InvalidToken => String::from("Invalid token"),
            AccountError::Unknown => String::from("Unknown error"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// DataStore
////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum AccountsTab {
    List {
        table_scroll: scrollable::State,
        new_btn: button::State,
    },
    Add {
        error: String,
        checking: bool,

        token: String,
        phone: String,

        token_edit: text_input::State,
        continue_btn: button::State,
        close_btn: button::State,
    },
}

impl AccountsTab {
    pub fn update(&mut self, msg: AccountsMsg, accounts: &mut Vec<Account>) -> Command<Message> {
        match msg {
            AccountsMsg::List => *self = AccountsTab::default(),
            AccountsMsg::Add => {
                *self = AccountsTab::Add {
                    error: String::new(),
                    checking: false,
                    token: String::new(),
                    token_edit: Default::default(),
                    phone: String::new(),
                    continue_btn: Default::default(),
                    close_btn: Default::default(),
                }
            }
            AccountsMsg::CheckStart => {
                if let AccountsTab::Add {
                    error,
                    checking,
                    token,
                    ..
                } = self
                {
                    if accounts.iter().all(|a| &a.token != token) {
                        if (token.len() == 712 || token.len() == 680 || token.len() == 584)
                            && token
                                .chars()
                                .all(|c| c.is_ascii_uppercase() || c.is_numeric())
                        {
                            *error = String::new();
                            *checking = true;
                            let token = token.clone();
                            return Command::perform(
                                async move { Account::info(&token).await },
                                |result| AccountsMsg::CheckEnd(result).into(),
                            );
                        } else {
                            *error = String::from(
                                "The token must be 584/680/712 characters long. Allowed characters: A-Z,0-9",
                            );
                        }
                    } else {
                        *error = String::from("Account with this token already exists");
                    }
                }
            }
            AccountsMsg::CheckEnd(user) => {
                if let AccountsTab::Add {
                    error,
                    checking,
                    phone,
                    ..
                } = self
                {
                    match user {
                        Ok(user) => {
                            if accounts.iter().all(|a| a.phone != user.phone_str) {
                                *phone = user.phone_str
                            } else {
                                *error =
                                    format!("Account with {} number already exists", user.phone_str)
                            }
                        }
                        Err(err) => *error = err.into(),
                    }
                    *checking = false;
                }
            }
            AccountsMsg::Create(phone, token) => {
                accounts.push(Account::new(phone, token));
                *self = AccountsTab::default();
            }
            AccountsMsg::Change(val) => {
                if let AccountsTab::Add { token, .. } = self {
                    *token = val;
                }
            }
            AccountsMsg::Status(id, active) => accounts[id].active = active,
            AccountsMsg::Delete(id) => {
                accounts.remove(id);
            }
        }

        Command::none()
    }

    pub fn view<'a>(
        &'a mut self,
        theme: &Theme,
        accounts: &'a mut Vec<Account>,
    ) -> Element<'a, Message> {
        match self {
            AccountsTab::List {
                table_scroll,
                new_btn,
            } => tab(&String::from("Data Store"))
                .push(
                    Button::new(
                        new_btn,
                        Text::new("Add account")
                            .width(Length::Fill)
                            .horizontal_alignment(HorizontalAlignment::Center),
                    )
                    .on_press(AccountsMsg::Add.into())
                    .width(Length::Fill)
                    .padding(8)
                    .style(theme.primary_btn()),
                )
                .push(accounts.iter_mut().enumerate().rev().fold(
                    Scrollable::new(table_scroll).width(Length::Fill).spacing(8),
                    |list, (id, account)| {
                        list.push(account.view(theme, id).map(move |msg| msg.into()))
                    },
                ))
                .into(),
            AccountsTab::Add {
                error,
                checking,
                token,
                token_edit,
                phone,
                continue_btn,
                close_btn,
            } => {
                let mut continue_button = Button::new(
                    continue_btn,
                    Text::new(if phone.is_empty() { "Check" } else { "Save" })
                        .width(Length::Fill)
                        .horizontal_alignment(HorizontalAlignment::Center),
                )
                .width(Length::Units(128))
                .padding(8)
                .style(theme.primary_btn());

                if !*checking {
                    if phone.is_empty() {
                        continue_button = continue_button.on_press(AccountsMsg::CheckStart.into());
                    } else {
                        continue_button = continue_button
                            .on_press(AccountsMsg::Create(phone.clone(), token.clone()).into())
                    }
                }

                Container::new(
                    Container::new(
                        Column::new()
                            .push(
                                Text::new(if phone.is_empty() {
                                    "Account token"
                                } else {
                                    "Is this the account number?"
                                })
                                .size(32)
                                .width(Length::Fill)
                                .horizontal_alignment(HorizontalAlignment::Center),
                            )
                            .push(Space::with_height(Length::Units(24)))
                            .push(
                                Text::new(if error.is_empty() { "" } else { &error })
                                    .width(Length::Fill)
                                    .horizontal_alignment(HorizontalAlignment::Center)
                                    .color(theme.color_danger()),
                            )
                            .push(Space::with_height(Length::Units(if error.is_empty() {
                                0
                            } else {
                                8
                            })))
                            .push(
                                TextInput::new(
                                    token_edit,
                                    r#"Paste here "WILDAUTHNEW_V3" cookie from your browser"#,
                                    if phone.is_empty() { &token } else { &phone },
                                    if phone.is_empty() && !*checking {
                                        |val| AccountsMsg::Change(val).into()
                                    } else {
                                        |_| Message::None
                                    },
                                )
                                .on_submit(if phone.is_empty() {
                                    AccountsMsg::CheckStart.into()
                                } else {
                                    Message::None
                                })
                                .padding(8)
                                .style(theme.text_input()),
                            )
                            .push(Space::with_height(Length::Units(16)))
                            .push(
                                Container::new(continue_button)
                                    .width(Length::Fill)
                                    .center_x(),
                            )
                            .push(Space::with_height(Length::Units(24)))
                            .push(
                                Container::new(
                                    Button::new(
                                        close_btn,
                                        Text::new("Cancel")
                                            .width(Length::Fill)
                                            .horizontal_alignment(HorizontalAlignment::Center),
                                    )
                                    .on_press(AccountsMsg::List.into())
                                    .width(Length::Units(128))
                                    .padding(8)
                                    .style(theme.danger_btn()),
                                )
                                .width(Length::Fill)
                                .center_x(),
                            ),
                    )
                    .padding(32)
                    .style(theme.card()),
                )
                .padding(32)
                .into()
            }
        }
    }
}

impl Default for AccountsTab {
    fn default() -> Self {
        AccountsTab::List {
            table_scroll: Default::default(),
            new_btn: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AccountsMsg {
    List,
    Add,

    CheckStart,
    CheckEnd(Result<User, AccountError>),
    Create(String, String),
    Change(String),

    Status(usize, bool),
    Delete(usize),
}

impl Into<Message> for AccountsMsg {
    fn into(self) -> Message {
        Message::TabMsg(TabMsg::AccountsMsg(self))
    }
}
