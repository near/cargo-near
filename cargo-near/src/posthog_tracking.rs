use reqwest::{header::HeaderMap, Client};
use serde::Serialize;
use std::{env, str};
use tracing_indicatif::span_ext::IndicatifSpanExt;

const SEND_TRACKING_REQUEST_ERROR: &str = "Can't send tracking usage event";

#[derive(Debug, Serialize)]
struct PosthogProperties {
    language: String,
    pkg_version: String,
    os: String,
}

#[derive(Debug, Serialize)]
struct TrackingData {
    api_key: String,
    event: String,
    distinct_id: String,
    properties: PosthogProperties,
}

#[tracing::instrument(
    target = "tracing_instrument",
    name = "Sending a tracking request using Mixpanel via url"
)]
pub(crate) fn track_usage() {
    let properties = PosthogProperties {
        language: "rs".to_string(),
        pkg_version: env!("CARGO_PKG_VERSION").to_string(),
        os: env::consts::OS.to_string(),
    };
    let tracking_data = TrackingData {
        api_key: "phc_95PGQnbyatmj2TBRPWYfhbHfqB6wgZj5QRL8WY9gW20".to_string(),
        distinct_id: "cargo-near".to_string(),
        event: "contract".to_string(),
        properties,
    };
    let serialized_data = serde_json::to_vec(&tracking_data).unwrap();

    let client = Client::new();

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to broadcast the tracking data, learn more https://mixpanel.com/"
    );
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "HTTP GET https://api.mixpanel.com/track",
    );

    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());

    if tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            client
                .post("https://eu.i.posthog.com/capture")
                .body(serialized_data)
                .headers(headers)
                .send(),
        )
        .inspect(|response| {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON Response:\n{}",
                near_cli_rs::common::indent_payload(&format!("{:#?}", response))
            );
        })
        .is_err()
    {
        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "JSON RPC Response:\n{}",
            near_cli_rs::common::indent_payload(SEND_TRACKING_REQUEST_ERROR)
        );
        tracing::debug!(SEND_TRACKING_REQUEST_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_tracking() {
        track_usage();
        assert!(!logs_contain(SEND_TRACKING_REQUEST_ERROR));
    }
}
