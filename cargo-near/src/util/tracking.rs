use chrono::prelude::*;
use reqwest::Client;
use rustc_version_runtime::version;
use std::env;

#[derive(Debug, serde::Serialize)]
struct MixpanelProperties {
  token: String,
  rustc_version: String,
  os: String,
  timestamp: String,
}

#[derive(Debug, serde::Serialize)]
struct TrackingData {
    event: String,
    properties: MixpanelProperties,
}

pub(crate) async fn track_usage() {
    let rustc_version = version();
    let properties = MixpanelProperties {
      token: "24177ef1ec09ffea5cb6f68909c66a61".to_string(),
      rustc_version: format!(
          "{}.{}.{}",
          rustc_version.major, rustc_version.minor, rustc_version.patch
      ),
      os: env::consts::OS.to_string(),
      timestamp: Utc::now().to_string(),
    };
    let tracking_data = TrackingData {
        event: "CNN".to_string(),
        properties,
    };

    let client = Client::new();

    if let Ok(_) = client
        .post("https://api.mixpanel.com/track")
        .json(&tracking_data)
        .send()
        .await
    {
    } else {
        println!("Can't send tracking usage event");
    }
}
