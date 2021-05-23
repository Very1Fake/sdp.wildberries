use super::section;
use crate::{layout::Message, themes::Theme};
use iced::{Element, Text};

#[derive(Default)]
pub struct HomeTab;

impl HomeTab {
    pub fn render<'a>(&mut self, theme: &Theme) -> Element<'a, Message> {
        section("Welcome", theme)
            .push(Text::new("Something useful will be here soon"))
            .into()
    }
}
