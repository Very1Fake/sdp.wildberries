use iced::{button, scrollable, Column, Container, Element, Length, Row, Rule, Text};

use super::{Message, View, ViewMessage};
use crate::layout::{icons::icon, themes::Theme, Layout};

#[derive(Clone, Debug)]
pub enum MainMessage {
    TabChange(Tab),
}

impl Into<Message> for MainMessage {
    fn into(self) -> Message {
        Message::View(ViewMessage::Main(self))
    }
}

#[derive(Default)]
pub struct MainViewState {
    pub tab: Tab,

    pub tab_scroll: scrollable::State,
    pub content_scroll: scrollable::State,

    pub home_button: button::State,
    pub settings_button: button::State,
    pub tabs: Vec<(button::State, &'static str, String)>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tabs
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum Tab {
    Home,
    Settings,
    Module(String),
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Home
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// View rendering & processing
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn update(layout: &mut Layout, msg: MainMessage) {
    match msg {
        MainMessage::TabChange(tab) => {
            if let View::Main(ref mut state) = layout.view {
                state.tab = tab
            }
        }
    }
}

pub fn view<'a>(state: &'a mut MainViewState, theme: &Theme) -> Element<'a, Message> {
    let mut tabs = scrollable::Scrollable::new(&mut state.tab_scroll)
        .push(
            button::Button::new(
                &mut state.home_button,
                icon("h").size(32).width(Length::Units(32)),
            )
            .on_press(MainMessage::TabChange(Tab::Home).into())
            .padding(16)
            .style(theme.icon_button()),
        )
        .height(Length::Fill);

    for (s, i, n) in &mut state.tabs {
        tabs = tabs.push(
            button::Button::new(s, icon(i).size(32).width(Length::Units(32)))
                .on_press(MainMessage::TabChange(Tab::Module(n.clone())).into())
                .padding(16)
                .style(theme.icon_button()),
        );
    }

    Container::new(
        Row::new()
            .push(
                Column::new()
                    .push(tabs)
                    .push(
                        button::Button::new(
                            &mut state.settings_button,
                            icon("s").size(32).width(Length::Units(32)),
                        )
                        .on_press(MainMessage::TabChange(Tab::Settings).into())
                        .padding(16)
                        .style(theme.icon_button()),
                    )
                    .height(Length::Fill)
                    .width(Length::Units(74)),
            )
            .push(
                Container::new(
                    scrollable::Scrollable::new(&mut state.content_scroll).push(
                        Container::new(match state.tab {
                            Tab::Home => Column::new().push(
                                section("Welcome", theme)
                                    .push(Text::new("Some staff will be here soon...")),
                            ),
                            Tab::Settings => Column::new().push(
                                section("Settings", theme)
                                    .push(Text::new("Some settings options will be here soon")),
                            ),
                            Tab::Module(ref name) => Column::new().push(
                                section(&format!("Module \"{}\"", name), theme)
                                    .push(Text::new("And again, soon...")),
                            ),
                        })
                        .max_width(900)
                        .center_x(),
                    ),
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

#[inline]
fn section<'a>(name: &str, theme: &Theme) -> Column<'a, Message> {
    Column::new()
        .push(Text::new(name).size(32))
        .push(Rule::horizontal(theme.section_divider_spacing()).style(theme.section_divider()))
        .padding(16)
        .spacing(16)
}
