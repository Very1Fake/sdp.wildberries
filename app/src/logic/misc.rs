use std::{ops::RangeInclusive, sync::Arc, time::Duration};

use rand::{thread_rng, Rng};
use reqwest::{cookie::Jar, header::HeaderMap, Client, Proxy, StatusCode, Url, Version};
use tokio::time::sleep;

use crate::{
    H_ACCEPT, H_ACCEPT_ENCODING, H_ACCEPT_LANGUAGE, H_CACHE_CONTROL, H_HOST, H_ORIGIN, H_PRAGMA,
    H_SEC_FETCH_DEST, H_SEC_FETCH_MODE, H_SEC_FETCH_SITE, H_TE, H_USER_AGENT, H_X_REQUESTED_WITH,
    H_X_SPA_VERSION,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Requests
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ResponseStatus {
    Timeout,
    ConnectionError,
}

impl ResponseStatus {
    pub fn to_string(&self, tier: &str) -> String {
        format!(
            "{} ({})",
            match &self {
                ResponseStatus::Timeout => String::from("Timeout"),
                ResponseStatus::ConnectionError => String::from("Connection Error"),
            },
            tier
        )
    }
}

pub enum ResponseMethod {
    GET,
    POST,
}

pub struct Response {
    pub method: ResponseMethod,
    pub version: Version,
    pub status: StatusCode,

    pub headers: HeaderMap,

    pub body: String,
}

pub enum RequestMethod<'a> {
    GET,
    POST(Option<&'a Vec<(String, String)>>),
}

pub async fn request<'a>(
    client: &mut Client,
    url: &str,
    method: RequestMethod<'a>,
    referer: &str,
    delay: u64,
) -> Result<Response, ResponseStatus> {
    #[cfg(debug_assertions)]
    println!("WAITING FOR GET: {}", url);
    sleep(Duration::from_millis(delay)).await;
    let resp_method;

    match match method {
        RequestMethod::GET => {
            resp_method = ResponseMethod::GET;

            #[cfg(debug_assertions)]
            println!("GET REQUEST ON: {}", url);
            client
                .get(url)
                .header("Host", H_HOST)
                .header("User-Agent", H_USER_AGENT)
                .header("Accept", H_ACCEPT)
                .header("Accept-Language", H_ACCEPT_LANGUAGE)
                .header("Accept-Encoding", H_ACCEPT_ENCODING)
                .header("Referer", referer)
                .header("x-requested-with", H_X_REQUESTED_WITH)
                .header("x-spa-version", H_X_SPA_VERSION)
                .header("DNT", 1)
                .header("Sec-Fetch-Dest", H_SEC_FETCH_DEST)
                .header("Sec-Fetch-Mode", H_SEC_FETCH_MODE)
                .header("Sec-Fetch-Site", H_SEC_FETCH_SITE)
                .header("Pragma", H_PRAGMA)
                .header("Cache-Control", H_CACHE_CONTROL)
                .header("TE", H_TE)
                .send()
                .await
        }
        RequestMethod::POST(form) => {
            resp_method = ResponseMethod::POST;

            #[cfg(debug_assertions)]
            println!("POST REQUEST ON: {}", url);
            let mut client = client
                .post(url)
                .header("Host", H_HOST)
                .header("User-Agent", H_USER_AGENT)
                .header("Accept", H_ACCEPT)
                .header("Accept-Language", H_ACCEPT_LANGUAGE)
                .header("Accept-Encoding", H_ACCEPT_ENCODING)
                .header("Referer", referer)
                .header("x-requested-with", H_X_REQUESTED_WITH)
                .header("x-spa-version", H_X_SPA_VERSION)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Origin", H_ORIGIN)
                .header("DNT", 1)
                .header("Sec-Fetch-Dest", H_SEC_FETCH_DEST)
                .header("Sec-Fetch-Mode", H_SEC_FETCH_MODE)
                .header("Sec-Fetch-Site", H_SEC_FETCH_SITE)
                .header("Pragma", H_PRAGMA)
                .header("Cache-Control", H_CACHE_CONTROL)
                .header("TE", H_TE);

            match form {
                Some(data) => client = client.form(data),
                None => client = client.header("Content-Length", "0"),
            }

            client.send().await
        }
    } {
        Ok(resp) => {
            let mut result = Response {
                method: resp_method,
                version: resp.version(),
                status: resp.status(),
                headers: resp.headers().clone(),
                body: String::new(),
            };

            match resp.text().await {
                Ok(text) => result.body = text,
                Err(err) => {
                    return if err.is_timeout() {
                        Err(ResponseStatus::Timeout)
                    } else {
                        Err(ResponseStatus::ConnectionError)
                    }
                }
            }

            Ok(result)
        }
        Err(err) => {
            return if err.is_timeout() {
                Err(ResponseStatus::Timeout)
            } else {
                Err(ResponseStatus::ConnectionError)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Requests client
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn client(proxy: Option<String>, cookies: Option<&[(String, String, String)]>) -> Client {
    let mut client = Client::builder()
        .tcp_keepalive(Some(Duration::from_secs(4)))
        .timeout(Duration::from_secs(8))
        .cookie_store(true)
        .user_agent(H_USER_AGENT)
        .gzip(true)
        .https_only(true)
        .http1_title_case_headers();

    match proxy {
        Some(address) => client = client.proxy(Proxy::all(format!("https://{}", address)).unwrap()),
        None => (),
    }

    match cookies {
        Some(val) => {
            let jar = Jar::default();
            for i in val {
                jar.add_cookie_str(
                    &format!("{}={}; Domain={}", i.0, i.1, i.2),
                    &format!("https://{}", i.2).parse::<Url>().unwrap(),
                );
            }
            client = client.cookie_provider(Arc::new(jar))
        }
        None => (),
    }

    client.build().unwrap()
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// String tools
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn retrieve(text: &String, start: &str, end: &str) -> Option<String> {
    match text.find(start) {
        Some(mut i) => {
            i += start.len();

            if end.is_empty() {
                Some(text[i..].to_string())
            } else {
                match text[i..].find(end) {
                    Some(j) => Some(text[i..i + j].to_string()),
                    None => None,
                }
            }
        }
        None => None,
    }
}

pub fn rand_millis(range: RangeInclusive<u64>) -> u64 {
    thread_rng().gen_range(range) * 10
}
