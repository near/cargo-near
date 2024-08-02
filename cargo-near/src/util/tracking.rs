use reqwest::Client;
use std::env;
use tracing::debug;

#[derive(Debug, serde::Serialize)]
struct MixpanelProperties {
  token: String,
  os: String,
}

#[derive(Debug, serde::Serialize)]
struct TrackingData {
    event: String,
    properties: MixpanelProperties,
}

pub(crate) fn track_usage() {
    let properties = MixpanelProperties {
      token: "24177ef1ec09ffea5cb6f68909c66a61".to_string(),
      os: env::consts::OS.to_string(),
    };
    let tracking_data = TrackingData {
        event: "CNN".to_string(),
        properties,
    };

    let client = Client::new();

    if let Err(_) = tokio::runtime::Runtime::new()
      .unwrap()
      .block_on(client
        .post("https://api.mixpanel.com/track")
        .json(&tracking_data)
        .send())
    {
      debug!("Can't send tracking usage event")
    }
}
