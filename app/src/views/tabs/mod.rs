use std::collections::BTreeMap;

use iced::{
    button, scrollable, Column, Container, Element, HorizontalAlignment, Length, Row, Rule, Space,
    Text,
};

use accounts::Account;
use add_tasks::AddTasksMsg;
use proxy::{Proxy, ProxyMode};
use settings::SettingsMsg;

use crate::{
    icons::{icon, Icon},
    layout::Message,
    logic::{activation::Activation, task::Task},
    themes::Theme,
};

pub mod accounts;
pub mod add_tasks;
pub mod home;
pub mod proxy;
pub mod settings;
pub mod tasks;

////////////////////////////////////////////////////////////////////////////////////////////////////
// View
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct TabsViewState {
    pub tab_scroll: scrollable::State,
    pub content_scroll: scrollable::State,
}

impl TabsViewState {
    pub fn view<'a>(
        &'a mut self,
        theme: &Theme,
        scale: f64,
        proxy_mode: &'a ProxyMode,
        activation: &Activation,
        w_id: u128,
        w_token: &String,
        tab: &usize,
        tabs: &'a mut Vec<(String, Tab, button::State)>,
        accounts: &'a mut Vec<Account>,
        proxies: &'a mut Vec<Proxy>,
        tasks: &'a mut BTreeMap<u64, Task>,
    ) -> Element<'a, Message> {
        let mut tab_bar = scrollable::Scrollable::new(&mut self.tab_scroll).height(Length::Fill);
        let mut current_tab: Option<&mut Tab> = None;
        let mut pinned_button: Option<button::Button<Message>> = None;
        let content_scroll = scrollable::Scrollable::new(&mut self.content_scroll);

        for (i, (_, t, b)) in tabs.iter_mut().enumerate() {
            if i == 0 {
                pinned_button = Some(
                    button::Button::new(b, icon(t.icon()).size(32).width(Length::Units(32)))
                        .on_press(Message::Tab(0))
                        .padding(16)
                        .style(theme.icon_button()),
                );
            } else {
                tab_bar = tab_bar.push(
                    button::Button::new(b, icon(t.icon()).size(32).width(Length::Units(32)))
                        .on_press(Message::Tab(i))
                        .padding(16)
                        .style(theme.icon_button()),
                );
            }

            if i == *tab {
                current_tab = Some(t);
            }
        }

        Container::<'a>::new(
            Row::new()
                .push(
                    Column::new()
                        .push(tab_bar)
                        .push(pinned_button.unwrap())
                        .height(Length::Fill)
                        .width(Length::Units(74)),
                )
                .push(
                    Container::new(
                        Container::new(match current_tab {
                            Some(tab) => match tab {
                                Tab::Home(ref mut state) => {
                                    content_scroll.push(state.view(theme, &activation)).into()
                                }
                                Tab::Settings(ref mut state) => content_scroll
                                    .push(state.view(
                                        theme,
                                        scale,
                                        proxy_mode,
                                        &activation.key,
                                        w_id,
                                        w_token,
                                    ))
                                    .into(),
                                Tab::Tasks(ref mut state) => state.view(theme, tasks),
                                Tab::AddTasks(ref mut state) => {
                                    content_scroll.push(state.view(theme)).into()
                                }
                                Tab::Accounts(ref mut state) => state.view(theme, accounts),
                                Tab::Proxy(ref mut state) => state.view(theme, proxies),
                            },
                            None => Text::new(format!("Unknown tab: {}", tab)).into(),
                        })
                        .max_width(920)
                        .center_x(),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .center_x(),
                ),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tabs
////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum Tab {
    Home(self::home::HomeTab),
    Settings(self::settings::SettingsTab),
    Tasks(self::tasks::TasksTab),
    AddTasks(self::add_tasks::AddTasksTab),
    Accounts(self::accounts::AccountsTab),
    Proxy(self::proxy::ProxyTab),
}

impl Tab {
    pub fn icon(&self) -> Icon {
        match *self {
            Tab::Home(..) => Icon::Home,
            Tab::Settings(..) => Icon::Settings,
            Tab::Tasks(..) => Icon::List,
            Tab::AddTasks(..) => Icon::Add,
            Tab::Accounts(..) => Icon::Account,
            Tab::Proxy(..) => Icon::Server,
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Home(Default::default())
    }
}

#[derive(Clone, Debug)]
pub enum TabMsg {
    SettingsMsg(SettingsMsg),
    AddTasksMsg(AddTasksMsg),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Custom elements
////////////////////////////////////////////////////////////////////////////////////////////////////

fn tab<'a>(name: &String) -> Column<'a, Message> {
    Column::new()
        .push(
            Text::new(name)
                .size(42)
                .width(Length::Fill)
                .horizontal_alignment(HorizontalAlignment::Center),
        )
        .push(Space::with_height(Length::Units(8)))
        .padding(16)
        .spacing(16)
}

fn section<'a>(name: &str, theme: &Theme) -> Column<'a, Message> {
    Column::new()
        .push(Text::new(name).size(32))
        .push(Rule::horizontal(theme.section_divider_spacing()).style(theme.section_divider()))
        .padding(8)
        .spacing(16)
}
