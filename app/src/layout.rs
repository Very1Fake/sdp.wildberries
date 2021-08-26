use std::{
    collections::BTreeMap,
    fs::{remove_file, write, File},
    path::Path,
    time::Duration,
};

use iced::{
    button, executor, time::every, Application, Clipboard, Command, Element, Subscription, Text,
};
use iced_native::{
    event::Event,
    keyboard::{Event as KeyEvent, KeyCode},
    window::Event as WinEvent,
};
use serde::Deserialize;
use serde_json::{from_reader, to_writer};
use tokio::time::sleep;

use crate::{
    logic::{
        activation::{Activation, ActivationError},
        models::{ProductCard, Settings, Size, Variant},
        task::{Task, TaskMsg, TaskProgress},
    },
    themes::Theme,
    views::{
        auth::{AuthViewState, Stage},
        splash,
        tabs::{
            accounts::Account,
            proxy::{Proxy, ProxyMode, ProxyMsg, ProxyState},
            Tab, TabMsg,
        },
        View, ViewMsg, ViewState,
    },
    ACCOUNTS_FILE, LICENSE_FILE, PROXY_FILE, SETTINGS_FILE,
};
use iced_native::event::Status;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Message
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum Message {
    View(View),
    ViewMsg(ViewMsg),
    Tab(usize),
    TabMsg(TabMsg),

    Proxy(usize, ProxyMsg),
    NewProxy,
    Task(u64, TaskMsg),
    AddTasks {
        card: ProductCard,
        variant: Variant,
        size: Size,
    },
    TaskProgressed((u64, TaskProgress)),

    Activation {
        activation: Activation,
        token: String,
    },
    ActivationError {
        err: ActivationError,
        key: String,
    },
    ActivationCheck,
    Logout,

    Event(Event),
    Theme(Theme),
    ProxyMode(ProxyMode),
    ExperimentalBool(u8, bool),
    ExperimentalNumber(u8, u64),
    ResetAppearance,

    None,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Layout controller
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Layout {
    settings: Settings,

    explain: bool,
    exit: bool,

    accounts: Vec<Account>,
    proxies: Vec<Proxy>,

    task_counter: u64,
    tasks: BTreeMap<u64, Task>,

    token: String,
    activation: Option<Activation>,

    view: View,
    state: ViewState,
    theme: Theme,

    tab: usize,
    tabs: Vec<(String, Tab, button::State)>,
}

impl Layout {
    fn graceful_exit(&mut self) {
        {
            let mut old: Vec<Account> = Vec::new();
            load_file(ACCOUNTS_FILE, &mut old);

            if (&self.accounts != &Vec::<Account>::new() || &old != &Vec::<Account>::new())
                && &self.accounts != &old
            {
                to_writer(
                    File::create(Path::new(ACCOUNTS_FILE)).unwrap(),
                    &self.accounts,
                )
                .unwrap();
            }
        }
        {
            let content = self
                .proxies
                .iter()
                .filter(|proxy| {
                    if let ProxyState::View { .. } = proxy.state {
                        true
                    } else {
                        false
                    }
                })
                .collect::<Vec<&Proxy>>();
            let mut old: Vec<Proxy> = Vec::new();
            load_file(PROXY_FILE, &mut old);

            if (content != Vec::<&Proxy>::new()
                || old.iter().collect::<Vec<&Proxy>>() != Vec::<&Proxy>::new())
                && content != old.iter().collect::<Vec<&Proxy>>()
            {
                to_writer(File::create(Path::new(PROXY_FILE)).unwrap(), &content).unwrap();
            }
        }

        {
            let content = self.settings.clone();
            let mut old = Settings::default();
            load_file(SETTINGS_FILE, &mut old);

            if content != Settings::default() && content != old {
                to_writer(File::create(Path::new(SETTINGS_FILE)).unwrap(), &content).unwrap();
            }
        }

        if !self.token.is_empty() {
            write(Path::new(LICENSE_FILE), &self.token).unwrap();
        }

        self.exit = true
    }

    async fn activation_check(token: &String) -> Message {
        match Activation::from_token(token) {
            Some(saved) => {
                let key = saved.key.clone();
                match saved.verify().await {
                    Ok((activation, token)) => Message::Activation { activation, token },
                    Err(err) => Message::ActivationError { err, key },
                }
            }
            None => {
                sleep(Duration::from_secs(1)).await;
                Message::View(View::Auth)
            }
        }
    }
}

impl Application for Layout {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut accounts = Vec::new();
        let mut proxies = Vec::new();
        let mut settings = Settings::default();

        load_file(ACCOUNTS_FILE, &mut accounts);
        load_file(PROXY_FILE, &mut proxies);
        load_file(SETTINGS_FILE, &mut settings);

        let token = Activation::load_token(LICENSE_FILE);

        (
            Layout {
                settings,
                accounts,
                proxies,
                token: token.clone(),
                tab: 1,
                tabs: vec![
                    (
                        String::from("Settings"),
                        Tab::Settings(Default::default()),
                        Default::default(),
                    ),
                    (String::from("Home"), Tab::default(), Default::default()),
                    (
                        String::from("Tasks"),
                        Tab::Tasks(Default::default()),
                        Default::default(),
                    ),
                    (
                        String::from("Create Tasks"),
                        Tab::AddTasks(Default::default()),
                        Default::default(),
                    ),
                    (
                        String::from("DataStore"),
                        Tab::Accounts(Default::default()),
                        Default::default(),
                    ),
                    (
                        String::from("Proxy"),
                        Tab::Proxy(Default::default()),
                        Default::default(),
                    ),
                ],
                ..Default::default()
            },
            Command::perform(
                async move { Layout::activation_check(&token).await },
                |msg| msg,
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
        match message {
            Message::View(view) => {
                self.state = view.state();
                self.view = view;
            }
            Message::ViewMsg(view_msg) => match view_msg {
                ViewMsg::Auth(msg) => {
                    if let ViewState::Auth(ref mut state) = self.state {
                        return state.update(msg);
                    }
                }
            },
            Message::Tab(tab) => self.tab = tab,
            Message::TabMsg(tab) => match tab {
                TabMsg::SettingsMsg(msg) => {
                    if let Tab::Settings(ref mut state) = self.tabs[0].1 {
                        state.update(msg, &mut self.settings)
                    }
                }
                TabMsg::AddTasksMsg(msg) => {
                    if let Tab::AddTasks(ref mut state) = self.tabs[3].1 {
                        return state.update(msg);
                    }
                }
                TabMsg::AccountsMsg(msg) => {
                    if let Tab::Accounts(ref mut state) = self.tabs[4].1 {
                        return state.update(msg, &mut self.accounts);
                    }
                }
            },
            Message::Proxy(id, ProxyMsg::Delete) => {
                self.proxies.remove(id);
            }
            Message::Proxy(id, msg) => self.proxies[id].update(msg),
            Message::NewProxy => self.proxies.push(Proxy::new()),
            Message::Task(id, TaskMsg::Delete) => {
                self.tasks.remove(&id);
            }
            Message::Task(id, msg) => self.tasks.get_mut(&id).unwrap().update(msg),
            Message::AddTasks {
                card,
                variant,
                size,
            } => {
                let mut proxies = self
                    .proxies
                    .iter()
                    .filter(|p| {
                        if let ProxyState::View { .. } = p.state {
                            p.active
                        } else {
                            false
                        }
                    })
                    .map(|p| p.address.clone())
                    .cycle()
                    .enumerate();
                let proxy_count = self
                    .proxies
                    .iter()
                    .filter(|p| {
                        if let ProxyState::View { .. } = p.state {
                            p.active
                        } else {
                            false
                        }
                    })
                    .count();
                let account_count = self.accounts.iter().filter(|a| a.active).count();

                let iterator = self.accounts.iter().filter(|a| a.active).rev().skip(
                    if let ProxyMode::Strict = self.settings.proxy_mode {
                        account_count - proxy_count
                    } else {
                        0
                    },
                );

                for a in iterator {
                    let p = if proxy_count == 0 {
                        None
                    } else {
                        match &self.settings.proxy_mode {
                            ProxyMode::Off => None,
                            ProxyMode::Repeat => Some(proxies.next().unwrap().1),
                            ProxyMode::Moderate => {
                                let (i, proxy) = proxies.next().unwrap();

                                if i < proxy_count {
                                    Some(proxy)
                                } else {
                                    None
                                }
                            }
                            ProxyMode::Strict => Some(proxies.next().unwrap().1),
                        }
                    };

                    self.task_counter += 1;

                    match self.tasks.insert(
                        self.task_counter,
                        Task::new(
                            self.task_counter,
                            p,
                            card.clone(),
                            variant.clone(),
                            size.clone(),
                            (a.phone.clone(), a.token.clone()),
                            self.settings.webhook.clone(),
                            (
                                self.settings.limiter,
                                self.settings.force,
                                self.settings.monitor,
                                self.settings.monitor_freq,
                            ),
                        ),
                    ) {
                        Some(_) => panic!(),
                        None => (),
                    }
                }

                self.tab = 2;
            }
            Message::TaskProgressed((uid, state)) => match self.tasks.get_mut(&uid) {
                Some(task) => task.progress = state,
                None => (),
            },
            Message::Activation { activation, token } => {
                self.activation = Some(activation);
                self.token = token;

                self.view = View::Main;
                self.state = View::Main.state();
            }
            Message::ActivationError { err, key } => {
                self.activation = None;
                self.view = View::Auth;
                self.state = ViewState::Auth(AuthViewState {
                    key,
                    stage: Stage::Failed(err),
                    ..Default::default()
                });
            }
            Message::ActivationCheck => {
                let token = self.token.clone();
                return Command::perform(
                    async move { Layout::activation_check(&token).await },
                    |msg| msg,
                );
            }
            Message::Logout => {
                self.token = String::new();
                match self.activation.take() {
                    Some(activation) => {
                        return Command::perform(
                            async move {
                                activation.deactivate().await;
                                View::Auth
                            },
                            Message::View,
                        )
                    }
                    None => {}
                };

                match remove_file(Path::new(LICENSE_FILE)) {
                    Ok(_) => (),
                    Err(_) => (),
                }

                self.view = View::Splash;
                self.state = View::Splash.state();
            }
            Message::Event(event) => match event {
                Event::Keyboard(event) => match event {
                    KeyEvent::KeyReleased {
                        key_code,
                        modifiers,
                    } => match key_code {
                        KeyCode::F3 if modifiers.alt => self.explain = !self.explain,
                        _ => (),
                    },
                    _ => (),
                },
                Event::Window(event) => match event {
                    WinEvent::CloseRequested => self.graceful_exit(),
                    _ => (),
                },
                _ => (),
            },
            Message::Theme(theme) => self.theme = theme,
            Message::ProxyMode(proxy_mode) => self.settings.proxy_mode = proxy_mode,
            Message::ExperimentalBool(flag, set) => match flag {
                0 => self.settings.limiter = set,
                1 => self.settings.force = set,
                2 => self.settings.monitor = set,
                _ => {}
            },
            Message::ExperimentalNumber(flag, num) => match flag {
                3 => self.settings.monitor_freq = num,
                _ => {}
            },
            Message::ResetAppearance => {
                self.settings.theme = Theme::default();
                self.settings.scale = 1.0;
            }
            Message::None => (),
        }

        return Command::none();
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subs = vec![iced_native::subscription::events_with(|e, s| match s {
            Status::Ignored => match &e {
                Event::Keyboard(event) => match event {
                    KeyEvent::KeyReleased { .. } => Some(e),
                    _ => None,
                },
                Event::Window(event) => match event {
                    WinEvent::CloseRequested => Some(e),
                    _ => None,
                },
                _ => None,
            },
            Status::Captured => None,
        })
        .map(Message::Event)];

        if self.activation.is_some() {
            subs.push(every(Duration::from_secs(1800)).map(|_i| Message::ActivationCheck));
        }

        subs.extend(self.tasks.values().map(Task::subscription));

        Subscription::batch(subs)
    }

    fn view(&mut self) -> Element<Self::Message> {
        let view = match &self.view {
            View::Splash => splash::view(&self.theme),
            _ => match self.state {
                ViewState::Auth(ref mut state) => state.view(&self.theme),
                ViewState::Main(ref mut state) => state.view(
                    &mut self.settings,
                    self.activation.as_ref().unwrap(),
                    &self.tab,
                    &mut self.tabs,
                    &mut self.accounts,
                    &mut self.proxies,
                    &mut self.tasks,
                ),
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
        self.settings.scale
    }

    fn should_exit(&self) -> bool {
        self.exit
    }
}

fn load_file<T: for<'de> Deserialize<'de>>(path: &str, object: &mut T) {
    let file = Path::new(path);
    if file.exists() {
        match from_reader::<File, T>(File::open(file).unwrap()) {
            Ok(result) => *object = result,
            Err(_) => (),
        }
    }
}
