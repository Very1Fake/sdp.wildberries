use std::{ops::RangeInclusive, time::Duration};

use rand::{thread_rng, Rng};
use reqwest::{Client, Response};
use tokio::time::sleep;

use crate::{
    H_ACCEPT, H_ACCEPT_ENCODING, H_ACCEPT_LANGUAGE, H_CONNECTION, H_HOST, H_ORIGIN,
    H_UPGRADE_INSECURE_REQUESTS, H_USER_AGENT,
};

use super::task::{Task, TaskProgress};

pub async fn request<'a>(
    client: &mut Client,
    url: &str,
    form: Option<&'a [(&'a str, &'a str)]>,
    referer: &str,
    delay: u64,
    tier: &str,
) -> Result<Response, Option<TaskProgress>> {
    #[cfg(debug_assertions)]
    println!("WAITING FOR GET: {}", url);
    sleep(Duration::from_millis(delay)).await;

    for _attempt in 0u8..=3 {
        match match form {
            Some(data) => {
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
                if Task::variti_ban(&resp) {
                    continue;
                } else {
                    return Ok(resp);
                }
            }
            Err(err) => {
                return if err.is_timeout() {
                    Err(Some(TaskProgress::Error(String::from("Timeout"))))
                } else {
                    Err(Some(TaskProgress::Error(String::from(
                        "Connection error. Try again later",
                    ))))
                }
            }
        }
    }

    Err(Some(TaskProgress::Error(format!("Variti Ban ({})", tier))))
}

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
