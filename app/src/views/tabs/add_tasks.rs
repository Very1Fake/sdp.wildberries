use std::fmt::{Display, Formatter};

use iced::{
    button, pick_list, text_input, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, PickList, Row, Space, Text, TextInput, VerticalAlignment,
};
use serde::Deserialize;
use serde_json::from_str;

use crate::{
    icons::{icon, Icon},
    layout::Message,
    logic::{
        misc::{client, request, RequestMethod, ResponseStatus},
        models::{ProductCard, ResponseResult, ResponseValue, SizeTag, Variant, Webhook},
    },
    themes::Theme,
};

use super::TabMsg;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Size
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct SizeOld {
    is_last: bool,
    name: String,
    #[serde(rename = "product_option_value_id")]
    so: String,
    #[serde(rename = "option_value_id")]
    sv: String,
    #[serde(rename = "product_option_id")]
    option: String,
}

impl Display for SizeOld {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.name,
            if self.is_last { " (last pair)" } else { "" }
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// CreateTasks Tab
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum AddTasksMsg {
    CodChange(String),
    SizeSelected(SizeTag),
    Check,
    Checked(Option<(ProductCard, Variant)>),
    Create,
    Done(String),
    Reset,
}

impl Into<Message> for AddTasksMsg {
    fn into(self) -> Message {
        Message::TabMsg(TabMsg::AddTasksMsg(self))
    }
}

#[derive(Default)]
pub struct AddTasksTab {
    processing: bool,
    error: String,

    cod: String,
    product: Option<(ProductCard, Variant)>,
    size: Option<SizeTag>,

    cod_input: text_input::State,
    size_pick: pick_list::State<SizeTag>,
    step_btn: button::State,
    reset_btn: button::State,
}

impl AddTasksTab {
    fn reset(&mut self) {
        self.error = String::new();
        self.product = None;
        self.size = None;
    }

    pub fn update(&mut self, msg: AddTasksMsg) -> Command<Message> {
        match msg {
            AddTasksMsg::CodChange(cod) if cod.parse::<u128>().is_ok() || cod.is_empty() => {
                if self.size.is_some() {
                    self.size = None
                }
                self.cod = cod
            }
            AddTasksMsg::SizeSelected(size) => {
                if size.quantity != 0 {
                    self.size = Some(size)
                } else {
                    self.error = String::from("This size cannot be selected (Sold Out)")
                }
            }
            AddTasksMsg::Create => {
                let (card, variant) = self.product.clone().unwrap();
                let size = if variant.sizes.len() > 1 {
                    variant.sizes[&self.size.clone().unwrap().id.to_string()].clone()
                } else {
                    variant.sizes.values().next().cloned().unwrap()
                };

                self.reset();

                return Command::perform(
                    async move { (card, variant, size) },
                    |(card, variant, size)| Message::AddTasks {
                        card,
                        variant,
                        size,
                    },
                );
            }
            AddTasksMsg::Check => {
                self.processing = true;
                self.error = String::new();
                let cod = self.cod.clone();

                return Command::perform(
                    async move {
                        match request(
                            &mut client(None, None),
                            &format!(
                                "https://www.wildberries.ru/{}/product/data?targetUrl=XS",
                                cod
                            ),
                            RequestMethod::GET,
                            &format!(
                                "https://www.wildberries.ru/catalog/{}/detail.aspx?targetUrl=XS",
                                cod
                            ),
                            0,
                        )
                        .await
                        {
                            Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                Ok(result) => {
                                    if let ResponseValue::Value(value) = result.value {
                                        let variant = match value.data.variant {
                                            Some(variant) => variant,
                                            None => return ("Can't parse product variant", None),
                                        };

                                        match value.data.product_card {
                                            Some(card) => ("", Some((card, variant))),
                                            None => ("Can't parse product info", None),
                                        }
                                    } else {
                                        ("Product scheme error", None)
                                    }
                                }
                                Err(_) => ("Product not found", None),
                            },
                            Err(err) => {
                                return (
                                    match err {
                                        ResponseStatus::Timeout => "Timeout",
                                        ResponseStatus::ConnectionError => "Connection Error",
                                    },
                                    None,
                                )
                            }
                        }
                    },
                    move |(err, result)| {
                        if err.is_empty() {
                            AddTasksMsg::Checked(result).into()
                        } else {
                            AddTasksMsg::Done(err.to_string()).into()
                        }
                    },
                );
            }
            AddTasksMsg::Checked(result) => {
                self.product = result;
                self.error = String::new();
                self.processing = false;
            }
            AddTasksMsg::Done(msg) => {
                self.error = msg;
                self.processing = false;
            }
            AddTasksMsg::Reset => self.reset(),
            _ => (),
        }

        Command::none()
    }

    pub fn view(&mut self, theme: &Theme, webhook: &Webhook) -> Element<Message> {
        let mut step_btn = Button::new(
            &mut self.step_btn,
            Text::new(if self.product.is_none() {
                "Search"
            } else {
                "Run"
            })
            .width(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Center),
        )
        .padding(8)
        .width(Length::Units(128))
        .style(theme.primary_btn());

        if !self.cod.is_empty() && !self.processing {
            if self.product.is_none() {
                step_btn = step_btn.on_press(AddTasksMsg::Check.into());
            } else {
                match &self.product {
                    Some((_, variant)) => {
                        if variant.sizes.len() == 1 || self.size.is_some() {
                            step_btn = step_btn.on_press(AddTasksMsg::Create.into());
                        }
                    }
                    None => {}
                }
            }
        }

        let cod_value = match &self.product {
            Some((card, variant)) => match &variant.name {
                Some(name) => format!("{} ({})", &card.name, name),
                None => card.name.clone(),
            },
            None => self.cod.clone(),
        };
        let mut inner = Column::new()
            .push(
                Text::new("Create tasks")
                    .size(32)
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center),
            )
            .push(Space::with_height(Length::Units(24)))
            .push(
                Text::new(&self.error)
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .color(theme.color_danger()),
            )
            .push(Space::with_height(Length::Units(
                if self.error.is_empty() { 0 } else { 8 },
            )))
            .push(Text::new(if self.product.is_none() {
                "PID"
            } else {
                "Product Name"
            }))
            .push(Space::with_height(Length::Units(8)))
            .push(
                TextInput::new(
                    &mut self.cod_input,
                    "Enter product ID",
                    &cod_value,
                    if self.product.is_none() {
                        |cod| AddTasksMsg::CodChange(cod).into()
                    } else {
                        |_| Message::None
                    },
                )
                .width(Length::FillPortion(2))
                .padding(8)
                .style(theme.text_input()),
            );

        match &self.product {
            Some((_, variant)) => {
                if variant.sizes.len() > 1 {
                    inner = inner
                        .push(Space::with_height(Length::Units(16)))
                        .push(Text::new("Size"))
                        .push(Space::with_height(Length::Units(8)))
                        .push(
                            PickList::new(
                                &mut self.size_pick,
                                variant.sizes_tags(),
                                self.size.clone(),
                                |variant| AddTasksMsg::SizeSelected(variant).into(),
                            )
                            .width(Length::Fill),
                        )
                }
            }
            None => (),
        }

        let mut content = Column::new();

        if webhook.id == 0 || webhook.token.is_empty() {
            content = content
                .push(
                    Container::new(
                        Row::new()
                            .push(icon(Icon::Alert).size(48).width(Length::Units(48)))
                            .push(Space::with_width(Length::Units(32)))
                            .push(
                                Text::new("Discord webhook is not configured!")
                                    .size(32)
                                    .height(Length::Units(48))
                                    .vertical_alignment(VerticalAlignment::Center),
                            ),
                    )
                    .padding(16)
                    .width(Length::Fill)
                    .style(theme.alert_box()),
                )
                .push(Space::with_height(Length::Units(32)));
        }

        content
            .push(
                Container::new(
                    inner
                        .push(Space::with_height(Length::Units(16)))
                        .push(Container::new(step_btn).width(Length::Fill).center_x())
                        .push(Space::with_height(Length::Units(24)))
                        .push(
                            Container::new(
                                Button::new(
                                    &mut self.reset_btn,
                                    Text::new("Reset")
                                        .width(Length::Fill)
                                        .horizontal_alignment(HorizontalAlignment::Center),
                                )
                                .on_press(AddTasksMsg::Reset.into())
                                .padding(8)
                                .width(Length::Units(128))
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
