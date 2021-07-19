use std::{
    fs::read_to_string,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use blake3::Hasher;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};

use crate::{OS, VERSION};

static PUBLIC_KEY: &str = include_str!("../../../key.pub");
static PRODUCT_ID: &str = "29c8e4a3-a2a6-411c-a4da-61b440be3f82";
static LEEWAY: u64 = 60;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Activation {
    #[serde(rename = "fp")]
    pub fingerprint: String,
    pub key: String,
    pub name: String,
    pub email: String,
    pub company: String,
    pub suspended: bool,
    pub aid: String,
    #[serde(alias = "eat")]
    pub exp: u64,
}

#[derive(Serialize)]
pub struct ActivationPayload {
    os: String,
    #[serde(rename = "osVersion")]
    os_version: String,
    fingerprint: String,
    hostname: String,
    #[serde(rename = "appVersion")]
    app_version: String,
    #[serde(rename = "userHash")]
    user_hash: String,
    #[serde(rename = "productId")]
    product_id: String,
    key: String,
}

#[derive(Deserialize)]
struct ServiceResponse {
    #[serde(rename = "activationToken")]
    token: Option<String>,
    #[serde(default)]
    code: String,
}

impl Activation {
    pub fn validate_key(key: &String) -> bool {
        if key.len() == 41 {
            key.chars().enumerate().all(|(i, c)| {
                if (i + 1) % 7 == 0 {
                    if c == '-' {
                        true
                    } else {
                        false
                    }
                } else {
                    if c.is_ascii_alphanumeric() {
                        true
                    } else {
                        false
                    }
                }
            })
        } else {
            false
        }
    }

    pub fn get_identity() -> (String, String, String, String) {
        let mut hasher_fp = Hasher::new();
        let mut system = System::new_all();
        system.refresh_all();

        hasher_fp.update(&system.physical_core_count().unwrap().to_be_bytes());
        hasher_fp.update(system.name().unwrap().as_bytes());
        hasher_fp.update(&system.total_memory().to_be_bytes());

        #[cfg(debug_assertions)]
        {
            let cpu = system.physical_core_count().unwrap();
            let os_ver = system.total_memory();
            let os = system.name().unwrap();

            println!("CPU count: {}\nRAM: {}\nOS: {}", &cpu, os_ver, os);
        }

        (
            system.host_name().unwrap(),
            hasher_fp.finalize().to_hex().to_string(),
            String::from(OS),
            system.os_version().unwrap(),
        )
    }

    fn client() -> Client {
        Client::builder()
            .https_only(true)
            .gzip(true)
            .user_agent(format!("SPD/{}", VERSION))
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap()
    }

    pub fn from_token(token: &String) -> Option<Activation> {
        match decode::<Activation>(
            token.as_str(),
            &DecodingKey::from_rsa_pem(PUBLIC_KEY.as_bytes()).unwrap(),
            &Validation {
                leeway: LEEWAY,
                validate_exp: false,
                algorithms: vec![Algorithm::RS256],
                ..Default::default()
            },
        ) {
            Ok(data) => Some(data.claims),
            Err(_) => None,
        }
    }

    pub fn load_token(path: &str) -> String {
        let file = Path::new(path);
        if file.exists() {
            match read_to_string(file) {
                Ok(token) => token,
                Err(_) => String::new(),
            }
        } else {
            String::new()
        }
    }

    fn payload(key: String) -> ActivationPayload {
        let (hostname, fingerprint, os, os_version) = Activation::get_identity();

        ActivationPayload {
            os,
            os_version,
            user_hash: fingerprint.clone(),
            fingerprint,
            hostname,
            app_version: String::from(VERSION),
            product_id: String::from(PRODUCT_ID),
            key: key.clone(),
        }
    }

    pub async fn activate(key: String) -> Result<(Activation, String), ActivationError> {
        let token: String = match Activation::client()
            .post("https://api.cryptlex.com/v3/activations")
            .header("Accept", "text/json")
            .json(&Activation::payload(key))
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                StatusCode::OK => (resp.json::<ServiceResponse>().await.unwrap())
                    .token
                    .unwrap(),
                StatusCode::BAD_REQUEST => {
                    let s_resp = resp.json::<ServiceResponse>().await.unwrap();

                    return if s_resp.code == "REVOKED_LICENSE" {
                        Err(ActivationError::RevokedLicense)
                    } else if s_resp.code == "INVALID_LICENSE_KEY" {
                        Err(ActivationError::InvalidKey)
                    } else if s_resp.code == "ACTIVATION_LIMIT_REACHED" {
                        Err(ActivationError::ActivationLimit)
                    } else {
                        Err(ActivationError::ActivationService)
                    };
                }
                _ => return Err(ActivationError::Unknown),
            },
            Err(err) => {
                return if err.is_timeout() {
                    Err(ActivationError::ConnectionTimeout)
                } else if err.is_connect() {
                    Err(ActivationError::Connection)
                } else {
                    Err(ActivationError::Unknown)
                }
            }
        };

        let activation = match Activation::from_token(&token) {
            Some(activation) => {
                if activation.exp
                    > (SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + LEEWAY)
                {
                    activation
                } else {
                    return Err(ActivationError::LicenseExpired);
                }
            }
            None => return Err(ActivationError::InvalidLicenseToken),
        };

        Ok((activation, token))
    }

    pub async fn deactivate(self) -> Option<Self> {
        match Activation::client()
            .delete(format!(
                "https://api.cryptlex.com/v3/activations/{}",
                &self.aid
            ))
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status() == StatusCode::NO_CONTENT {
                    None
                } else {
                    Some(self)
                }
            }
            Err(_) => Some(self),
        }
    }

    pub async fn verify(self) -> Result<(Activation, String), ActivationError> {
        Activation::activate(self.key.clone()).await
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum ActivationError {
    InvalidKeyFormat,
    InvalidKey,
    Connection,
    ConnectionTimeout,
    ActivationService,
    ActivationLimit,
    InvalidLicenseToken,
    LicenseExpired,
    RevokedLicense,
    Unknown,
}

impl ActivationError {
    pub fn code(&self) -> u8 {
        match *self {
            ActivationError::InvalidKeyFormat => 1,
            ActivationError::InvalidKey => 2,
            ActivationError::Connection => 3,
            ActivationError::ConnectionTimeout => 4,
            ActivationError::ActivationService => 5,
            ActivationError::ActivationLimit => 6,
            ActivationError::InvalidLicenseToken => 7,
            ActivationError::LicenseExpired => 8,
            ActivationError::RevokedLicense => 9,
            ActivationError::Unknown => 0,
        }
    }

    pub fn as_str(&self) -> &str {
        match *self {
            ActivationError::InvalidKeyFormat => "Incorrect key format",
            ActivationError::InvalidKey => "Invalid license key",
            ActivationError::Connection => "Connection error",
            ActivationError::ConnectionTimeout => "Connection timeout",
            ActivationError::ActivationService => "Activation service error",
            ActivationError::ActivationLimit => "Activation limit reached",
            ActivationError::InvalidLicenseToken => "Invalid license token",
            ActivationError::LicenseExpired => "License expired",
            ActivationError::RevokedLicense => "License has been revoked",
            ActivationError::Unknown => "Unknown error",
        }
    }
}
