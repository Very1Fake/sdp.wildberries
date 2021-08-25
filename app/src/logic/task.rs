use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{offset::TimeZone, Local, NaiveDate, NaiveDateTime, Utc};
use iced::{
    button, Align, Button, Color, Column, Container, Element, Length, Row, Rule, Subscription, Text,
};
use iced_futures::futures::stream;
use iced_native::subscription::Recipe;
use reqwest::{Client, StatusCode};
use serde_json::{from_str, json};

use crate::{
    icons::{icon, Icon},
    layout::Message,
    logic::{
        misc::RequestMethod,
        models::{Basket, ProductCard, ResponseResult, ResponseValue, Webhook},
    },
    themes::Theme,
    EDITION, SITE, VERSION,
};

use super::{
    misc::{client, rand_millis, request, retrieve},
    models::{Size, Variant},
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Task
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Task {
    pub uid: u64,

    pub proxy: Option<String>,
    pub card: ProductCard,
    pub variant: Variant,
    pub size: Size,

    pub account: (String, String),
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
        card: ProductCard,
        variant: Variant,
        size: Size,
        account: (String, String),
        webhook: Webhook,
        flags: (bool, bool),
    ) -> Task {
        Task {
            uid,
            proxy,
            card,
            variant,
            size,
            account,
            webhook,
            flags,
            progress: TaskProgress::Start,
            link: Arc::new(()),
            state: TaskState::default(),
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
                    .push(Text::new(&format!("#{}", self.uid)).width(Length::Units(24)))
                    .push(Text::new(&self.card.name).width(Length::FillPortion(2)))
                    .push(
                        Text::new(self.progress.to_str())
                            .width(Length::FillPortion(1))
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
                let field = |name: &str, value: &str, fill: u16| {
                    Column::new()
                        .push(Text::new(name).color(theme.color_text_muted()).size(15))
                        .push(Text::new(value).size(19))
                        .width(Length::FillPortion(fill))
                };
                let mut retry_btn = Button::new(retry_btn, icon(Icon::Reload))
                    .width(Length::Shrink)
                    .padding(8)
                    .style(theme.primary_btn());

                if let TaskProgress::Error(_) = self.progress {
                    retry_btn = retry_btn.on_press(TaskMsg::Retry);
                }

                Container::new(
                    Column::new()
                        .push(
                            Row::new()
                                .push(Text::new(&format!("#{}", self.uid)).width(Length::Units(24)))
                                .push(Text::new(&self.card.name).width(Length::FillPortion(2)))
                                .push(
                                    Text::new(self.progress.to_str())
                                        .width(Length::FillPortion(1))
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
                            Rule::horizontal(theme.task_divider_spacing())
                                .style(theme.task_divider()),
                        )
                        .push(
                            Row::new()
                                .push(field(
                                    "Variant (Color)",
                                    match self.variant.name {
                                        Some(ref name) => name,
                                        None => "-",
                                    },
                                    3,
                                ))
                                .push(field(
                                    "Size",
                                    if self.variant.sizes.len() == 1 {
                                        "-"
                                    } else {
                                        &self.size.name
                                    },
                                    2,
                                ))
                                .push(field("Account", &self.account.0, 3))
                                .push(field(
                                    "Proxy",
                                    match self.proxy {
                                        Some(ref proxy) => proxy,
                                        None => "-",
                                    },
                                    4,
                                ))
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
            | TaskProgress::Completing(_) => Subscription::from_recipe(Background {
                uid: self.uid,
                state: BackgroundState {
                    card: self.card.clone(),
                    variant: self.variant.clone(),
                    size: self.size.clone(),
                    phone: self.account.0.clone(),
                    webhook: self.webhook.clone(),
                    flags: self.flags.clone(),
                    client: client(
                        self.proxy.clone(),
                        Some(&[(
                            String::from("WILDAUTHNEW_V3"),
                            self.account.1.clone(),
                            String::from("wildberries.ru"),
                        )]),
                    ),
                    progress: TaskProgress::Start,
                    step: BackgroundStep::Start,
                    substep: 0,
                    start: SystemTime::now(),
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
    Completing(Option<String>),

    Error(String),
    Complete(Option<String>),
    Failed(Option<String>),
}

impl TaskProgress {
    fn to_str(&self) -> String {
        match self {
            TaskProgress::Start => String::from("Starting"),
            TaskProgress::WarmingUp => String::from("Warming Up"),
            TaskProgress::Processing => String::from("Processing"),
            TaskProgress::Completing(msg) => match msg {
                Some(ref text) => format!("Completing: {}", text.clone()),
                None => String::from("Completing"),
            },
            TaskProgress::Complete(msg) => match msg {
                Some(ref text) => format!("Complete: {}", text),
                None => String::from("Complete"),
            },
            TaskProgress::Failed(msg) => match msg {
                Some(text) => format!("Failed: {}", text),
                None => String::from("Failed"),
            },
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
            // TaskProgress::PostSuccess(_) => Color::from_rgb(0.188, 0.31, 0.996),
            TaskProgress::Completing(_) => Color::from_rgb(0.0, 0.784, 0.325),
            TaskProgress::Complete(_) => Color::from_rgb(0.392, 0.867, 0.09),
            TaskProgress::Failed(_) | TaskProgress::Error(_) => Color::from_rgb(0.835, 0.0, 0.0),
        }
    }
}

impl Default for TaskProgress {
    fn default() -> Self {
        TaskProgress::Start
    }
}

enum TaskError {
    Response,
    Scheme,
    Unknown,
}

impl TaskError {
    pub fn to_string(&self, tier: &str) -> String {
        match &self {
            TaskError::Response => format!("Bad response ({})", tier),
            TaskError::Scheme => format!("Unknown scheme ({})", tier),
            TaskError::Unknown => format!("Unknown ({})", tier),
        }
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
                        action = LoopAction::Move(BackgroundStep::Warmup, None);
                    }
                    BackgroundStep::Warmup => match state.substep {
                        // Token check (A)
                        0 => match request(
                            &mut state.client,
                            "https://www.wildberries.ru/lk/personalcabinet/data",
                            RequestMethod::GET,
                            "https://www.wildberries.ru/lk",
                            0,
                        )
                        .await
                        {
                            Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                Ok(result) => {
                                    if result.state == -1 {
                                        action = LoopAction::Error(String::from(
                                            "Account token is expired",
                                        ))
                                    }
                                }
                                Err(_) => {
                                    action = LoopAction::Error(TaskError::Response.to_string("A"))
                                }
                            },
                            Err(err) => action = LoopAction::Error(err.to_string("A")),
                        },
                        // User location cookie (B)
                        1 => match request(
                            &mut state.client,
                            "https://www.wildberries.ru/geo/getuserlocationinfo",
                            RequestMethod::POST(None),
                            "https://www.wildberries.ru/login?returnUrl=https://wildberries.ru/",
                            if state.flags.0 {
                                rand_millis(5..=10)
                            } else {
                                0
                            },
                        )
                        .await
                        {
                            Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                Ok(result) => {
                                    if result.state == -1 {
                                        action = LoopAction::Error(String::from(
                                            "Can't get user location (B)",
                                        ))
                                    }
                                }
                                Err(_) => {
                                    action = LoopAction::Error(TaskError::Response.to_string("B"))
                                }
                            },
                            Err(err) => action = LoopAction::Error(err.to_string("B")),
                        },
                        // Check cart for other products (C)
                        2 => {
                            match request(
                                &mut state.client,
                                "https://www.wildberries.ru/lk/basket/data",
                                RequestMethod::GET,
                                "https://www.wildberries.ru/lk/basket",
                                if state.flags.0 {
                                    rand_millis(10..=20)
                                } else {
                                    0
                                },
                            )
                            .await
                            {
                                Ok(resp) => {
                                    match from_str::<ResponseResult>(&resp.body) {
                                        Ok(result) => {
                                            if result.state == 0 {
                                                if let ResponseValue::Value(value) = result.value {
                                                    match value.data.basket {
                                                        Some(basket) => {
                                                            if !basket.order_items.is_empty() {
                                                                // Clear cart (D)
                                                                match request(
                                                                &mut state.client,
                                                                "https://www.wildberries.ru/lk/basket/spa/delete",
                                                                RequestMethod::POST(
                                                                    Some(
                                                                        &basket.order_items.iter()
                                                                            .enumerate()
                                                                            .map(|(i, p)| {
                                                                                (format!("chrtIds[{}]", i), p.to_string())
                                                                            })
                                                                            .collect()
                                                                    )
                                                                ),
                                                                "https://www.wildberries.ru/lk/basket",
                                                                if state.flags.0 {
                                                                    rand_millis(5..=10)
                                                                } else {
                                                                    0
                                                                },
                                                            ).await {
                                                                Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                                                    Ok(result) => if result.state == -1 {
                                                                        action =
                                                                            LoopAction::Error(
                                                                                String::from("Can't remove other items from cart"),

                                                                            )
                                                                    }
                                                                    Err(_) => action = LoopAction::Error(
                                                                        TaskError::Response.to_string("D"),

                                                                    ),
                                                                },
                                                                Err(err) => action = LoopAction::Error(
                                                                    err.to_string("D"),

                                                                )
                                                            }
                                                            }
                                                        }
                                                        None => {
                                                            action = LoopAction::Error(
                                                                TaskError::Scheme.to_string("C/B"),
                                                            )
                                                        }
                                                    }
                                                }
                                            } else {
                                                action = LoopAction::Error(String::from(
                                                    "Can't retrieve cart data",
                                                ))
                                            }
                                        }
                                        Err(_) => {
                                            action = LoopAction::Error(
                                                TaskError::Response.to_string("C"),
                                            )
                                        }
                                    }

                                    match action {
                                        LoopAction::Error(_) if !state.flags.1 => {}
                                        LoopAction::Continue | LoopAction::Error(_) => {
                                            action = LoopAction::Move(
                                                BackgroundStep::Process {
                                                    cart: Basket::default(),
                                                },
                                                None,
                                            )
                                        }
                                        _ => {}
                                    }
                                }
                                Err(err) => action = LoopAction::Error(err.to_string("C")),
                            }
                        }
                        _ => {}
                    },
                    BackgroundStep::Process { ref mut cart } => {
                        match state.substep {
                            // Check product availability (E)
                            0 => match request(
                                &mut state.client,
                                &format!(
                                    "https://www.wildberries.ru/{}/product/data?targetUrl=XS",
                                    state.variant.id,
                                ),
                                RequestMethod::GET,
                                &format!(
                                "https://www.wildberries.ru/catalog/{}/detail.aspx?targetUrl=XS",
                                state.variant.id
                            ),
                                if state.flags.0 {
                                    rand_millis(25..=30)
                                } else {
                                    0
                                },
                            )
                            .await
                            {
                                Ok(resp) => {
                                    if resp.status == StatusCode::NOT_FOUND {
                                        action =
                                            LoopAction::Error(String::from("Product not found"));
                                    } else {
                                        match from_str::<ResponseResult>(&resp.body) {
                                            Ok(result) => {
                                                if let ResponseValue::Value(value) = result.value {
                                                    match value.data.variant {
                                                        Some(variant) => {
                                                            if variant.sold_out {
                                                                action = LoopAction::Error(
                                                                    String::from(
                                                                        "Product already sold out",
                                                                    ),
                                                                );
                                                            } else {
                                                                match variant
                                                                .sizes
                                                                .get(&state.size.id.to_string())
                                                            {
                                                                Some(size) => {
                                                                    if size.sold_out {
                                                                        action = LoopAction::Error(String::from("Product size already sold out"), );
                                                                    }
                                                                }
                                                                None => action = LoopAction::Error(
                                                                    String::from(
                                                                        "Product size not found",
                                                                    ),

                                                                ),
                                                            }
                                                            }
                                                        }
                                                        None => {
                                                            action = LoopAction::Error(
                                                                TaskError::Scheme
                                                                    .to_string("E/VRT"),
                                                            )
                                                        }
                                                    }
                                                } else {
                                                    action = LoopAction::Error(
                                                        TaskError::Scheme.to_string("E/V"),
                                                    );
                                                }
                                            }
                                            Err(_) => {
                                                action = LoopAction::Error(
                                                    TaskError::Response.to_string("E"),
                                                )
                                            }
                                        }
                                    }
                                }
                                Err(err) => action = LoopAction::Error(err.to_string("E")),
                            },
                            // Add product to cart (F)
                            1 => match request(
                                &mut state.client,
                                "https://www.wildberries.ru/product/addtobasket",
                                RequestMethod::POST(Some(&vec![
                                    (String::from("cod1S"), state.variant.id.to_string()),
                                    (String::from("characteristicId"), state.size.id.to_string()),
                                    (String::from("quantity"), String::from("1")),
                                ])),
                                &format!(
                                "https://www.wildberries.ru/catalog/{}/detail.aspx?targetUrl=XS",
                                state.variant.id
                            ),
                                if state.flags.0 {
                                    rand_millis(25..=30)
                                } else {
                                    0
                                },
                            )
                            .await
                            {
                                Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                    Ok(result) => {
                                        if result.state == -1 {
                                            action = LoopAction::Error(String::from(
                                                "Something went wrong (F)",
                                            ));
                                        } else {
                                            if let ResponseValue::Basket(data) = result.value {
                                                match data.basket_info {
                                                    Some(basket_short) => {
                                                        match basket_short.quantity {
                                                            0 => {
                                                                action =
                                                                    LoopAction::Error(String::from(
                                                                        "Can't add product to cart",
                                                                    ))
                                                            }
                                                            1 => {}
                                                            _ => {
                                                                if !state.flags.1 {
                                                                    action = LoopAction::Error(
                                                            String::from("Cart corrupted. Check it by yourself"),

                                                        )
                                                                }
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        action = LoopAction::Error(
                                                            TaskError::Scheme.to_string("F/BS"),
                                                        )
                                                    }
                                                }
                                            } else {
                                                action = LoopAction::Error(
                                                    TaskError::Scheme.to_string("F/V"),
                                                );
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        action =
                                            LoopAction::Error(TaskError::Response.to_string("F"))
                                    }
                                },
                                Err(err) => action = LoopAction::Error(err.to_string("F")),
                            },
                            // Collect final cart data (G)
                            2 => match request(
                                &mut state.client,
                                "https://www.wildberries.ru/lk/basket/data",
                                RequestMethod::GET,
                                "https://www.wildberries.ru/lk/basket",
                                if state.flags.0 {
                                    rand_millis(15..=25)
                                } else {
                                    0
                                },
                            )
                            .await
                            {
                                Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                    Ok(result) => {
                                        if result.state == 0 {
                                            if let ResponseValue::Value(value) = result.value {
                                                match value.data.basket {
                                                    Some(data) => *cart = data,
                                                    None => {
                                                        action = LoopAction::Error(
                                                            TaskError::Scheme.to_string("G/B"),
                                                        )
                                                    }
                                                }
                                            } else {
                                                action = LoopAction::Error(
                                                    TaskError::Scheme.to_string("G/V"),
                                                );
                                            }
                                        } else {
                                            action = LoopAction::Error(String::from(
                                                "Can't retrieve cart data",
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        action =
                                            LoopAction::Error(TaskError::Response.to_string("G"))
                                    }
                                },
                                Err(err) => action = LoopAction::Error(err.to_string("G")),
                            },
                            // Submit order (H)
                            3 => {
                                let mut form = vec![
                                    (
                                        String::from("orderDetails.DeliveryPointId"),
                                        cart.delivery_point.id.to_string(),
                                    ),
                                    (
                                        String::from("orderDetails.DeliveryWay"),
                                        cart.delivery_way.clone(),
                                    ),
                                    (String::from("orderDetails.DeliveryPrice"), String::new()),
                                ];

                                {
                                    cart.delivery_ways[0].calendars.iter().enumerate().for_each(|(cid, c)| {
                                    form.push((String::from("orderDetails.DeliveryDts.Index"), cid.to_string()));
                                    form.push((
                                        format!("orderDetails.DeliveryDts[{}].Date", cid),
                                        NaiveDate::parse_from_str(&c.shipping_interval.delivery_date, "%-m/%-d/%Y").unwrap().format("%d.%m.%Y").to_string(),
                                    ));
                                    form.push((String::from("orderDetails.DeliveryDts[0].IntervalId"), c.shipping_interval.id.to_string()));

                                    c.store_ids.iter().enumerate().for_each(|(sid, s)| {
                                        form.push((String::from("orderDetails.DeliveryDts[0].StoreIds.Index"), sid.to_string()));
                                        form.push((format!("orderDetails.DeliveryDts[0].StoreIds[{}]", sid), s.to_string()));
                                    });
                                });

                                    form.push((
                                        String::from("orderDetails.GooglePayToken"),
                                        false.to_string(),
                                    ));
                                    form.push((
                                        String::from("orderDetails.PaymentType.Id"),
                                        cart.payment_type.id.to_string(),
                                    ));
                                    form.push((
                                        String::from("orderDetails.MaskedCardId"),
                                        cart.payment_type.card.clone(),
                                    ));
                                    form.push((
                                        String::from("orderDetails.SberPayPhone"),
                                        String::new(),
                                    ));
                                    form.push((
                                        String::from("orderDetails.AgreePublicOffert"),
                                        true.to_string(),
                                    ));
                                    form.push((
                                        String::from("orderDetails.TotalPrice"),
                                        cart.total_price.to_string(),
                                    ));

                                    cart.order_items.iter().enumerate().for_each(|(id, i)| {
                                        form.push((
                                            String::from("orderDetails.UserBasketItems.Index"),
                                            id.to_string(),
                                        ));
                                        form.push((
                                            format!(
                                                "orderDetails.UserBasketItems[{}].CharacteristicId",
                                                id
                                            ),
                                            i.to_string(),
                                        ));
                                        form.push((
                                            format!("orderDetails.IncludeInOrder[{}]", id),
                                            i.to_string(),
                                        ));
                                    });
                                }

                                match request(
                                    &mut state.client,
                                    "https://www.wildberries.ru/lk/basket/spa/submitorder",
                                    RequestMethod::POST(Some(&form)),
                                    "https://www.wildberries.ru/lk/basket",
                                    if state.flags.0 {
                                        rand_millis(15..=20)
                                    } else {
                                        0
                                    },
                                )
                                .await
                                {
                                    Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                        Ok(result) => {
                                            if result.state == -1 {
                                                action = LoopAction::Error(
                                                    TaskError::Unknown.to_string("H"),
                                                );
                                            } else {
                                                if let ResponseValue::Order { url } = result.value {
                                                    if url.ends_with("payment/fail") {
                                                        action = LoopAction::Jump(4)
                                                    } else if url
                                                        .starts_with("https://beta.paywb.com")
                                                    {
                                                        action = LoopAction::Move(
                                                            BackgroundStep::End {
                                                                content: url,
                                                                cart: cart.clone(),
                                                                kind: EndKind::UserAction,
                                                            },
                                                            None,
                                                        )
                                                    } else if url.contains("orderId") {
                                                        // Confirm payment success (J)
                                                        action = LoopAction::Move(
                                                            BackgroundStep::End {
                                                                content: url.clone(),
                                                                cart: cart.clone(),
                                                                kind: EndKind::Succeed(match retrieve(&url, "?orderId=", "&paid") {
                                                                    Some(oid) => match request(
                                                                        &mut state.client,
                                                                        &format!("https://www.wildberries.ru/lk/order/confirmed/data?orderId={}&paid=True", oid),
                                                                        RequestMethod::GET,
                                                                        "https://www.wildberries.ru/lk/basket",
                                                                        if state.flags.0 {
                                                                            rand_millis(10..=15)
                                                                        } else {
                                                                            0
                                                                        },
                                                                    ).await {
                                                                        Ok(resp) => match from_str::<ResponseResult>(&resp.body) {
                                                                            Ok(result) => result.state == 0,
                                                                            Err(_) => false,
                                                                        },
                                                                        Err(_) => false,
                                                                    }
                                                                    None => false,
                                                                })
                                                            },
                                                            Some(String::from("Sending embed")),
                                                        )
                                                    } else {
                                                        action = LoopAction::Error(
                                                            TaskError::Scheme.to_string("H/URL"),
                                                        );
                                                    }
                                                } else {
                                                    action = LoopAction::Error(
                                                        TaskError::Scheme.to_string("H/V"),
                                                    );
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            action = LoopAction::Error(
                                                TaskError::Response.to_string("H"),
                                            )
                                        }
                                    },
                                    Err(err) => action = LoopAction::Error(err.to_string("H")),
                                }
                            }
                            // Parse payment error (I)
                            4 => match request(
                                &mut state.client,
                                "https://www.wildberries.ru/lk/payment/fail",
                                RequestMethod::GET,
                                "https://www.wildberries.ru/lk/basket",
                                if state.flags.0 {
                                    rand_millis(10..=15)
                                } else {
                                    0
                                },
                            )
                            .await
                            {
                                Ok(resp) => {
                                    action = LoopAction::Move(
                                        BackgroundStep::End {
                                            content: match retrieve(
                                                &resp.body,
                                                r#"<p class="field-validation-error">"#,
                                                "</p>",
                                            ) {
                                                Some(desc) => desc,
                                                None => String::from("<Can't parse error>"),
                                            },
                                            cart: cart.clone(),
                                            kind: EndKind::Failed,
                                        },
                                        None,
                                    )
                                }
                                Err(err) => action = LoopAction::Error(err.to_string("I")),
                            },
                            _ => {}
                        }
                    }
                    BackgroundStep::End {
                        ref content,
                        ref cart,
                        ref kind,
                    } => {
                        let start_time = match state.start.duration_since(UNIX_EPOCH) {
                            Ok(dur) => Utc
                                .from_utc_datetime(&NaiveDateTime::from_timestamp(
                                    dur.as_secs() as i64,
                                    dur.subsec_nanos(),
                                ))
                                .with_timezone(&Local)
                                .format("%H:%M:%S %d/%m/%Y")
                                .to_string(),
                            Err(_) => String::from("-"),
                        };

                        let fields = match kind {
                            EndKind::Succeed(_) => json!([
                                {
                                    "name": "Order details",
                                    "value": format!("[Click](https://www.wildberries.ru{})", content),
                                },
                                {
                                    "name": "Task ID",
                                    "value": format!("#{}", uid),
                                    "inline": true,
                                },
                                {
                                    "name": "Account",
                                    "value": format!("||{}||", &state.phone),
                                    "inline": true,
                                },
                                { "name": "Site", "value": SITE, "inline": true },
                                { "name": "Product", "value": state.card.name },
                                {
                                    "name": "Variant (Color)",
                                    "value": state.variant.name,
                                    "inline": true,
                                },
                                {
                                    "name": "Size",
                                    "value": if state.variant.sizes.len() == 1 {
                                        "-"
                                    } else {
                                        &state.size.name
                                    },
                                    "inline": true,
                                },
                                {
                                    "name": "Total",
                                    "value": format!("{} RUB", cart.total_price),
                                    "inline": true,
                                },
                                {
                                    "name": "Estimated delivery",
                                    "value": cart.delivery_interval_str,
                                    "inline": true,
                                },
                                {
                                    "name": "Elapsed",
                                    "value": format!(
                                        "{:.3} sec",
                                        state.start.elapsed().unwrap().as_secs_f32()
                                    ),
                                    "inline": true,
                                },
                                { "name": "Start Time", "value": start_time, "inline": true },
                            ]),
                            EndKind::UserAction => json!([
                                {
                                    "name": "Payment confirmation",
                                    "value": format!("[Click]({})", content),
                                },
                                {
                                    "name": "Task ID",
                                    "value": format!("#{}", uid),
                                    "inline": true,
                                },
                                {
                                    "name": "Account",
                                    "value": format!("||{}||", &state.phone),
                                    "inline": true,
                                },
                                { "name": "Site", "value": SITE, "inline": true },
                                { "name": "Product", "value": state.card.name },
                                {
                                    "name": "Variant (Color)",
                                    "value": state.variant.name,
                                    "inline": true,
                                },
                                {
                                    "name": "Size",
                                    "value": if state.variant.sizes.len() == 1 {
                                        "-"
                                    } else {
                                        &state.size.name
                                    },
                                    "inline": true,
                                },
                                {
                                    "name": "Amount of payment",
                                    "value": format!("{} RUB", cart.total_price),
                                    "inline": true,
                                },
                                {
                                    "name": "Estimated delivery",
                                    "value": cart.delivery_interval_str,
                                    "inline": true,
                                },
                                {
                                    "name": "Elapsed",
                                    "value": format!(
                                        "{:.3} sec",
                                        state.start.elapsed().unwrap().as_secs_f32()
                                    ),
                                    "inline": true,
                                },
                                { "name": "Start Time", "value": start_time, "inline": true },
                            ]),
                            EndKind::Failed => json!([
                                { "name": "Description", "value": content },
                                {
                                    "name": "Task ID",
                                    "value": format!("#{}", uid),
                                    "inline": true,
                                },
                                {
                                    "name": "Account",
                                    "value": format!("||{}||", &state.phone),
                                    "inline": true,
                                },
                                { "name": "Site", "value": SITE, "inline": true },
                                { "name": "Product", "value": state.card.name },
                                {
                                    "name": "Variant (Color)",
                                    "value": state.variant.name,
                                    "inline": true,
                                },
                                {
                                    "name": "Size",
                                    "value": if state.variant.sizes.len() == 1 {
                                        "-"
                                    } else {
                                        &state.size.name
                                    },
                                    "inline": true,
                                },
                                {
                                    "name": "Total",
                                    "value": format!("{} RUB", cart.total_price),
                                    "inline": true,
                                },
                                {
                                    "name": "Elapsed",
                                    "value": format!(
                                        "{:.3} sec",
                                        state.start.elapsed().unwrap().as_secs_f32()
                                    ),
                                    "inline": true,
                                },
                                { "name": "Start Time", "value": start_time, "inline": true },
                            ]),
                        };

                        let resp = client(None, None)
                            .post(format!("https://discord.com/api/webhooks/{}/{}", state.webhook.id, state.webhook.token))
                            .json(&json!({
                            "username": "SDP Pre-Alpha",
                            "embeds": [{
                                "title": match kind {
                                    EndKind::Succeed(true) => "Successful Payment",
                                    EndKind::Succeed(false) => "Successful Payment (Unconfirmed)",
                                    EndKind::UserAction => "Bank Payment Confirmation (3D-Secure)",
                                    EndKind::Failed => "Payment Failed",
                                },
                                "description": match kind {
                                    EndKind::Succeed(_) => "Click the link below to see details of your order.",
                                    EndKind::UserAction => "User action required. Click the link below to complete your order payment.",
                                    EndKind::Failed => "",
                                },
                                "color": match kind {
                                    EndKind::Succeed(true) => 51283,
                                    EndKind::Succeed(false) => 2712319,
                                    EndKind::UserAction => 16771584,
                                    EndKind::Failed => 16717636,
                                },
                                "fields": &fields,
                                "footer": {
                                    "text": format!("SDP Pre-Alpha (v{}) (e{})", VERSION, EDITION),
                                    "icon_url": "https://en.gravatar.com/userimage/182691345/3ce2e13566d08dd3ae6513f6b0404900.png"
                                }
                            }]
                        }))
                            .send()
                            .await
                            .unwrap();

                        if resp.status() == StatusCode::NO_CONTENT {
                            action = match kind {
                                EndKind::Succeed(confirm) => {
                                    LoopAction::Complete(Some(String::from(if *confirm {
                                        "Success"
                                    } else {
                                        "Success (Unconfirmed)"
                                    })))
                                }
                                EndKind::UserAction => {
                                    LoopAction::Complete(Some(String::from("User Action Required")))
                                }
                                EndKind::Failed => {
                                    LoopAction::Failed(Some(String::from("Bank error")))
                                }
                            }
                        } else {
                            action = LoopAction::Failed(Some(String::from("Webhook error")));
                        }
                    }
                }
                match action {
                    LoopAction::Continue => state.substep += 1,
                    LoopAction::Jump(substep) => state.substep = substep,
                    LoopAction::Move(ref step, ref msg) => {
                        state.progress = match step {
                            BackgroundStep::Start => TaskProgress::Start,
                            BackgroundStep::Warmup => TaskProgress::WarmingUp,
                            BackgroundStep::Process { .. } => TaskProgress::Processing,
                            BackgroundStep::End { .. } => TaskProgress::Completing(msg.clone()),
                        };
                        state.step = step.clone();
                    }
                    LoopAction::Error(ref msg) => {
                        state.progress = TaskProgress::Error(String::from(msg));
                    }
                    LoopAction::Complete(ref msg) => {
                        state.progress = TaskProgress::Complete(msg.clone());
                    }
                    LoopAction::Failed(ref msg) => {
                        state.progress = TaskProgress::Failed(msg.clone());
                    }
                }

                match action {
                    LoopAction::Continue | LoopAction::Jump(_) => {}
                    _ => {
                        match action {
                            LoopAction::Error(_)
                            | LoopAction::Complete(_)
                            | LoopAction::Failed(_) => state.stopped = true,
                            _ => state.substep = 0,
                        }
                        break;
                    }
                }
            }

            Some(((uid, state.progress.clone()), state))
        }))
    }
}

struct BackgroundState {
    card: ProductCard,
    variant: Variant,
    size: Size,

    phone: String,
    webhook: Webhook,
    flags: (bool, bool),

    client: Client,
    progress: TaskProgress,

    step: BackgroundStep,
    substep: u8,
    start: SystemTime,
    stopped: bool,
    link: Arc<()>,
}

#[derive(Clone)]
enum BackgroundStep {
    Start,
    Warmup,
    Process {
        cart: Basket,
    },
    End {
        content: String,
        cart: Basket,
        kind: EndKind,
    },
}

#[derive(Clone)]
enum EndKind {
    Succeed(bool),
    UserAction,
    Failed,
}

enum LoopAction {
    Continue,
    Jump(u8),

    Move(BackgroundStep, Option<String>),

    Error(String),
    Complete(Option<String>),
    Failed(Option<String>),
}
