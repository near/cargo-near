use reqwest::Client;
use std::{env, process::Command, str};
use tracing::debug;

#[derive(Debug, serde::Serialize)]
struct MixpanelProperties {
    token: String,
    cli_version: String,
    os: String,
}

#[derive(Debug, serde::Serialize)]
struct TrackingData {
    event: String,
    properties: MixpanelProperties,
}

fn trim_newline(s: String) -> String {
    let mut result = s;

    if result.ends_with('\n') {
        result.pop();
        if result.ends_with('\r') {
            result.pop();
        }
    }

    result
}

pub(crate) fn track_usage() {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "near --version"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("near --version")
            .output()
            .expect("failed to execute process")
    };

    let cli_version: String = match str::from_utf8(&output.stdout) {
        Ok(v) => v.split(" ").collect::<Vec<_>>()[1].to_string(),
        Err(_) => "unknown".to_string(),
    };

    let properties = MixpanelProperties {
        token: "24177ef1ec09ffea5cb6f68909c66a61".to_string(),
        cli_version: trim_newline(cli_version),
        os: env::consts::OS.to_string(),
    };
    let tracking_data = TrackingData {
        event: "CNN".to_string(),
        properties,
    };

    let client = Client::new();

    if let Err(_) = tokio::runtime::Runtime::new().unwrap().block_on(
        client
            .post("https://api.mixpanel.com/track")
            .json(&tracking_data)
            .send(),
    ) {
        debug!("Can't send tracking usage event")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking() {
        let thread_handle = std::thread::Builder::new().spawn(|| track_usage()).unwrap();

        if let Err(e) = thread_handle.join() {
            panic!("{:?}", e);
        }
    }
}
