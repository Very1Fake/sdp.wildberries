use std::fmt::{Display, Formatter};

use iced::{
    button, pick_list, text_input, Align, Button, Color, Command, Container, Element, Length,
    PickList, Row, Text, TextInput,
};
use serde::Deserialize;
use serde_json::from_str;

use crate::{
    layout::Message,
    logic::{
        misc::{request, BanKind, RespStatus},
        task::{Delivery, Task},
    },
    themes::Theme,
};

use super::{tab, TabMsg};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Size
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct Size {
    is_last: bool,
    name: String,
    #[serde(rename = "product_option_value_id")]
    so: String,
    #[serde(rename = "option_value_id")]
    sv: String,
    #[serde(rename = "product_option_id")]
    option: String,
}

impl Display for Size {
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
    PIDChange(String),
    SizeSelected(Size),
    DeliverySelected(Delivery),
    Check,
    Checked(Vec<Size>),
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

    pid: String,
    size: Option<Size>,
    sizes: Vec<Size>,
    delivery: Option<Delivery>,

    pid_input: text_input::State,
    size_pick: pick_list::State<Size>,
    delivery_pick: pick_list::State<Delivery>,
    step_btn: button::State,
    reset_btn: button::State,
}

impl AddTasksTab {
    fn reset(&mut self) {
        self.error = String::new();
        self.size = None;
        self.sizes = Vec::new();
        self.delivery = None;
    }

    pub fn update(&mut self, msg: AddTasksMsg) -> Command<Message> {
        match msg {
            AddTasksMsg::PIDChange(pid) if pid.parse::<u128>().is_ok() || pid.is_empty() => {
                if self.size.is_some() {
                    self.size = None
                }
                if !self.sizes.is_empty() {
                    self.sizes = Vec::new()
                }
                self.pid = pid
            }
            AddTasksMsg::SizeSelected(size) => self.size = Some(size),
            AddTasksMsg::DeliverySelected(delivery) => self.delivery = Some(delivery),
            AddTasksMsg::Create => {
                let pid = self.pid.clone();
                let Size {
                    name,
                    so,
                    sv,
                    option,
                    ..
                } = self.size.clone().unwrap();
                let delivery = self.delivery.clone().unwrap();

                self.reset();

                return Command::perform(
                    async move { (pid, name, so, sv, option, delivery) },
                    |(pid, size_name, so, sv, option, delivery)| Message::AddTasks {
                        pid,
                        size_name,
                        so,
                        sv,
                        option,
                        delivery,
                    },
                );
            }
            AddTasksMsg::Check => {
                self.processing = true;
                let pid = self.pid.clone();

                return Command::perform(
                    async move {
                        match request(
                            &mut Task::init_client(None),
                            &format!("https://brandshop.ru/getproductsize/{}/", pid),
                            None,
                            "https://brandshop.ru/",
                            0,
                        )
                        .await
                        {
                            Ok(resp) => {
                                let sizes = from_str::<Vec<Size>>(&resp.body).unwrap();
                                if sizes.is_empty() {
                                    ("Product doesn't exists or sold out", Vec::new())
                                } else {
                                    ("", sizes)
                                }
                            }
                            Err(err) => {
                                return (
                                    match err {
                                        RespStatus::Timeout => "Timeout",
                                        RespStatus::ConnectionError => "Connection Error",
                                        RespStatus::ProtectionBan(kind) => match kind {
                                            BanKind::Variti => "Variti Ban",
                                            BanKind::DDOSGuard => "Protection Ban",
                                        },
                                    },
                                    Vec::new(),
                                )
                            }
                        }
                    },
                    move |(err, sizes)| {
                        if err.is_empty() {
                            AddTasksMsg::Checked(sizes).into()
                        } else {
                            AddTasksMsg::Done(err.to_string()).into()
                        }
                    },
                );
            }
            AddTasksMsg::Checked(sizes) => {
                self.sizes = sizes;
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

        return Command::none();
    }

    pub fn view(&mut self, theme: &Theme) -> Element<Message> {
        let mut step_btn = Button::new(
            &mut self.step_btn,
            if self.sizes.is_empty() {
                Text::new("Check")
            } else {
                Text::new("Create")
            },
        )
        .padding(8)
        .style(theme.primary_btn());
        if !self.pid.is_empty() && !self.processing {
            if self.sizes.is_empty() {
                step_btn = step_btn.on_press(AddTasksMsg::Check.into());
            } else {
                if self.size.is_some() && self.delivery.is_some() {
                    step_btn = step_btn.on_press(AddTasksMsg::Create.into());
                }
            }
        }

        let mut content = tab(&String::from("Create tasks"))
            .push(
                Text::new(&self.error)
                    .height(if !self.error.is_empty() {
                        Length::Shrink
                    } else {
                        Length::Units(0)
                    })
                    .color(Color::from_rgb(1.0, 0.0, 0.0)),
            )
            .push(
                Row::new()
                    .push(Text::new("Product ID").width(Length::FillPortion(1)))
                    .push(
                        TextInput::new(&mut self.pid_input, "PID", &self.pid, |pid| {
                            AddTasksMsg::PIDChange(pid).into()
                        })
                        .width(Length::FillPortion(2))
                        .padding(8)
                        .style(theme.text_input()),
                    )
                    .align_items(Align::Center),
            );

        if !self.sizes.is_empty() {
            content = content
                .push::<Element<Message>>(
                    Row::new()
                        .push(Text::new("Size").width(Length::FillPortion(1)))
                        .push(
                            PickList::new(
                                &mut self.size_pick,
                                self.sizes.clone(),
                                self.size.clone(),
                                |size| AddTasksMsg::SizeSelected(size).into(),
                            )
                            .width(Length::FillPortion(2)),
                        )
                        .align_items(Align::Center)
                        .into(),
                )
                .push::<Element<Message>>(
                    Row::new()
                        .push(Text::new("Delivery").width(Length::FillPortion(1)))
                        .push(
                            PickList::new(
                                &mut self.delivery_pick,
                                &Delivery::ALL[..],
                                self.delivery.clone(),
                                |delivery| AddTasksMsg::DeliverySelected(delivery).into(),
                            )
                            .width(Length::FillPortion(2)),
                        )
                        .align_items(Align::Center)
                        .into(),
                )
        }

        content
            .push(Container::new(step_btn).width(Length::Fill).center_x())
            .push(
                Container::new(
                    Button::new(&mut self.reset_btn, Text::new("Reset"))
                        .on_press(AddTasksMsg::Reset.into())
                        .padding(8)
                        .style(theme.primary_btn()),
                )
                .width(Length::Fill)
                .center_x(),
            )
            .into()
    }
}
