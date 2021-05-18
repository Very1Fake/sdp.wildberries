use iced::{executor, Application, Clipboard, Command, Element};

use views::{auth, splash, View, ViewMessage};

mod views;

#[derive(Clone, Debug)]
pub enum Message {
    View(views::ViewMessage),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Layout controller
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Layout {
    view: View,
}

impl Application for Layout {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        use std::time::Duration;
        use tokio::time::sleep;

        (
            Layout { view: View::Splash },
            Command::perform(
                async {
                    sleep(Duration::from_millis(500)).await;
                    ViewMessage::Splash(splash::SplashMessage::Done)
                },
                Message::View,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Sellars Desktop Platform")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::View(view) => match view {
                ViewMessage::Splash(msg) => splash::update(self, msg),
                ViewMessage::Auth(msg) => auth::update(self, msg),
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        match &mut self.view {
            View::Splash => splash::view(),
            View::Auth(state) => auth::view(state),
        }
    }
}
