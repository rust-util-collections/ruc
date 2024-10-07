#![allow(missing_docs)]

use crate::*;
use reqwest::{
    StatusCode, {Client, ClientBuilder},
};
use std::{env, sync::LazyLock, time::Duration};

static TIME_OUT: LazyLock<Duration> = LazyLock::new(|| {
    let secs = if let Ok(t) = env::var("RUC_HTTP_TIMEOUT") {
        pnk!(t.parse::<u64>())
    } else {
        3
    };
    Duration::from_secs(secs)
});

pub async fn http_get(
    url: &str,
    headers: Option<&[(&'static str, &'static str)]>,
) -> Result<(StatusCode, Vec<u8>)> {
    let mut builder = http_cli().get(url);
    if let Some(headers) = headers {
        for (h, v) in headers.iter().copied() {
            builder = builder.header(h, v);
        }
    }
    let resp = builder.send().await.c(d!(url))?;
    let code = resp.status();
    let msg = resp.bytes().await.c(d!())?;
    Ok((code, msg.into()))
}

pub async fn http_get_ret_string(
    url: &str,
    headers: Option<&[(&'static str, &'static str)]>,
) -> Result<(StatusCode, String)> {
    http_get(url, headers)
        .await
        .c(d!())
        .map(|(code, msg)| (code, String::from_utf8_lossy(&msg).into_owned()))
}

pub async fn http_post(
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
    let resp = builder.body(body.to_owned()).send().await.c(d!(url))?;
    let code = resp.status();
    let msg = resp.bytes().await.c(d!())?;
    Ok((code, msg.into()))
}

/// # Example
///
/// ```ignore
///    let url = "http://......";
///    let req = "...".as_bytes();
///    http_post_ret_string(
///        url,
///        req,
///        Some(&[("Content-Type", "application/json")]),
///    )
///    .await
///    .c(d!(url))
/// ```
pub async fn http_post_ret_string(
    url: &str,
    body: &[u8],
    headers: Option<&[(&'static str, &'static str)]>,
) -> Result<(StatusCode, String)> {
    http_post(url, body, headers)
        .await
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
