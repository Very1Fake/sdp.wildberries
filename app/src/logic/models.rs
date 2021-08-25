use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{themes::Theme, views::tabs::proxy::ProxyMode};

////////////////////////////////////////////////////////////////////////////////////////////////////
// System models
////////////////////////////////////////////////////////////////////////////////////////////////////

// Settings model
#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(default)]
pub struct Settings {
    pub webhook: Webhook,
    pub proxy_mode: ProxyMode,

    // Appearance
    #[serde(skip)]
    pub theme: Theme,
    pub scale: f64,

    // Experimental flags
    pub limiter: bool,
    pub force: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            webhook: Webhook::default(),
            proxy_mode: ProxyMode::default(),

            theme: Theme::Light,
            scale: 1.0,

            limiter: true,
            force: false,
        }
    }
}

// Webhook model
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Webhook {
    pub id: u64,
    pub token: String,
}

impl Default for Webhook {
    fn default() -> Self {
        Webhook {
            id: 0,
            token: String::new(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Site specific models
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Clone, Debug)]
pub struct ResponseResult {
    #[serde(rename = "resultState", alias = "ResultState")]
    pub state: i64,
    #[serde(alias = "Value", default)]
    pub value: ResponseValue,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResponseValue {
    Message(String),
    Value(Value),
    Order { url: String },
    Basket(Data),
    None,
}

impl Default for ResponseValue {
    fn default() -> Self {
        ResponseValue::None
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Value {
    #[serde(alias = "Data")]
    pub data: Data,
    #[serde(rename = "userInfo")]
    pub user: Option<User>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    pub country: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "middleName")]
    pub middle_name: String,
    pub phone: u64,
    #[serde(rename = "formattedPhoneMobile")]
    pub phone_str: String,
    #[serde(rename = "someId")]
    pub id: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Data {
    pub basket: Option<Basket>,
    #[serde(rename = "basketInfo", alias = "basketShortInfo")]
    pub basket_info: Option<BasketInfo>,
    #[serde(rename = "productCard")]
    pub product_card: Option<ProductCard>,
    #[serde(rename = "selectedNomenclature")]
    pub variant: Option<Variant>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct Basket {
    #[serde(rename = "paymentType")]
    pub payment_type: PaymentType,
    #[serde(rename = "deliveryWays")]
    pub delivery_ways: Vec<DeliveryWay>,
    #[serde(rename = "deliveryWay")]
    pub delivery_way: String,
    #[serde(rename = "deliveryIntervalTxt")]
    pub delivery_interval_str: Option<String>,
    #[serde(rename = "deliveryPoint")]
    pub delivery_point: DeliveryPoint,
    #[serde(rename = "includeInOrder")]
    pub order_items: Vec<u64>,
    #[serde(rename = "totalPriceToPay")]
    pub total_price: u64,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct PaymentType {
    pub id: String,
    #[serde(rename = "bankCardId")]
    pub card: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BasketInfo {
    #[serde(rename = "isAuthenticated")]
    pub is_auth: bool,
    #[serde(rename = "basketQuantity")]
    pub quantity: u64,
    #[serde(rename = "eventsCount")]
    pub events_count: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProductCard {
    #[serde(rename = "goodsName")]
    pub name: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Delivery
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Default, Clone, Debug)]
pub struct DeliveryWay {
    pub code: String,
    pub calendars: Vec<Calendar>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct Calendar {
    #[serde(rename = "storeIds")]
    pub store_ids: Vec<u64>,
    #[serde(rename = "shippingInterval")]
    pub shipping_interval: ShippingInterval,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct ShippingInterval {
    #[serde(rename = "intervalId")]
    pub id: u64,
    #[serde(rename = "deliveryDateShort")]
    pub delivery_date: String,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct DeliveryPoint {
    #[serde(rename = "kladrId")]
    pub id: u64,
    pub address: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Variant
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Clone, Debug)]
pub struct Variant {
    #[serde(rename = "isSoldOut")]
    pub sold_out: bool,
    #[serde(rename = "cod1S")]
    pub id: u64,
    #[serde(rename = "rusName")]
    pub name: Option<String>,
    pub sizes: BTreeMap<String, Size>,
}

impl Variant {
    pub fn sizes_tags(&self) -> Vec<SizeTag> {
        self.sizes.iter().map(|(_, s)| s.as_tag()).collect()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Size
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Clone, Debug)]
pub struct Size {
    #[serde(rename = "characteristicId")]
    pub id: u64,
    #[serde(rename = "sizeName")]
    pub name: String,
    #[serde(rename = "price")]
    pub price: u64,
    #[serde(rename = "priceWithSale")]
    pub sale_price: u64,
    #[serde(rename = "quantity")]
    pub quantity: u64,
    #[serde(rename = "isSoldOut")]
    pub sold_out: bool,
}

impl Size {
    pub fn as_tag(&self) -> SizeTag {
        SizeTag {
            id: self.id,
            name: self.name.clone(),
            quantity: self.quantity,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SizeTag {
    pub id: u64,
    pub name: String,
    pub quantity: u64,
}

impl Display for SizeTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.quantity == 0 {
            write!(f, r#"Size "{}" (Sold Out)"#, self.name)
        } else {
            write!(f, r#"Size "{}" (Q: {})"#, self.name, self.quantity)
        }
    }
}
