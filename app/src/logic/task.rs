use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use iced::{
    button, Align, Button, Color, Column, Container, Element, Length, Row, Subscription, Text,
};
use iced_futures::futures::stream;
use iced_native::subscription::Recipe;
use reqwest::{redirect::Policy, Client, Proxy, Response, StatusCode};
use serde::Deserialize;
use serde_json::json;
use tokio::time::sleep;

use crate::{
    icons::{icon, Icon},
    layout::Message,
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
    Flat,
    Shipard,
    EMS,
    DHL,
}

impl Delivery {
    pub const ALL: [Delivery; 4] = [
        Delivery::Flat,
        Delivery::Shipard,
        Delivery::EMS,
        Delivery::DHL,
    ];

    fn to_str(&self) -> &str {
        match *self {
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
    pub w_id: u128,
    pub w_token: String,
    pub progress: TaskProgress,

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
        w_id: u128,
        w_token: String,
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
            w_id,
            w_token,
            progress: TaskProgress::Start,
            state: TaskState::default(),
        }
    }

    pub fn init_client(proxy: Option<String>) -> Client {
        // use std::sync::Arc;

        // let mut tls = rustls::ClientConfig::new();
        // tls.set_protocols(&["h2".into(), "http/1.1".into()]);
        // tls.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        // tls.key_log = Arc::new(rustls::KeyLogFile::new());

        let mut client = Client::builder()
            .tcp_keepalive(Some(Duration::from_secs(4)))
            // .use_preconfigured_tls(tls)
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
            .http1_title_case_headers();

        match proxy {
            Some(address) => {
                client = client.proxy(Proxy::all(format!("http://{}", address)).unwrap())
            }
            None => (),
        }

        client.build().unwrap()
    }

    pub fn variti_ban(resp: &Response) -> bool {
        if resp.status() == StatusCode::OK {
            match resp.headers().get("Server").unwrap().to_str() {
                Ok(server) => {
                    if server.starts_with("Variti") {
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        }
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
                    w_id: self.w_id,
                    w_token: self.w_token.clone(),
                    client: Task::init_client(self.proxy.clone()),
                    step: BackgroundStep::Start,
                    start: Instant::now(),
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
            let mut task_progress = None;

            match state.step {
                BackgroundStep::Start => {
                    state.step = BackgroundStep::WarmingUp;
                    task_progress = Some(TaskProgress::WarmingUp);
                }
                BackgroundStep::WarmingUp => {
                    match request(
                        &mut state.client,
                        "https://brandshop.ru/",
                        None,
                        "https://google.ru/",
                        0,
                        "A",
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(err) => task_progress = err,
                    }
                    if task_progress.is_none() {
                        match request(
                            &mut state.client,
                            "https://brandshop.ru/login/",
                            Some(&[
                                ("email", state.login.as_str()),
                                ("password", state.password.as_str()),
                                ("redirect", ""),
                            ]),
                            "https://brandshop.ru/login/",
                            rand_millis(45..=50),
                            "B/1",
                        )
                        .await
                        {
                            Ok(resp) => {
                                if resp.status() == StatusCode::FOUND {
                                    match request(
                                        &mut state.client,
                                        "https://brandshop.ru/account/",
                                        None,
                                        "https://brandshop.ru/login/",
                                        rand_millis(24..=26),
                                        "B/2",
                                    )
                                    .await
                                    {
                                        Ok(_) => {}
                                        Err(err) => task_progress = err,
                                    }
                                } else {
                                    task_progress =
                                        Some(TaskProgress::Error(String::from("Bad credentials")));
                                }
                            }
                            Err(err) => task_progress = err,
                        }
                    }
                    if task_progress.is_none() {
                        match request(
                            &mut state.client,
                            "https://brandshop.ru/xhr/cart/",
                            None,
                            "https://brandshop.ru/",
                            rand_millis(28..=30),
                            "C/2",
                        )
                        .await
                        {
                            Ok(resp) => {
                                let cart = resp.json::<Cart>().await.unwrap();
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
                                            rand_millis(9..=11),
                                            "C/2",
                                        )
                                        .await
                                        {
                                            Ok(_) => (),
                                            Err(_) => (),
                                        }
                                    }
                                }

                                state.step = BackgroundStep::Processing;
                                task_progress = Some(TaskProgress::Processing);
                            }
                            Err(err) => task_progress = err,
                        }
                    }
                }
                BackgroundStep::Processing => {
                    match request(
                        &mut state.client,
                        "https://brandshop.ru/index.php?route=checkout/cart/add",
                        Some(&[
                            ("quantity", "1"),
                            ("product_id", &state.pid),
                            ("option_value_id", &state.sv),
                            (format!("option[{}]", &state.option).as_str(), &state.so),
                        ]),
                        "https://brandshop.ru/",
                        rand_millis(45..=50),
                        "D",
                    )
                    .await
                    {
                        Ok(resp) => {
                            if resp.text().await.unwrap() == "[]" {
                                task_progress = Some(TaskProgress::Failed(String::from(
                                    "Can't add product to cart",
                                )));
                            }
                        }
                        Err(err) => task_progress = err,
                    }
                    if task_progress.is_none() {
                        match request(
                            &mut state.client,
                            "https://brandshop.ru/checkout/",
                            None,
                            "https://brandshop.ru/",
                            rand_millis(28..=30),
                            "E",
                        )
                        .await
                        {
                            Ok(resp) => {
                                if resp.status() != StatusCode::OK {
                                    task_progress = Some(TaskProgress::Failed(String::from(
                                        "Something went wrong",
                                    )));
                                }
                            }
                            Err(err) => task_progress = err,
                        }
                    }
                    if task_progress.is_none() {
                        match request(&mut state.client, "https://brandshop.ru/index.php?route=checkout/checkout/setshippingmethod", Some(&[("shipping_method", state.delivery.to_str())]), "https://brandshop.ru/checkout/", rand_millis(24..=26), "F").await {
                            Ok(resp) => if !resp.json::<Checker>().await.unwrap().ok() {
                                task_progress = Some(TaskProgress::Failed(String::from(
                                    "Can't select delivery method",
                                )));
                            }
                            Err(err) => task_progress = err,
                        }
                    }
                    if task_progress.is_none() {
                        match request(&mut state.client, "https://brandshop.ru/index.php?route=checkout/checkout/setpaymentmethod", Some(&[("payment_method", "payture")]), "https://brandshop.ru/checkout/", rand_millis(28..=30), "G").await {
                            Ok(resp) => if !resp.json::<Checker>().await.unwrap().ok() {
                                task_progress = Some(TaskProgress::Failed(String::from(
                                    "Can't select payment method",
                                )));
                            }
                            Err(err) => task_progress = err,
                        }
                    }
                    if task_progress.is_none() {
                        match request(
                            &mut state.client,
                            "https://brandshop.ru/checkout/",
                            None,
                            "https://brandshop.ru/",
                            rand_millis(28..=30),
                            "H",
                        )
                        .await
                        {
                            Ok(resp) => {
                                if resp.status() != StatusCode::OK {
                                    task_progress = Some(TaskProgress::Failed(String::from(
                                        "Something went wrong",
                                    )));
                                }
                            }
                            Err(err) => task_progress = err,
                        }
                    }

                    let mut link = String::new();

                    if task_progress.is_none() {
                        match request(
                            &mut state.client,
                            "https://brandshop.ru/xhr/payture/",
                            Some(&[]),
                            "https://brandshop.ru/checkout/",
                            rand_millis(10..=15),
                            "I",
                        )
                        .await
                        {
                            Ok(resp) => {
                                let result = resp.json::<Checker>().await.unwrap();
                                if result.ok() {
                                    link = result.success;
                                } else {
                                    task_progress = Some(TaskProgress::Failed(String::from(
                                        "Can't retrieve payment link",
                                    )));
                                }
                            }
                            Err(err) => task_progress = err,
                        }
                    }
                    if task_progress.is_none() {
                        match state
                            .client
                            .get(&link)
                            .header("Referer", "https://brandshop.ru/")
                            .send()
                            .await
                        {
                            Ok(resp) => {
                                if resp.headers().get("Transfer-Encoding").is_some() {
                                    let body = resp.text().await.unwrap();
                                    state.step = BackgroundStep::Success {
                                        link,
                                        total: match retrieve(&body, r#"al" value=""#, "\"") {
                                            Some(sum) => sum,
                                            None => String::from("-"),
                                        },
                                        order: match retrieve(&body, r#"а №"#, "\"") {
                                            Some(oid) => oid,
                                            None => String::from("-"),
                                        },
                                    };
                                    task_progress = Some(TaskProgress::Success(Some(
                                        String::from("Sending link"),
                                    )));
                                } else {
                                    task_progress = Some(TaskProgress::Failed(String::from(
                                        "Payment link broken",
                                    )));
                                }
                            }
                            Err(err) => {
                                task_progress = if err.is_timeout() {
                                    Some(TaskProgress::Error(String::from("Timeout")))
                                } else {
                                    Some(TaskProgress::Error(String::from(
                                        "Connection error. Try again later",
                                    )))
                                }
                            }
                        }
                    }
                }
                BackgroundStep::Success {
                    ref link,
                    ref total,
                    ref order,
                } => {
                    let resp = Task::init_client(None)
                        .post(format!("https://discord.com/api/webhooks/{}/{}", state.w_id, state.w_token))
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
                        task_progress = Some(TaskProgress::PostSuccess(Some(String::from(
                            "Checker: Starting",
                        ))));
                        state.step = BackgroundStep::PostSuccess {
                            order: order.clone(),
                            attempt: 0,
                            substep: 0,
                        };
                    } else {
                        task_progress = Some(TaskProgress::Error(String::from("Webhook error")));
                    }
                }
                BackgroundStep::PostSuccess {
                    ref order,
                    ref mut attempt,
                    ref mut substep,
                } => {
                    let c = substep.clone();
                    match c {
                        0 => {
                            if *attempt == 120 {
                                task_progress = Some(TaskProgress::Failed(String::from(
                                    "Checker: order not detected",
                                )));
                            } else {
                                sleep(Duration::from_secs(3)).await;
                                task_progress = Some(TaskProgress::PostSuccess(Some(
                                    String::from("Checker: Waiting"),
                                )));
                                *substep = 1;
                            }
                        }
                        1 => {
                            sleep(Duration::from_secs(27)).await;
                            task_progress = Some(TaskProgress::PostSuccess(Some(String::from(
                                "Checker: Scanning",
                            ))));
                            *substep = 2;
                        }
                        2 => {
                            match request(
                                &mut state.client,
                                "https://brandshop.ru/order/",
                                Some(&[]),
                                "https://brandshop.ru/account/",
                                rand_millis(10..=15),
                                "J",
                            )
                            .await
                            {
                                Ok(resp) => {
                                    let body = resp.text().await.unwrap();

                                    if let Some(order_body) = retrieve(&body, &order, "</li>") {
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
                                                    let resp = Task::init_client(None)
                                                        .post(format!("https://discord.com/api/webhooks/{}/{}", state.w_id, state.w_token))
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
                                                        task_progress =
                                                            Some(TaskProgress::Complete);
                                                    } else {
                                                        task_progress = Some(TaskProgress::Error(
                                                            String::from("Checker: Webhook error"),
                                                        ));
                                                    }
                                                }
                                            }
                                        }

                                        if task_progress.is_none() {
                                            task_progress = Some(TaskProgress::Failed(
                                                String::from("Checker: Something went wrong"),
                                            ))
                                        }
                                    }
                                }
                                Err(err) => task_progress = err,
                            }

                            if task_progress.is_none() {
                                *substep = 0;
                                *attempt += 1;
                                task_progress = Some(TaskProgress::PostSuccess(Some(
                                    String::from("Checker: Waiting"),
                                )));
                            }
                        }
                        _ => {
                            task_progress = Some(TaskProgress::Error(String::from("Checker down")))
                        }
                    }
                }
                BackgroundStep::Stop => {
                    return None;
                }
            }

            match task_progress.clone().unwrap() {
                TaskProgress::Complete | TaskProgress::Failed(_) | TaskProgress::Error(_) => {
                    state.step = BackgroundStep::Stop;
                }
                _ => (),
            }

            Some(((uid, task_progress.unwrap()), state))
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
    w_id: u128,
    w_token: String,

    client: Client,
    step: BackgroundStep,
    start: Instant,
}

enum BackgroundStep {
    Start,
    WarmingUp,
    Processing,
    Success {
        link: String,
        order: String,
        total: String,
    },
    PostSuccess {
        order: String,
        attempt: u8,
        substep: u8,
    },
    Stop,
}
