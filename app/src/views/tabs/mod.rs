use iced::{
    button, scrollable, Column, Container, Element, HorizontalAlignment, Length, Row, Rule, Space,
    Text,
};

use crate::{icons::icon, layout::Message, themes::Theme};

pub mod home;
pub mod settings;

////////////////////////////////////////////////////////////////////////////////////////////////////
// View
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct TabsViewState {
    pub tab_scroll: scrollable::State,
    pub content_scroll: scrollable::State,

    pub home_button: button::State,
    pub settings_button: button::State,
}

impl TabsViewState {
    pub fn view<'a>(
        &'a mut self,
        tab: &usize,
        tabs: &'a mut Vec<(String, Tab, button::State)>,
        theme: &Theme,
        scale: f64,
    ) -> Element<'a, Message> {
        let mut tab_bar = scrollable::Scrollable::new(&mut self.tab_scroll).height(Length::Fill);
        let mut current_tab: Option<&mut Tab> = None;
        let mut pinned_button: Option<button::Button<Message>> = None;

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
                        scrollable::Scrollable::new(&mut self.content_scroll).push(
                            Container::new(match current_tab {
                                Some(tab) => match tab {
                                    Tab::Home(ref mut state) => state.render(theme),
                                    Tab::Settings(ref mut state) => state.render(theme, scale),
                                },
                                None => Text::new(format!("Unknown tab: {}", tab)).into(),
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tabs
////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum Tab {
    Home(self::home::HomeTab),
    Settings(self::settings::SettingsTab),
}

impl Tab {
    pub fn icon(&self) -> &str {
        match *self {
            Tab::Home(..) => "h",
            Tab::Settings(..) => "s",
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Home(Default::default())
    }
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
