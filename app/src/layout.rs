use std::{
    collections::BTreeMap,
    fs::{remove_file, write, File},
    path::Path,
    time::Duration,
};

use iced::{button, executor, Application, Clipboard, Command, Element, Subscription, Text};
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
        task::{Delivery, Task, TaskMsg, TaskProgress},
    },
    settings::Settings,
    themes::Theme,
    views::{
        auth::{AuthViewState, Stage},
        splash,
        tabs::{
            accounts::{Account, AccountMsg, AccountState},
            proxy::{Proxy, ProxyMode, ProxyMsg, ProxyState},
            Tab, TabMsg,
        },
        View, ViewMsg, ViewState,
    },
    ACCOUNTS_FILE, LICENSE_FILE, PROXY_FILE, SETTINGS_FILE,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Message
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum Message {
    View(View),
    ViewMsg(ViewMsg),
    Tab(usize),
    TabMsg(TabMsg),

    Account(usize, AccountMsg),
    NewAccount,
    Proxy(usize, ProxyMsg),
    NewProxy,
    Task(u64, TaskMsg),
    AddTasks {
        pid: String,
        size_name: String,
        so: String,
        sv: String,
        option: String,
        delivery: Delivery,
    },
    TaskProgressed((u64, TaskProgress)),

    ActivationComplete {
        activation: Activation,
        token: String,
    },
    ActivationFailed {
        err: ActivationError,
        key: String,
    },
    Logout,

    Event(Event),
    Theme(Theme),
    ProxyMode(ProxyMode),
    Experimental(u8, bool),
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
            let content = self
                .accounts
                .iter()
                .filter(|account| {
                    if let AccountState::View { .. } = account.state {
                        true
                    } else {
                        false
                    }
                })
                .collect::<Vec<&Account>>();
            let mut old: Vec<Account> = Vec::new();
            load_file(ACCOUNTS_FILE, &mut old);

            if content != Vec::<&Account>::new() && content != old.iter().collect::<Vec<&Account>>()
            {
                to_writer(File::create(Path::new(ACCOUNTS_FILE)).unwrap(), &content).unwrap();
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

            if content != Vec::<&Proxy>::new() && content != old.iter().collect::<Vec<&Proxy>>() {
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
                async move {
                    match Activation::from_token(&token) {
                        Some(saved) => {
                            let key = saved.key.clone();
                            match saved.verify().await {
                                Ok((activation, token)) => {
                                    Message::ActivationComplete { activation, token }
                                }
                                Err(err) => Message::ActivationFailed { err, key },
                            }
                        }
                        None => {
                            sleep(Duration::from_secs(1)).await;
                            Message::View(View::Auth)
                        }
                    }
                },
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
            Message::Tab(tab) => self.tab = tab,
            Message::TabMsg(tab) => match tab {
                TabMsg::SettingsMsg(msg) => {
                    if let Tab::Settings(ref mut state) = self.tabs[0].1 {
                        state.update(msg, &mut self.settings)
                    }
                }
                TabMsg::AddTasksMsg(msg) => {
                    if let Tab::AddTasks(ref mut state) = self.tabs[3].1 {
                        result = state.update(msg)
                    }
                }
            },
            Message::Account(id, AccountMsg::Delete) => {
                self.accounts.remove(id);
            }
            Message::Account(id, msg) => self.accounts[id].update(msg),
            Message::NewAccount => self.accounts.push(Account::new()),
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
                pid,
                size_name,
                so,
                sv,
                option,
                delivery,
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
                let account_count = self
                    .accounts
                    .iter()
                    .filter(|a| {
                        if let AccountState::View { .. } = a.state {
                            a.active
                        } else {
                            false
                        }
                    })
                    .count();

                let iterator = self
                    .accounts
                    .iter()
                    .filter(|a| {
                        if let AccountState::View { .. } = a.state {
                            a.active
                        } else {
                            false
                        }
                    })
                    .rev()
                    .skip(if let ProxyMode::Strict = self.settings.proxy_mode {
                        account_count - proxy_count
                    } else {
                        0
                    });

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
                            pid.clone(),
                            size_name.clone(),
                            so.clone(),
                            sv.clone(),
                            option.clone(),
                            delivery.clone(),
                            a.login.clone(),
                            a.password.clone(),
                            self.settings.webhook.clone(),
                            (self.settings.limiter, self.settings.checker),
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
            Message::ActivationComplete { activation, token } => {
                self.activation = Some(activation);
                self.token = token;

                self.view = View::Main;
                self.state = View::Main.state();
            }
            Message::ActivationFailed { err, key } => {
                self.view = View::Auth;
                self.state = ViewState::Auth(AuthViewState {
                    key,
                    stage: Stage::Failed(err),
                    ..Default::default()
                });
            }
            Message::Logout => {
                self.token = String::new();
                match self.activation.take() {
                    Some(activation) => {
                        result = Command::perform(
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
                Event::Window(win_event) => match win_event {
                    WinEvent::CloseRequested => self.graceful_exit(),
                    _ => (),
                },
                _ => (),
            },
            Message::Theme(theme) => self.theme = theme,
            Message::ProxyMode(proxy_mode) => self.settings.proxy_mode = proxy_mode,
            Message::Experimental(flag, set) => match flag {
                0 => self.settings.limiter = set,
                1 => self.settings.checker = set,
                _ => {}
            },
            Message::ResetAppearance => {
                self.settings.theme = Theme::default();
                self.settings.scale = 1.0;
            }
            Message::None => (),
        }

        result
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subs = vec![iced_native::subscription::events().map(Message::Event)];
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
