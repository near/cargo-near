use base64::{engine::general_purpose, Engine as _};
use reqwest::{header::HeaderMap, Client};
use serde::Serialize;
use std::{env, str};
use tracing_indicatif::span_ext::IndicatifSpanExt;

const SEND_TRACKING_REQUEST_ERROR: &str = "Can't send tracking usage event";

#[derive(Debug, Serialize)]
struct MixpanelProperties {
    token: String,
    pkg_version: String,
    os: String,
}

#[derive(Debug, Serialize)]
struct TrackingData {
    event: String,
    properties: MixpanelProperties,
}

#[tracing::instrument(
    target = "tracing_instrument",
    name = "Sending a tracking request using Mixpanel via url"
)]
pub(crate) fn track_usage() {
    tracing::Span::current().pb_set_message("https://api.mixpanel.com/track ...");
    tracing::info!(target: "near_teach_me", "https://api.mixpanel.com/track ...");

    let properties = MixpanelProperties {
        token: "24177ef1ec09ffea5cb6f68909c66a61".to_string(),
        pkg_version: env!("CARGO_PKG_VERSION").to_string(),
        os: env::consts::OS.to_string(),
    };
    let tracking_data = TrackingData {
        event: "CNN".to_string(),
        properties,
    };
    let serialized_data = serde_json::to_vec(&tracking_data).unwrap();
    let base64_encoded_data = general_purpose::STANDARD.encode(&serialized_data);

    let client = Client::new();

    let request_payload = serde_json::json!({
       "base64_encoded_tracking_data": base64_encoded_data
    });
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
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "JSON Body:\n{}",
        near_cli_rs::common::indent_payload(&format!("{:#}", request_payload))
    );

    let mut headers = HeaderMap::new();
    headers.insert("accept", "text/plain".parse().unwrap());
    headers.insert("content-type", "application/json".parse().unwrap());

    if tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            client
                .get("https://api.mixpanel.com/track")
                .query(&[("data", &base64_encoded_data)])
                .headers(headers.clone())
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
