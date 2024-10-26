#![allow(missing_docs)]

use crate::*;
use reqwest::{
    blocking::{Client, ClientBuilder},
    StatusCode,
};
use std::{env, sync::LazyLock, time::Duration};

static TIME_OUT: LazyLock<Duration> = LazyLock::new(|| {
    let default = 3;
    let secs = if let Ok(t) = env::var("RUC_HTTP_TIMEOUT") {
        t.parse::<u8>().unwrap_or(default)
    } else {
        default
    };
    Duration::from_secs(secs as u64)
});

pub fn get(
    url: &str,
    headers: Option<&[(&'static str, &'static str)]>,
) -> Result<(StatusCode, Vec<u8>)> {
    let mut builder = http_cli().get(url);
    if let Some(headers) = headers {
        for (h, v) in headers.iter().copied() {
            builder = builder.header(h, v);
        }
    }
    let resp = builder.send().c(d!(url))?;
    let code = resp.status();
    let msg = resp.bytes().c(d!())?;
    Ok((code, msg.into()))
}

pub fn get_resp_str(
    url: &str,
    headers: Option<&[(&'static str, &'static str)]>,
) -> Result<(StatusCode, String)> {
    get(url, headers)
        .c(d!())
        .map(|(code, msg)| (code, String::from_utf8_lossy(&msg).into_owned()))
}

pub fn post(
    url: &str,
    body: &[u8],
    headers: Option<&[(&'static str, &'static str)]>,
) -> Result<(StatusCode, Vec<u8>)> {
    let mut builder = http_cli().post(url);
    if let Some(headers) = headers {
        for (h, v) in headers.iter().copied() {
            builder = builder.header(h, v);
        }
    }
    let resp = builder.body(body.to_owned()).send().c(d!(url))?;
    let code = resp.status();
    let msg = resp.bytes().c(d!())?;
    Ok((code, msg.into()))
}

/// # Example
///
/// ```no_run
///    use ruc::*;
///
///    let url = "http://......";
///    let req = "...".as_bytes();
///    ruc::http::post_resp_str(
///        url,
///        req,
///        Some(&[("Content-Type", "application/json")]),
///    )
///    .unwrap();
/// ```
pub fn post_resp_str(
    url: &str,
    body: &[u8],
    headers: Option<&[(&'static str, &'static str)]>,
) -> Result<(StatusCode, String)> {
    post(url, body, headers)
        .c(d!())
        .map(|(code, msg)| (code, String::from_utf8_lossy(&msg).into_owned()))
}

fn http_cli() -> Client {
    ClientBuilder::new()
        .timeout(*TIME_OUT)
        .http1_only()
        .build()
        .unwrap()
}
