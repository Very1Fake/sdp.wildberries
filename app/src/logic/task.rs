use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    sync::Arc,
    time::{Duration, Instant},
};

use iced::{
    button, Align, Button, Color, Column, Container, Element, Length, Row, Subscription, Text,
};
use iced_futures::futures::stream;
use iced_native::subscription::Recipe;
use reqwest::{redirect::Policy, Client, Proxy, StatusCode};
use serde::Deserialize;
use serde_json::{from_str, json};
use tokio::time::sleep;

use crate::{
    icons::{icon, Icon},
    layout::Message,
    settings::Webhook,
    themes::Theme,
    H_USER_AGENT, VERSION,
};

use super::misc::{rand_millis, request, retrieve};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Checker
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct Checker {
    #[serde(default, alias = "Success")]
    success: String,
    #[serde(default, alias = "Warning")]
    warning: String,
    #[serde(default, alias = "Error")]
    error: bool,
}

impl Checker {
    fn ok(&self) -> bool {
        self.warning.is_empty() && !self.error
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Cart
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct Product {
    key: String,
}

#[derive(Deserialize)]
struct Cart {
    #[serde(default)]
    products: Vec<Product>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Delivery
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Delivery {
    Pickup,
    Flat,
    Shipard,
    EMS,
    DHL,
}

impl Delivery {
    pub const ALL: [Delivery; 5] = [
        Delivery::Pickup,
        Delivery::Flat,
        Delivery::Shipard,
        Delivery::EMS,
        Delivery::DHL,
    ];

    fn to_str(&self) -> &str {
        match *self {
            Delivery::Pickup => "pickup",
            Delivery::Flat => "flat",
            Delivery::Shipard => "shipard",
            Delivery::EMS => "ems",
            Delivery::DHL => "dhl",
        }
    }
}

impl Display for Delivery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Delivery::Pickup => "Pickup",
                Delivery::Flat => "Courier delivery (Moscow, within MKAD)",
                Delivery::Shipard => "Russian Post",
                Delivery::EMS => "EMS (Russian Post)",
                Delivery::DHL => "DHL",
            }
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Task
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Task {
    pub uid: u64,

    pub proxy: Option<String>,
    pub pid: String,
    pub size_name: String,
    pub so: String,
    pub sv: String,
    pub option: String,
    pub delivery: Delivery,

    pub login: String,
    pub password: String,
    pub webhook: Webhook,
    pub flags: (bool, bool),

    pub progress: TaskProgress,

    link: Arc<()>,
    state: TaskState,
}

impl Task {
    pub fn new(
        uid: u64,
        proxy: Option<String>,
        pid: String,
        size_name: String,
        so: String,
        sv: String,
        option: String,
        delivery: Delivery,
        login: String,
        password: String,
        webhook: Webhook,
        flags: (bool, bool),
    ) -> Task {
        Task {
            uid,
            proxy,
            pid,
            size_name,
            so,
            sv,
            option,
            delivery,
            login,
            password,
            webhook,
            flags,
            progress: TaskProgress::Start,
            link: Arc::new(()),
            state: TaskState::default(),
        }
    }

    pub fn init_client(proxy: Option<String>) -> Client {
        let mut tls = rustls::ClientConfig::new();
        tls.set_protocols(&["http/1.1".into()]);
        tls.root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        tls.key_log = Arc::new(rustls::KeyLogFile::new());

        let mut client = Client::builder()
            .tcp_keepalive(Some(Duration::from_secs(4)))
            .use_preconfigured_tls(tls)
            .redirect(Policy::custom(|attempt| {
                if attempt.previous().len() > 10 {
                    attempt.error("too many redirects")
                } else if attempt.url().domain() == Some("brandshop.ru")
                    && attempt.url().path() == "/account/"
                {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .timeout(Duration::from_secs(8))
            .cookie_store(true)
            .user_agent(H_USER_AGENT)
            .gzip(true)
            .https_only(true)
            // .http2_adaptive_window(true)
            // .http2_initial_stream_window_size(Some(131072))
            // .http2_max_frame_size(16384)
            .http1_title_case_headers();

        match proxy {
            Some(address) => {
                client = client.proxy(Proxy::all(format!("http://{}", address)).unwrap())
            }
            None => (),
        }

        client.build().unwrap()
    }

    pub fn update(&mut self, msg: TaskMsg) {
        match msg {
            TaskMsg::State(state) => self.state = state,
            TaskMsg::Retry => self.progress = TaskProgress::Start,
            TaskMsg::Delete => (),
        }
    }

    pub fn view(&mut self, theme: &Theme) -> Element<TaskMsg> {
        match self.state {
            TaskState::Collapsed { ref mut expand_btn } => Container::new(
                Row::new()
                    .push(Text::new(&self.pid).width(Length::FillPortion(2)))
                    .push(Text::new(&self.size_name).width(Length::FillPortion(2)))
                    .push(Text::new(&self.login).width(Length::FillPortion(5)))
                    .push(
                        Text::new(self.progress.to_str())
                            .width(Length::FillPortion(4))
                            .color(self.progress.color()),
                    )
                    .push(
                        Button::new(expand_btn, icon(Icon::ArrowDown))
                            .on_press(TaskMsg::State(TaskState::Expanded {
                                collapse_btn: button::State::new(),
                                retry_btn: button::State::new(),
                                delete_btn: button::State::new(),
                            }))
                            .width(Length::Shrink)
                            .padding(8)
                            .style(theme.primary_btn()),
                    )
                    .align_items(Align::Center)
                    .padding(8)
                    .spacing(8),
            )
            .style(theme.card())
            .into(),
            TaskState::Expanded {
                ref mut collapse_btn,
                ref mut retry_btn,
                ref mut delete_btn,
            } => {
                let mut retry_btn = Button::new(retry_btn, icon(Icon::Reload))
                    .width(Length::Shrink)
                    .padding(8)
                    .style(theme.primary_btn());

                if let TaskProgress::Error(..) = self.progress {
                    retry_btn = retry_btn.on_press(TaskMsg::Retry);
                }

                Container::new(
                    Column::new()
                        .push(
                            Row::new()
                                .push(Text::new(&self.pid).width(Length::FillPortion(2)))
                                .push(Text::new(&self.size_name).width(Length::FillPortion(2)))
                                .push(Text::new(&self.login).width(Length::FillPortion(5)))
                                .push(
                                    Text::new(self.progress.to_str())
                                        .width(Length::FillPortion(4))
                                        .color(self.progress.color()),
                                )
                                .push(
                                    Button::new(collapse_btn, icon(Icon::ArrowUp))
                                        .on_press(TaskMsg::State(TaskState::Collapsed {
                                            expand_btn: button::State::new(),
                                        }))
                                        .width(Length::Shrink)
                                        .padding(8)
                                        .style(theme.primary_btn()),
                                )
                                .align_items(Align::Center)
                                .padding(8)
                                .spacing(8),
                        )
                        .push(
                            Row::new()
                                .push(
                                    Text::new(format!("{}", &self.delivery))
                                        .width(Length::FillPortion(1)),
                                )
                                .push(
                                    Text::new(match self.proxy {
                                        Some(ref proxy) => proxy,
                                        None => "No Proxy",
                                    })
                                    .width(Length::FillPortion(3)),
                                )
                                .push(retry_btn)
                                .push(
                                    Button::new(delete_btn, icon(Icon::Delete))
                                        .on_press(TaskMsg::Delete)
                                        .width(Length::Shrink)
                                        .padding(8)
                                        .style(theme.danger_btn()),
                                )
                                .align_items(Align::Center)
                                .padding(8)
                                .spacing(8),
                        ),
                )
                .style(theme.card())
                .into()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.progress {
            TaskProgress::Start
            | TaskProgress::WarmingUp
            | TaskProgress::Processing
            | TaskProgress::Success(_)
            | TaskProgress::PostSuccess(_) => Subscription::from_recipe(Background {
                uid: self.uid,
                state: BackgroundState {
                    pid: self.pid.clone(),
                    so: self.so.clone(),
                    sv: self.sv.clone(),
                    option: self.option.clone(),
                    login: self.login.clone(),
                    password: self.password.clone(),
                    delivery: self.delivery.clone(),
                    webhook: self.webhook.clone(),
                    flags: self.flags.clone(),
                    client: Task::init_client(self.proxy.clone()),
                    progress: TaskProgress::Start,
                    step: BackgroundStep::Start,
                    substep: 0,
                    start: Instant::now(),
                    stopped: false,
                    link: self.link.clone(),
                },
            })
            .map(Message::TaskProgressed),
            _ => Subscription::none(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TaskMsg {
    State(TaskState),
    Retry,
    Delete,
}

#[derive(Clone, Debug)]
pub enum TaskState {
    Collapsed {
        expand_btn: button::State,
    },
    Expanded {
        collapse_btn: button::State,
        retry_btn: button::State,
        delete_btn: button::State,
    },
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Collapsed {
            expand_btn: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Task Progress
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Clone, Debug)]
pub enum TaskProgress {
    Start,
    WarmingUp,
    Processing,
    Success(Option<String>),
    PostSuccess(Option<String>),
    Complete,
    Failed(String),
    Error(String),
}

impl TaskProgress {
    fn to_str(&self) -> String {
        match self {
            TaskProgress::Start => String::from("Starting"),
            TaskProgress::WarmingUp => String::from("Warming Up"),
            TaskProgress::Processing => String::from("Processing"),
            TaskProgress::Success(content) => match content {
                Some(ref text) => text.clone(),
                None => String::from("Success"),
            },
            TaskProgress::PostSuccess(content) => match content {
                Some(ref text) => text.clone(),
                None => String::from("Post Success Procedure"),
            },
            TaskProgress::Complete => String::from("Complete"),
            TaskProgress::Failed(msg) => {
                if msg.is_empty() {
                    String::from("Failed")
                } else {
                    format!("Failed: {}", msg)
                }
            }
            TaskProgress::Error(msg) => {
                if msg.is_empty() {
                    String::from("Error")
                } else {
                    format!("Error: {}", msg)
                }
            }
        }
    }

    fn color(&self) -> Color {
        match *self {
            TaskProgress::Start => Color::BLACK,
            TaskProgress::WarmingUp => Color::from_rgb(1.0, 0.671, 0.0),
            TaskProgress::Processing => Color::from_rgb(0.867, 0.173, 0.0),
            TaskProgress::PostSuccess(_) => Color::from_rgb(0.188, 0.31, 0.996),
            TaskProgress::Success(_) => Color::from_rgb(0.0, 0.784, 0.325),
            TaskProgress::Complete => Color::from_rgb(0.392, 0.867, 0.09),
            TaskProgress::Failed(_) | TaskProgress::Error(_) => Color::from_rgb(0.835, 0.0, 0.0),
        }
    }
}

impl Default for TaskProgress {
    fn default() -> Self {
        TaskProgress::Start
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Task Background
////////////////////////////////////////////////////////////////////////////////////////////////////

struct Background<T> {
    uid: T,
    state: BackgroundState,
}

impl<H, I, T> Recipe<H, I> for Background<T>
where
    T: 'static + Hash + Copy + Send + Display,
    H: Hasher,
{
    type Output = (T, TaskProgress);

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.uid.hash(state);
    }

    fn stream(self: Box<Self>, _input: stream::BoxStream<I>) -> stream::BoxStream<Self::Output> {
        let uid = self.uid;

        Box::pin(stream::unfold(self.state, move |mut state| async move {
            async fn checkout(client: &mut Client, tier: &'static str) -> LoopAction {
                match request(
                    client,
                    "https://brandshop.ru/checkout/",
                    None,
                    "https://brandshop.ru/",
                    rand_millis(28..=30),
                )
                .await
                {
                    Ok(resp) => {
                        if resp.status != StatusCode::OK {
                            LoopAction::Error(String::from("Something went wrong"), true)
                        } else {
                            LoopAction::Continue
                        }
                    }
                    Err(err) => LoopAction::Error(err.task_progress(tier), false),
                }
            }
            if state.stopped {
                return None;
            }

            loop {
                if Arc::strong_count(&state.link) == 1 {
                    println!("Task dropped");
                    return None;
                }

                let mut action = LoopAction::Continue;

                match state.step {
                    BackgroundStep::Start => {
                        action = LoopAction::Move(BackgroundStep::WarmingUp, None);
                    }
                    BackgroundStep::WarmingUp => match state.substep {
                        0 => match request(
                            &mut state.client,
                            "https://brandshop.ru/",
                            None,
                            "https://google.ru/",
                            0,
                        )
                            .await
                        {
                            Ok(_) => {},
                            Err(err) => action = LoopAction::Error(err.task_progress("A"), false),
                        },
                        1 => match request(
                            &mut state.client,
                            "https://brandshop.ru/login/",
                            Some(&[
                                ("email", state.login.as_str()),
                                ("password", state.password.as_str()),
                                ("redirect", ""),
                            ]),
                            "https://brandshop.ru/login/",
                            if state.flags.0 {
                                rand_millis(45..=50)
                            } else {
                                0
                            },
                        )
                            .await
                        {
                            Ok(resp) => {
                                if resp.status == StatusCode::FOUND {
                                    match request(
                                        &mut state.client,
                                        "https://brandshop.ru/account/",
                                        None,
                                        "https://brandshop.ru/login/",
                                        if state.flags.0 {
                                            rand_millis(24..=26)
                                        } else {
                                            0
                                        },
                                    )
                                        .await
                                    {
                                        Ok(_) => {}
                                        Err(err) => action = LoopAction::Error(err.task_progress("B/2"), false),
                                    }
                                } else {
                                    action = LoopAction::Error(String::from("Bad credentials"), true);
                                }
                            }
                            Err(err) => action = LoopAction::Error(err.task_progress("B/1"), false),
                        },
                        2 => match request(
                            &mut state.client,
                            "https://brandshop.ru/xhr/cart/",
                            None,
                            "https://brandshop.ru/",
                            if state.flags.0 {
                                rand_millis(28..=30)
                            } else {
                                0
                            },
                        )
                            .await
                        {
                            Ok(resp) => {
                                let cart = from_str::<Cart>(&resp.body).unwrap();
                                if !cart.products.is_empty() {
                                    for product in cart.products {
                                        match request(
                                            &mut state.client,
                                            "https://brandshop.ru/cart/",
                                            Some(&[(
                                                format!("quantity[{}]", product.key).as_str(),
                                                "0",
                                            )]),
                                            "https://brandshop.ru/",
                                            if state.flags.0 {
                                                rand_millis(9..=11)
                                            } else {
                                                0
                                            },
                                        )
                                            .await
                                        {
                                            Ok(_) => (),
                                            Err(_) => (),
                                        }
                                    }
                                }

                                action = LoopAction::Move(BackgroundStep::Processing {
                                    link: String::new(),
                                }, None);
                            }
                            Err(err) => action = LoopAction::Error(err.task_progress("C/1"), false),
                        },
                        _ => {}
                    },
                    BackgroundStep::Processing { ref mut link } => match state.substep {
                        0 => match request(
                            &mut state.client,
                            "https://brandshop.ru/index.php?route=checkout/cart/add",
                            Some(&[
                                ("quantity", "1"),
                                ("product_id", &state.pid),
                                ("option_value_id", &state.sv),
                                (format!("option[{}]", &state.option).as_str(), &state.so),
                            ]),
                            "https://brandshop.ru/",
                            if state.flags.0 {
                                rand_millis(45..=50)
                            } else {
                                0
                            },
                        )
                            .await
                        {
                            Ok(resp) => {
                                if resp.body == "[]" {
                                    action = LoopAction::Error(String::from(
                                        "Can't add product to cart",
                                    ), true);
                                }
                            }
                            Err(err) => action = LoopAction::Error(err.task_progress("D"), false),
                        },
                        1 => action = checkout(&mut state.client, "E").await,
                        2 => match request(
                            &mut state.client,
                            "https://brandshop.ru/index.php?route=checkout/checkout/setshippingmethod",
                            Some(&[("shipping_method", state.delivery.to_str())]),
                            "https://brandshop.ru/checkout/",
                            if state.flags.0 {
                                rand_millis(24..=26)
                            } else {
                                0
                            },
                        )
                            .await
                        {
                            Ok(resp) => {
                                if !from_str::<Checker>(&resp.body).unwrap().ok() {
                                    action = LoopAction::Error(String::from(
                                        "Can't select delivery method",
                                    ), true);
                                }
                            }
                            Err(err) => action = LoopAction::Error(err.task_progress("F"), false),
                        },
                        3 => match request(
                            &mut state.client,
                            "https://brandshop.ru/index.php?route=checkout/checkout/setpaymentmethod",
                            Some(&[("payment_method", "payture")]),
                            "https://brandshop.ru/checkout/",
                            if state.flags.0 {
                                rand_millis(28..=30)
                            } else {
                                0
                            },
                        )
                            .await
                        {
                            Ok(resp) => {
                                if !from_str::<Checker>(&resp.body).unwrap().ok() {
                                    action = LoopAction::Error(String::from("Can't select payment method"), true);
                                }
                            }
                            Err(err) => action = LoopAction::Error(err.task_progress("G"), false),
                        },
                        4 => action = checkout(&mut state.client, "H").await,
                        5 => match request(
                            &mut state.client,
                            "https://brandshop.ru/xhr/payture/",
                            Some(&[]),
                            "https://brandshop.ru/checkout/",
                            if state.flags.0 {
                                rand_millis(10..=15)
                            } else {
                                0
                            },
                        )
                            .await
                        {
                            Ok(resp) => {
                                let result = from_str::<Checker>(&resp.body).unwrap();
                                if result.ok() {
                                    *link = result.success;
                                } else {
                                    action = LoopAction::Error(String::from(
                                        "Can't retrieve payment link",
                                    ), false);
                                }
                            }
                            Err(err) => action = LoopAction::Error(err.task_progress("I"), false),
                        },
                        6 => match state
                            .client
                            .get(link.clone())
                            .header("Referer", "https://brandshop.ru/")
                            .send()
                            .await
                        {
                            Ok(resp) => {
                                if resp.headers().get("Transfer-Encoding").is_some() {
                                    let body = resp.text().await.unwrap();
                                    action = LoopAction::Move(BackgroundStep::Success {
                                        link: link.clone(),
                                        total: match retrieve(&body, r#"al" value=""#, "\"") {
                                            Some(sum) => sum,
                                            None => String::from("-"),
                                        },
                                        order: match retrieve(&body, r#"а №"#, "\"") {
                                            Some(oid) => oid,
                                            None => String::from("-"),
                                        },
                                    }, Some(String::from("Sending link")));
                                } else {
                                    action =
                                        LoopAction::Error(String::from("Payment link broken"), true);
                                }
                            }
                            Err(err) => {
                                action = if err.is_timeout() {
                                    LoopAction::Error(String::from("Timeout"), false)
                                } else {
                                    LoopAction::Error(String::from(
                                        "Connection error. Try again later",
                                    ), false)
                                }
                            }
                        },
                        _ => {}
                    },
                    BackgroundStep::Success {
                        ref link,
                        ref total,
                        ref order,
                    } => {
                        let resp = Task::init_client(None)
                            .post(format!("https://discord.com/api/webhooks/{}/{}", state.webhook.id, state.webhook.token))
                            .json(&json!({
                            "username": "SDP",
                            "embeds": [{
                                "title": "Payment link generated",
                                "description": "Click the link below to complete your order.",
                                "color": 3166206,
                                "fields": [
                                    {"name": "Link", "value": format!("||{}||", link)},
                                    {"name": "Site", "value": "Brandshop", "inline": true},
                                    {"name": "Account", "value": format!("||{}||", &state.login), "inline": true},
                                    {"name": "\u{200B}", "value": "\u{200B}", "inline": true},
                                    {"name": "Order ID", "value": format!("||{}||", order), "inline": true},
                                    {"name": "Product ID", "value": &state.pid, "inline": true},
                                    {"name": "Total", "value": total, "inline": true},
                                    {"name": "Elapsed", "value": format!("{:.3} sec", state.start.elapsed().as_secs_f32())},
                                ],
                                "footer": {
                                    "text": format!("SDP Alpha (v{})", VERSION),
                                    "icon_url": "https://en.gravatar.com/userimage/182691345/3ce2e13566d08dd3ae6513f6b0404900.png"
                                }
                            }]
                        }))
                            .send()
                            .await
                            .unwrap();

                        if resp.status() == StatusCode::NO_CONTENT {
                            if state.flags.1 {
                                action = LoopAction::Move(BackgroundStep::PostSuccess {
                                    order: order.clone(),
                                    attempt: 0,
                                }, Some(String::from("Checker: Starting")));
                            } else {
                                action = LoopAction::Complete;
                            }
                        } else {
                            action = LoopAction::Error(String::from("Webhook error"), false);
                        }
                    }
                    BackgroundStep::PostSuccess {
                        ref order,
                        ref mut attempt,
                    } => {
                        match state.substep {
                            0 => {
                                sleep(Duration::from_secs(3)).await;
                                action = LoopAction::Break(Some(
                                    String::from("Checker: Waiting"),
                                ));
                            }
                            1 => {
                                if *attempt == 120 {
                                    action = LoopAction::Error(String::from(
                                        "Checker: order not detected",
                                    ), true);
                                } else {
                                    sleep(Duration::from_secs(30)).await;
                                    action = LoopAction::Break(Some(String::from(
                                        "Checker: Scanning",
                                    )));
                                }
                            }
                            2 => {
                                match request(
                                    &mut state.client,
                                    "https://brandshop.ru/order/",
                                    Some(&[]),
                                    "https://brandshop.ru/account/",
                                    if state.flags.0 {
                                        rand_millis(10..=15)
                                    } else {
                                        0
                                    },
                                )
                                    .await
                                {
                                    Ok(resp) => {
                                        if let Some(order_body) = retrieve(&resp.body, &order, "</li>")
                                        {
                                            let mut ok = false;

                                            if let Some(date) =
                                            retrieve(&order_body, "sm-3\">", "</div>")
                                            {
                                                if let Some(total) =
                                                retrieve(&order_body, "-sm\">", " <em")
                                                {
                                                    if let Some(status) = retrieve(
                                                        &order_body,
                                                        "em></div>
                                <div class=\"col col-2 col-sm-12 hidden-sm\">",
                                                        "</div>",
                                                    ) {
                                                        ok = true;

                                                        let resp = Task::init_client(None)
                                                            .post(format!("https://discord.com/api/webhooks/{}/{}", state.webhook.id, state.webhook.token))
                                                            .json(&json!({
                                                            "username": "SDP",
                                                            "embeds": [{
                                                                "title": "Order processed",
                                                                "color": 16776960,
                                                                "fields": [
                                                                    {"name": "Site", "value": "Brandshop"},
                                                                    {"name": "Account","value": format!("||{}||", &state.login),"inline": true},
                                                                    {"name": "Order ID","value": format!("||{}||", order),"inline": true},
                                                                    {"name": "Product Id","value": &state.pid},
                                                                    {"name": "Status","value": &status,"inline": true},
                                                                    {"name": "Total","value": &total,"inline": true},
                                                                    {"name": "Date","value": &date}
                                                                ],
                                                                "footer": {
                                                                    "text": format!("SDP Alpha (v{})", VERSION),
                                                                    "icon_url": "https://en.gravatar.com/userimage/182691345/3ce2e13566d08dd3ae6513f6b0404900.png"
                                                                }
                                                            }]
                                                        }))
                                                            .send()
                                                            .await
                                                            .unwrap();

                                                        if resp.status() == StatusCode::NO_CONTENT {
                                                            action = LoopAction::Complete;
                                                        } else {
                                                            action = LoopAction::Error(
                                                                String::from("Checker: Webhook error"),
                                                                false
                                                            );
                                                        }
                                                    }
                                                }
                                            }

                                            if ok {
                                                action = LoopAction::Error(
                                                    String::from("Checker: Can't retrieve order data"),
                                                    true
                                                )
                                            }
                                        } else {
                                            state.substep = 0;
                                            *attempt += 1;
                                            action = LoopAction::Break(Some(
                                                String::from("Checker: Waiting"),
                                            ));
                                        }
                                    }
                                    Err(err) => action = LoopAction::Error(err.task_progress("J"), false),
                                }
                            }
                            _ => action = LoopAction::Error(String::from("Checker down"), false)
                        }
                    }
                }

                match action {
                    LoopAction::Continue => state.substep += 1,
                    LoopAction::Error(ref msg, fail) => {
                        state.progress = if fail {
                            TaskProgress::Failed(String::from(msg))
                        } else {
                            TaskProgress::Error(String::from(msg))
                        }
                    }
                    LoopAction::Break(ref msg) => {
                        let cloned = msg.clone();
                        state.substep += 1;

                        match state.progress {
                            TaskProgress::Success(_) => {
                                state.progress = TaskProgress::Success(cloned)
                            }
                            TaskProgress::PostSuccess(_) => {
                                state.progress = TaskProgress::PostSuccess(cloned)
                            }
                            TaskProgress::Failed(_) => match cloned {
                                Some(val) => state.progress = TaskProgress::Failed(val),
                                None => {}
                            },
                            TaskProgress::Error(_) => match cloned {
                                Some(val) => state.progress = TaskProgress::Error(val),
                                None => {}
                            },
                            _ => {}
                        }
                    }
                    LoopAction::Move(ref step, ref msg) => {
                        state.progress = match step {
                            BackgroundStep::Start => TaskProgress::Start,
                            BackgroundStep::WarmingUp => TaskProgress::WarmingUp,
                            BackgroundStep::Processing { .. } => TaskProgress::Processing,
                            BackgroundStep::Success { .. } => TaskProgress::Success(msg.clone()),
                            BackgroundStep::PostSuccess { .. } => {
                                TaskProgress::PostSuccess(msg.clone())
                            }
                        };
                        state.step = step.clone();
                    }
                    LoopAction::Complete => {
                        state.progress = TaskProgress::Complete;
                        state.stopped = true;
                    }
                }

                if let LoopAction::Continue = action {
                } else {
                    if let LoopAction::Break(_) = action {
                    } else {
                        state.substep = 0;
                    }
                    break;
                }
            }

            Some(((uid, state.progress.clone()), state))
        }))
    }
}

struct BackgroundState {
    pid: String,
    so: String,
    sv: String,
    option: String,
    delivery: Delivery,

    login: String,
    password: String,
    webhook: Webhook,
    flags: (bool, bool),

    client: Client,
    progress: TaskProgress,

    step: BackgroundStep,
    substep: u8,
    start: Instant,
    stopped: bool,
    link: Arc<()>,
}

#[derive(Clone)]
enum BackgroundStep {
    Start,
    WarmingUp,
    Processing {
        link: String,
    },
    Success {
        link: String,
        order: String,
        total: String,
    },
    PostSuccess {
        order: String,
        attempt: u8,
    },
}

enum LoopAction {
    Continue,
    Error(String, bool),

    Break(Option<String>),
    Move(BackgroundStep, Option<String>),
    Complete,
}
