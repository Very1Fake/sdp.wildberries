use iced::{scrollable, Container, Element, Length, Row, Text};

use super::{Message, View};
use crate::layout::{themes::Theme, Layout};

#[derive(Clone, Debug)]
pub enum MainMessage {
    TabChange(Tab),
}

#[derive(Default)]
pub struct MainViewState {
    tab: Tab,
    tab_scroll: scrollable::State,
    section_scroll: scrollable::State,
}

#[derive(Clone, Debug)]
enum Tab {
    Welcome,
    Settings,
    Info,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Welcome
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
    Container::new(
        Row::new()
            .push(
                scrollable::Scrollable::new(&mut state.tab_scroll)
                    .push(Text::new("Tab scroll here")),
            )
            .push(
                Container::new(
                    scrollable::Scrollable::new(&mut state.section_scroll)
                        .push(Text::new("Section scroll here")),
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
