use crate::AssistantResult;

/// API-economy default: do not run automatic post-generation repair calls.
///
/// Strut now pushes quality requirements into the first system prompt instead of
/// spending extra provider requests after a result arrives. This keeps the normal
/// generation path to one provider request for free-tier BYOK accounts. Manual
/// user edits can still be sent as explicit follow-up prompts.
pub fn quality_repair_prompt(_user_prompt: &str, _result: &AssistantResult) -> Option<String> {
    None
}
