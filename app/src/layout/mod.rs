use iced::{executor, Application, Clipboard, Command, Element};

use themes::Theme;
use views::{auth, main, splash, View, ViewMessage};

mod icons;
mod themes;
mod views;

#[derive(Clone, Debug)]
pub enum Message {
    View(views::ViewMessage),
    Log(String),
    None,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Layout controller
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Layout {
    view: View,
    theme: Theme,
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
                view: View::Splash,
                theme: Theme::Light,
            },
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
            Message::Log(msg) => println!("L: {}", msg),
            _ => (),
        }

        result
    }

    fn view(&mut self) -> Element<Self::Message> {
        match &mut self.view {
            View::Splash => splash::view(&self.theme),
            View::Auth(state) => auth::view(state, &self.theme),
            View::Main(state) => main::view(state, &self.theme),
        }
    }
}
