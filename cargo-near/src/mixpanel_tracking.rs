use base64::{engine::general_purpose, Engine as _};
use reqwest::{header::HeaderMap, Client};
use serde::Serialize;
use std::{env, str};
use tracing::debug;

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

pub(crate) fn track_usage() {
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

    let mut headers = HeaderMap::new();
    headers.insert("accept", "text/plain".parse().unwrap());
    headers.insert("content-type", "application/json".parse().unwrap());

    if tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            client
                .get("https://api.mixpanel.com/track")
                .query(&[("data", base64_encoded_data)])
                .headers(headers)
                .send(),
        )
        .is_err()
    {
        debug!(SEND_TRACKING_REQUEST_ERROR)
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
