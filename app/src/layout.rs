use iced::{button, executor, Application, Clipboard, Command, Element, Subscription, Text};
use iced_native::{
    event::Event,
    keyboard::{Event as KeyEvent, KeyCode},
};

use crate::{
    themes::Theme,
    views::{splash, tabs::Tab, View, ViewMsg, ViewState},
};

#[derive(Clone, Debug)]
pub enum Message {
    View(View),
    ViewMsg(ViewMsg),
    Event(Event),
    Theme(Theme),
    Scale(f64),
    ResetAppearance,
    Tab(usize),
    None,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Layout controller
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Layout {
    explain: bool,
    scale: f64,

    view: View,
    state: ViewState,
    theme: Theme,

    tab: usize,
    tabs: Vec<(String, Tab, button::State)>,
}

impl Application for Layout {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        use std::time::Duration;
        use tokio::time::sleep;

        (
            Layout {
                scale: 1.0,
                tab: 1,
                tabs: vec![
                    (
                        String::from("Settings"),
                        Tab::Settings(Default::default()),
                        Default::default(),
                    ),
                    (String::from("Home"), Tab::default(), Default::default()),
                ],
                ..Default::default()
            },
            Command::perform(
                async {
                    sleep(Duration::from_millis(1500)).await;
                    View::Auth
                },
                Message::View,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("SDP")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        let mut result = Command::none();

        match message {
            Message::View(view) => {
                self.state = view.state();
                self.view = view;
            }
            Message::ViewMsg(view_msg) => match view_msg {
                ViewMsg::Auth(msg) => {
                    if let ViewState::Auth(ref mut state) = self.state {
                        result = state.update(msg)
                    }
                }
            },
            Message::Event(event) => match event {
                Event::Keyboard(key_event) => match key_event {
                    KeyEvent::KeyReleased {
                        key_code,
                        modifiers,
                    } => match key_code {
                        KeyCode::F3 if modifiers.alt => self.explain = !self.explain,
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            },
            Message::Theme(theme) => self.theme = theme,
            Message::Scale(scale) => self.scale = scale,
            Message::ResetAppearance => {
                self.theme = Theme::default();
                self.scale = 1.0
            }
            Message::Tab(tab) => self.tab = tab,
            Message::None => (),
        }

        result
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    fn view(&mut self) -> Element<Self::Message> {
        let view = match &self.view {
            View::Splash => splash::view(&self.theme),
            _ => match self.state {
                ViewState::Auth(ref mut state) => state.view(&self.theme),
                ViewState::Main(ref mut state) => {
                    state.view(&self.tab, &mut self.tabs, &self.theme, self.scale)
                }
                ViewState::None => Text::new("Unknown view state").into(),
            },
        };

        if self.explain {
            view.explain(self.theme.color_opposite())
        } else {
            view
        }
    }

    fn scale_factor(&self) -> f64 {
        self.scale
    }
}
