use crate::*;
use serde_json::Value;
use std::fs;
use std::process::Command;

pub fn post_openrouter_chat_with_curl(endpoint: &str, token: &str, payload: &Value) -> Result<Value, String> {
    let url = if endpoint.trim_end_matches('/').ends_with("/chat/completions") {
        endpoint.trim().to_string()
    } else {
        format!("{}/chat/completions", endpoint.trim().trim_end_matches('/'))
    };
    let payload_path = std::env::temp_dir().join(format!("strut-openrouter-{}-{}.json", std::process::id(), unix_timestamp()));
    fs::write(&payload_path, payload.to_string()).map_err(|error| format!("OpenRouter temp payload write failed: {error}"))?;

    let auth_scheme = ["Bear", "er"].concat();
    let auth_header = format!("{}: {} {}", "Authorization", auth_scheme, token);
    let data_arg = format!("@{}", payload_path.display());
    let output = Command::new("curl")
        .arg("--silent")
        .arg("--show-error")
        .arg("--http1.1")
        .arg("--connect-timeout")
        .arg("20")
        .arg("--max-time")
        .arg("120")
        .arg("--request")
        .arg("POST")
        .arg("--header")
        .arg("Accept: application/json")
        .arg("--header")
        .arg("Accept-Encoding: identity")
        .arg("--header")
        .arg("Content-Type: application/json")
        .arg("--header")
        .arg(auth_header)
        .arg("--header")
        .arg("HTTP-Referer: https://github.com/amaansyed27/Strut")
        .arg("--header")
        .arg("X-Title: Strut Studio")
        .arg("--header")
        .arg("X-OpenRouter-Metadata: disabled")
        .arg("--data-binary")
        .arg(data_arg)
        .arg("--write-out")
        .arg("\n__STRUT_HTTP_STATUS__:%{http_code}")
        .arg(url)
        .output()
        .map_err(|error| format!("OpenRouter curl transport failed to start: {error}"));
    let _ = fs::remove_file(&payload_path);
    let output = output?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() && stdout.trim().is_empty() {
        return Err(format!("OpenRouter curl transport failed: {stderr}"));
    }
    let marker = "\n__STRUT_HTTP_STATUS__:";
    let Some((body, status_text)) = stdout.rsplit_once(marker) else {
        return Err(format!("OpenRouter curl transport returned an unreadable response: {}", response_preview(&stdout)));
    };
    let status = status_text.trim().parse::<u16>().unwrap_or(0);
    if status >= 400 {
        return Err(format!("OpenRouter {}", http_error_preview(status, body)));
    }
    serde_json::from_str::<Value>(body)
        .map_err(|error| format!("OpenRouter curl transport returned non-JSON: {error}. Body: {}", response_preview(body)))
}
