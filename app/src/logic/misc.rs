use std::{ops::RangeInclusive, time::Duration};

use rand::{thread_rng, Rng};
use reqwest::{header::HeaderMap, Client, StatusCode, Version};
use tokio::time::sleep;

use crate::{
    H_ACCEPT, H_ACCEPT_ENCODING, H_ACCEPT_LANGUAGE, H_CONNECTION, H_HOST, H_ORIGIN,
    H_UPGRADE_INSECURE_REQUESTS, H_USER_AGENT,
};

pub enum BanKind {
    Variti,
    DDOSGuard,
}

pub enum RespStatus {
    Timeout,
    ConnectionError,
    ProtectionBan(BanKind),
}

impl RespStatus {
    pub fn task_progress(&self, tier: &str) -> String {
        match &self {
            RespStatus::Timeout => String::from("Timeout"),
            RespStatus::ConnectionError => String::from("Connection Error"),
            RespStatus::ProtectionBan(kind) => {
                if !tier.is_empty() {
                    match kind {
                        BanKind::Variti => format!("Variti Ban ({})", tier),
                        BanKind::DDOSGuard => format!("Protection Ban ({})", tier),
                    }
                } else {
                    String::from("Protection Ban")
                }
            }
        }
    }
}

pub enum Method {
    GET,
    POST,
}

pub struct Resp {
    pub method: Method,
    pub version: Version,
    pub status: StatusCode,

    pub headers: HeaderMap,

    pub body: String,
}

impl Resp {
    pub fn ban_check(&self) -> Option<BanKind> {
        match self.headers.get("Server").unwrap().to_str() {
            // Variti protection
            Ok(server) => {
                if server.starts_with("Variti") {
                    return Some(BanKind::Variti);
                }
            }
            Err(_) => {}
        }
        match self.headers.get("Server").unwrap().to_str() {
            // DDOS Guard protection
            Ok(server) => {
                if server == "ddos-guard" {
                    if self.body.contains("<title>DDOS-GUARD</title>") {
                        return Some(BanKind::DDOSGuard);
                    }

                    if let Method::GET = self.method {
                        if self.body.is_empty() {
                            return Some(BanKind::DDOSGuard);
                        }
                    }
                }
            }
            Err(_) => {}
        }

        None
    }
}

pub async fn request<'a>(
    client: &mut Client,
    url: &str,
    form: Option<&'a [(&'a str, &'a str)]>,
    referer: &str,
    delay: u64,
) -> Result<Resp, RespStatus> {
    #[cfg(debug_assertions)]
    println!("WAITING FOR GET: {}", url);
    sleep(Duration::from_millis(delay)).await;
    let mut method;

    for _attempt in 0u8..=3 {
        match match form {
            Some(data) => {
                method = Method::POST;

                #[cfg(debug_assertions)]
                println!("POST REQUEST ON: {}", url);
                let mut client = client
                    .post(url)
                    .header("Host", H_HOST)
                    .header("User-Agent", H_USER_AGENT)
                    .header("Accept", H_ACCEPT)
                    .header("Accept-Language", H_ACCEPT_LANGUAGE)
                    .header("Accept-Encoding", H_ACCEPT_ENCODING)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Origin", H_ORIGIN)
                    .header("Connection", H_CONNECTION)
                    .header("Referer", referer)
                    .header("Upgrade-Insecure-Requests", H_UPGRADE_INSECURE_REQUESTS);

                if !data.is_empty() {
                    client = client.form(data);
                }

                client.send().await
            }
            None => {
                method = Method::GET;

                #[cfg(debug_assertions)]
                println!("GET REQUEST ON: {}", url);
                client
                    .get(url)
                    .header("Host", H_HOST)
                    .header("User-Agent", H_USER_AGENT)
                    .header("Accept", H_ACCEPT)
                    .header("Accept-Language", H_ACCEPT_LANGUAGE)
                    .header("Accept-Encoding", H_ACCEPT_ENCODING)
                    .header("Connection", H_CONNECTION)
                    .header("Referer", referer)
                    .header("Upgrade-Insecure-Requests", H_UPGRADE_INSECURE_REQUESTS)
                    .send()
                    .await
            }
        } {
            Ok(resp) => {
                let mut result = Resp {
                    method,
                    version: resp.version(),
                    status: resp.status(),
                    headers: resp.headers().clone(),
                    body: String::new(),
                };

                match resp.text().await {
                    Ok(text) => result.body = text,
                    Err(err) => {
                        return if err.is_timeout() {
                            Err(RespStatus::Timeout)
                        } else {
                            Err(RespStatus::ConnectionError)
                        }
                    }
                }

                match result.ban_check() {
                    Some(_) => continue,
                    None => return Ok(result),
                }
            }
            Err(err) => {
                return if err.is_timeout() {
                    Err(RespStatus::Timeout)
                } else {
                    Err(RespStatus::ConnectionError)
                }
            }
        }
    }

    Err(RespStatus::ProtectionBan(BanKind::DDOSGuard))
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// String tools
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn retrieve(text: &String, start: &str, end: &str) -> Option<String> {
    match text.find(start) {
        Some(mut i) => {
            i += start.len();

            match text[i..].find(end) {
                Some(j) => Some(text[i..i + j].to_string()),
                None => None,
            }
        }
        None => None,
    }
}

pub fn rand_millis(range: RangeInclusive<u64>) -> u64 {
    thread_rng().gen_range(range) * 10
}
