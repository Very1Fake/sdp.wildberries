use iced::{executor, Application, Clipboard, Command, Element, Subscription};
use iced_native::{
    event::Event,
    keyboard::{Event as KeyEvent, KeyCode},
};

use themes::Theme;
use views::{auth, main, splash, View, ViewMessage};

mod icons;
mod themes;
mod views;

#[derive(Clone, Debug)]
pub enum Message {
    View(views::ViewMessage),
    Event(Event),
    Log(String),
    None,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Layout controller
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Layout {
    view: View,
    theme: Theme,
    explain: bool,
}

impl Application for Layout {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        use std::time::Duration;
        use tokio::time::sleep;

        (
            Layout::default(),
            Command::perform(
                async {
                    sleep(Duration::from_millis(1500)).await;
                    ViewMessage::Splash(splash::SplashMessage::Done)
                },
                Message::View,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("SDP")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        let mut result = Command::none();

        match message {
            Message::View(view) => match view {
                ViewMessage::Splash(msg) => splash::update(self, msg),
                ViewMessage::Auth(msg) => result = auth::update(self, msg),
                ViewMessage::Main(msg) => main::update(self, msg),
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
            Message::Log(msg) => println!("L: {}", msg),
            _ => (),
        }

        result
    }

    fn view(&mut self) -> Element<Self::Message> {
        let view = match &mut self.view {
            View::Splash => splash::view(&self.theme),
            View::Auth(state) => auth::view(state, &self.theme),
            View::Main(state) => main::view(state, &self.theme),
        };

        if self.explain {
            view.explain(self.theme.color_opposite())
        } else {
            view
        }
    }
}
