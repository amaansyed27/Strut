use std::net::IpAddr;
use std::time::Duration;
use std::path::PathBuf;
use crate::*;

pub fn provider_config_path() -> Result<PathBuf, String> {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return Ok(PathBuf::from(appdata).join("Strut").join("providers").join("byok.json"));
    }
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home).join("strut").join("providers").join("byok.json"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".config").join("strut").join("providers").join("byok.json"));
    }
    Err("could not resolve a local config directory".to_string())
}

pub fn ensure_byok_config(config: &ByokProviderConfig) -> Result<(), String> {
    if config.endpoint.trim().is_empty() { return Err("provider endpoint is required".to_string()); }
    ensure_safe_endpoint(&config.endpoint)?;
    if config.model.trim().is_empty() { return Err("provider model is required".to_string()); }
    if config.api_key.as_deref().unwrap_or_default().trim().is_empty() && !local_compatible_provider(config) {
        return Err(format!("{} credential is required", provider_label(&config.provider_id)));
    }
    Ok(())
}

fn local_compatible_provider(config: &ByokProviderConfig) -> bool {
    if config.provider_id != "openai-compatible" { return false; }
    let Ok(url) = reqwest::Url::parse(config.endpoint.trim()) else { return false; };
    let Some(host) = url.host_str() else { return false; };
    host.eq_ignore_ascii_case("localhost") || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

pub fn ensure_safe_endpoint(endpoint: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(endpoint.trim()).map_err(|error| format!("provider endpoint is not a valid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") { return Err("provider endpoint must use http or https".to_string()); }
    let host = url.host_str().ok_or_else(|| "provider endpoint must include a host".to_string())?;
    if host.eq_ignore_ascii_case("localhost") { return Ok(()); }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if ip.is_loopback() { return Ok(()); }
        if is_blocked_internal_ip(ip) {
            return Err("provider endpoint cannot target private, link-local, or unspecified network addresses unless it is loopback".to_string());
        }
    }
    Ok(())
}

pub fn is_blocked_internal_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_private() || ip.is_link_local() || ip.is_unspecified() || ip.is_broadcast(),
        IpAddr::V6(ip) => ip.is_unique_local() || ip.is_unicast_link_local() || ip.is_unspecified(),
    }
}

pub fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder().timeout(Duration::from_secs(30)).build().map_err(|error| error.to_string())
}

pub fn endpoint_base(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

pub fn provider_label(provider_id: &str) -> &'static str {
    match provider_id {
        "openai" => "OpenAI",
        "anthropic" => "Anthropic",
        "gemini" => "Gemini",
        "openrouter" => "OpenRouter",
        "azure-openai" => "Azure OpenAI",
        "openai-compatible" => "OpenAI Compatible",
        _ => "Provider",
    }
}

pub fn http_error_preview(status: u16, body: &str) -> String {
    let preview: String = body.chars().take(260).collect();
    if preview.trim().is_empty() { format!("HTTP {status}") } else { format!("HTTP {status}: {preview}") }
}
