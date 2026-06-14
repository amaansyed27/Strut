import re

with open('apps/studio/src-tauri/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Add ASSISTANT_ROUTER_SYSTEM_PROMPT
prompt = '''const ASSISTANT_ROUTER_SYSTEM_PROMPT: &str = r#"You are the Strut generation router. The user will provide a prompt. You must output exactly ONE valid JSON object and nothing else. The JSON object must match this schema:
{
    "type": "Chat",
    "message": "Your response message"
}
OR
{
    "type": "DocumentCreated",
    "message": "A summary of what you created",
    "document": { ... valid strut document ... }
}
OR
{
    "type": "DocumentUpdated",
    "message": "A summary of what you updated",
    "document": { ... valid strut document ... }
}
Do not use markdown blocks around the JSON."#;
'''
if 'ASSISTANT_ROUTER_SYSTEM_PROMPT' not in content:
    content = content.replace('const GENERATION_PLAN_SYSTEM_PROMPT: &str =', prompt + '\nconst GENERATION_PLAN_SYSTEM_PROMPT: &str =')

# 2. Replace ChatAnswer and GeneratedCharacter with AssistantResult
assistant_result = '''#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum AssistantResult {
    Chat {
        message: String,
        #[serde(default)]
        source: String,
    },
    DocumentCreated {
        message: String,
        #[serde(default)]
        source: String,
        document: strut_core::Document,
    },
    DocumentUpdated {
        message: String,
        #[serde(default)]
        source: String,
        document: strut_core::Document,
    },
}
'''
content = re.sub(r'#\[derive\(Debug, Clone, Serialize\)\]\s*struct GeneratedCharacter \{.*?\s*operation_count: Option<usize>,\s*\}\s*#\[derive\(Debug, Clone, Serialize\)\]\s*#\[serde\(rename_all = "camelCase"\)\]\s*struct ChatAnswer \{.*?\s*message: String,\s*\}', assistant_result, content, flags=re.DOTALL)

# 3. Clean up GenerationContext and GenerationContextMessage
content = re.sub(r'#\[derive\(Debug, Clone, Deserialize\)\]\s*#\[serde\(rename_all = "camelCase"\)\]\s*struct GenerationContext \{.*?current_document: Option<strut_core::Document>,\s*\}\s*#\[derive\(Debug, Clone, Deserialize\)\]\s*#\[serde\(rename_all = "camelCase"\)\]\s*struct GenerationContextMessage \{.*?attachments: Option<Vec<String>>,\s*\}', 
'''#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationContext {
    project_name: Option<String>,
    active_chat_title: Option<String>,
    current_document_summary: Option<String>,
}''', content, flags=re.DOTALL)

# 4. Remove RequestIntent enum
content = re.sub(r'#\[derive\(Debug, Clone, Copy, PartialEq, Eq\)\]\s*enum RequestIntent \{\s*Conversation,\s*Generate,\s*\}\s*', '', content)

# 5. Replace chat_with_provider and generate_character with assistant_message
assistant_message = '''#[tauri::command]
async fn assistant_message(
    prompt: String,
    provider: Option<GenerationProvider>,
    references: Option<Vec<ReferenceImageInput>>,
    context: Option<GenerationContext>,
) -> Result<AssistantResult, String> {
    let references = references.unwrap_or_default();
    let provider = provider.ok_or_else(|| {
        "Select a real local CLI, Ollama, or BYOK provider before generating.".to_string()
    })?;

    let mut system_prompt = ASSISTANT_ROUTER_SYSTEM_PROMPT.to_string();
    if let Some(context) = context.as_ref() {
        if let Some(project_name) = &context.project_name {
            system_prompt.push_str(&format!("\\nProject: {project_name}"));
        }
        if let Some(chat_title) = &context.active_chat_title {
            system_prompt.push_str(&format!("\\nChat: {chat_title}"));
        }
        if let Some(summary) = &context.current_document_summary {
            system_prompt.push_str(&format!("\\n\\nThe scene currently contains this document:\\n{summary}"));
        }
    }

    let user_prompt = prompt.clone();

    let text = match provider.mode.as_str() {
        "byok" => {
            let config = provider
                .byok
                .ok_or_else(|| "BYOK provider config missing".to_string())?;
            byok_generate_text(&user_prompt, &config, &references, Some(&system_prompt)).await?
        }
        "local" => {
            let adapter_id = provider
                .local_adapter_id
                .ok_or_else(|| "Select a local CLI or Ollama adapter".to_string())?;
            chat_with_local_adapter(&adapter_id, &user_prompt, &references).await?
        }
        _ => return Err("Unknown provider mode".to_string()),
    };

    if let Ok(result) = serde_json::from_str::<AssistantResult>(&text) {
        return Ok(result);
    }
    
    if let Some(start) = text.find("```json\\n") {
        if let Some(end) = text[start + 8..].find("```") {
            let json_str = &text[start + 8..start + 8 + end];
            if let Ok(result) = serde_json::from_str::<AssistantResult>(json_str) {
                return Ok(result);
            }
        }
    }

    Ok(AssistantResult::Chat {
        message: text.clone(),
        source: "raw".to_string(),
    })
}'''
content = re.sub(r'#\[tauri::command\]\s*async fn generate_character\(.*?#\[tauri::command\]\s*async fn chat_with_provider\(.*?_ => Err\("Unknown provider mode"\.to_string\(\)\),\s*\}[\n\s]*\}', assistant_message, content, flags=re.DOTALL)

# 6. Delete all the dead code functions
content = re.sub(r'fn run_local_cli_chat_command\(.*?\s*run_command_with_stdin\(command, &args, &env, None, input, timeout\)\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn local_chat_args\(.*?\s*\}\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn contextual_generation_prompt\(.*?\s*base\s*\}\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn reference_message\(.*?\}\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn classify_request_intent\(.*?\s*RequestIntent::Conversation\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn chat_system_prompt\(.*?\s*Ok\(system_prompt\)\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'async fn generate_document_with_local_adapter\(.*?\s*\}\s*\}\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn generate_document_with_sprite_python\(.*?\s*\}\s*\}\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn sprite_python_example_for_prompt\(.*?\s*\.to_string\(\)\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'async fn generate_document_with_ollama\(.*?\s*\}\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn cli_assistant_text\(.*?\s*\}\s*\}', '', content, flags=re.DOTALL)
content = re.sub(r'fn collect_text_fields\(.*?\s*\}\s*\}', '', content, flags=re.DOTALL)

# 7. Add system_prompt to byok_generate_text and the providers
content = content.replace(
'''async fn byok_generate_text(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    Ok(match config.provider_id.as_str() {
        "anthropic" => anthropic_message(prompt, config, references).await?,
        "gemini" => gemini_generate_content(prompt, config, references).await?,
        _ => openai_compatible_chat(prompt, config, references).await?,
    })
}''',
'''async fn byok_generate_text(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {
    Ok(match config.provider_id.as_str() {
        "anthropic" => anthropic_message(prompt, config, references, system_prompt).await?,
        "gemini" => gemini_generate_content(prompt, config, references, system_prompt).await?,
        _ => openai_compatible_chat(prompt, config, references, system_prompt).await?,
    })
}''')

# Update openai
content = content.replace('''async fn openai_compatible_chat(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {''', '''async fn openai_compatible_chat(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {''')
content = content.replace('''{"role": "system", "content": GENERATION_PLAN_SYSTEM_PROMPT},''', '''{"role": "system", "content": system_prompt.unwrap_or(GENERATION_PLAN_SYSTEM_PROMPT)},''')

# Update anthropic
content = content.replace('''async fn anthropic_message(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {''', '''async fn anthropic_message(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {''')
content = content.replace('''"system": GENERATION_PLAN_SYSTEM_PROMPT,''', '''"system": system_prompt.unwrap_or(GENERATION_PLAN_SYSTEM_PROMPT),''')

# Update gemini
content = content.replace('''async fn gemini_generate_content(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {''', '''async fn gemini_generate_content(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {''')
content = content.replace('''"text": format!("{GENERATION_PLAN_SYSTEM_PROMPT}\\nPrompt: {}"''', '''"text": format!("{}\\nPrompt: {}", system_prompt.unwrap_or(GENERATION_PLAN_SYSTEM_PROMPT)''')

# 8. Fix chat_with_local_adapter
chat_local = '''async fn chat_with_local_adapter(
    adapter_id: &str,
    prompt: &str,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    let definition = local_adapter_definitions()
        .into_iter()
        .find(|definition| definition.id == adapter_id)
        .ok_or_else(|| format!("{adapter_id} is not registered"))?;

    if definition.generation == LocalGenerationKind::OllamaHttp {
        return chat_with_ollama(prompt).await;
    }

    if definition.generation == LocalGenerationKind::SpritePython {
        return Ok("I can help ideate motion and generate deterministic sprite-python plans locally. Ask for a specific asset, mascot, logo, UI state, icon, or animation when you want me to create a validated Strut scene.".to_string());
    }

    if definition.generation == LocalGenerationKind::AcpOnly {
        return Err(format!(
            "{} uses an ACP-style runtime. Strut detects it, but chat is disabled until ACP transport support lands.",
            definition.name
        ));
    }

    let command = resolve_adapter_command(&definition).ok_or_else(|| {
        format!(
            "{} was not found on PATH or common tool directories",
            definition.commands.join(" / ")
        )
    })?;
    
    let reference_files = write_reference_files(references)?;
    let reference_dir = reference_files
        .as_ref()
        .map(|files| files.directory.as_path());
    let local_prompt = local_character_prompt(prompt, references, reference_files.as_ref());
    let output = run_local_cli_command(
        &definition,
        &command,
        reference_dir,
        &local_prompt,
        Duration::from_secs(240),
    )?;
    let _ = reference_files
        .as_ref()
        .map(|files| fs::remove_dir_all(&files.directory));
        
    if !output.ok {
        return Err(command_output_preview(&output.stdout, &output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(format!("{stdout}\\n{stderr}").trim().to_string())
}'''
content = re.sub(r'async fn chat_with_local_adapter\(adapter_id: &str, prompt: &str\) -> Result<String, String> \{.*?\s*Ok\(message\.trim\(\)\.to_string\(\)\)\s*\}', chat_local, content, flags=re.DOTALL)

# 9. Modify builder to register assistant_message instead of generate_character and chat_with_provider
content = content.replace('generate_character,', 'assistant_message,')
content = content.replace('chat_with_provider,', '')

with open('apps/studio/src-tauri/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Done python script")
