use crate::*;

#[tauri::command]
pub async fn test_byok_provider_v2(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    ensure_byok_config(&config)?;
    Ok(ProviderOperationResult { ok: true, status: format!("{} config valid", provider_label(&config.provider_id)), detail: "endpoint, model, and session key are present".to_string() })
}
