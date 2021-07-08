use std::collections::BTreeMap;

use iced::{scrollable, Element, Scrollable, Text};

use crate::{layout::Message, logic::task::Task, themes::Theme};

use super::tab;

#[derive(Default)]
pub struct TasksTab {
    table_scroll: scrollable::State,
}

impl TasksTab {
    pub fn view<'a>(
        &'a mut self,
        theme: &Theme,
        tasks: &'a mut BTreeMap<u64, Task>,
    ) -> Element<'a, Message> {
        tab(&String::from("Tasks"))
            .push::<Element<'a, Message>>(if tasks.len() != 0 {
                tasks
                    .values_mut()
                    .rev()
                    .fold(
                        Scrollable::new(&mut self.table_scroll).spacing(8),
                        |table, task| {
                            let id = task.uid;
                            table.push(task.view(&theme).map(move |msg| Message::Task(id, msg)))
                        },
                    )
                    .into()
            } else {
                Text::new("No active tasks").into()
            })
            .into()
    }
}
