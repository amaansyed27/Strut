# Strut Animation Bugs - High-Level Technical Design

## Overview

This design addresses two critical bugs in the Strut animation system through targeted architectural improvements:

1. **Chat JSON Dump Bug**: The request routing and error recovery logic incorrectly returns raw JSON documents to the chat interface when conversational responses are expected.

2. **Animation Quality Bug**: The local CLI adapter pipeline fails to propagate premium animation instructions through the system prompt chain, resulting in low-quality generation output.

The fixes preserve all existing functionality while strengthening the intent classification boundaries and the system prompt propagation pipeline.

## Bug Details

### Bug 1: Chat JSON Dump

**Observable Symptom**: When users send conversational messages (e.g., "hi"), the chat interface displays raw Strut document JSON instead of natural language responses.

**Bug Condition**: 
- Request is classified as `RequestIntent::Conversation` OR 
- Context `response_mode` is "chat"
- BUT the LLM output contains malformed JSON or the parsing fallback returns raw text

**Trigger Inputs**:
- Conversational greetings: "hi", "hello", "hey"
- Questions about workspace: "how does X work"
- Brainstorming queries
- Any input with explicit `context.response_mode == "chat"`

**Current Defective Behavior**:
1. User sends "hi"
2. `classify_request_intent()` returns `RequestIntent::Conversation`
3. System calls LLM with `chat_system_prompt`
4. LLM responds with malformed JSON instead of conversational text
5. `parse_assistant_result_from_text()` fails parsing
6. Fallback logic (commands.rs lines 437-443) returns `AssistantResult::Chat { message: raw_json_text, source: "raw" }`
7. Frontend displays raw JSON in chat interface

### Bug 2: Animation Quality

**Observable Symptom**: Local CLI adapters (especially Gemini CLI) produce low-quality, simplistic animations lacking premium visual effects.

**Bug Condition**:
- Generation request uses local CLI adapter (Gemini CLI, Codex, etc.)
- BUT `GENERATION_PLAN_SYSTEM_PROMPT` is not propagated through the pipeline

**Trigger Inputs**:
- Any animation generation request using local CLI provider
- Particularly affects Gemini CLI users

**Current Defective Behavior**:
1. User requests animation generation
2. `assistant_message()` constructs `system_prompt` with `GENERATION_PLAN_SYSTEM_PROMPT`
3. Calls `chat_with_local_adapter(adapter_id, prompt, references, system_prompt)`
4. `chat_with_local_adapter()` IGNORES `system_prompt` parameter
5. Calls `contextual_generation_prompt(prompt, None, GenerationStrategy::ProviderPlan)` without system context
6. Calls `run_local_cli_command()` with minimal prompt
7. For Gemini CLI: Hardcoded `--prompt 'Generate exactly the requested JSON from stdin.'`
8. LLM receives only minimal instructions → produces low-quality animations

## Expected Behavior

### Bug 1: Chat Mode Response

**Property**: For ALL inputs where `classify_request_intent(input) == Conversation` OR `context.response_mode == "chat"`, the system SHALL:
- Return `AssistantResult::Chat` with natural language text
- NOT attempt JSON parsing on the LLM response
- NOT expose raw JSON to the user interface

**Correct Flow**:
1. User sends conversational message
2. Detect chat mode via intent classification OR context
3. Call provider with `chat_system_prompt`
4. Receive natural language response from LLM
5. Return `AssistantResult::Chat` immediately (NO JSON parsing)
6. Display conversational text in chat interface

### Bug 2: Premium Animation Quality

**Property**: For ALL generation requests using local CLI adapters, the system SHALL:
- Include complete `GENERATION_PLAN_SYSTEM_PROMPT` in the prompt sent to LLM
- Propagate system instructions through `chat_with_local_adapter` → `run_local_cli_command`
- NOT use hardcoded minimal prompts

**Correct Flow**:
1. User requests animation generation
2. Construct `system_prompt = ASSISTANT_ROUTER + GENERATION_PLAN_SYSTEM_PROMPT`
3. Pass `system_prompt` to `chat_with_local_adapter`
4. Combine: `full_prompt = system_prompt + "\n\n" + user_prompt`
5. Pass `full_prompt` to `run_local_cli_command` via stdin
6. LLM receives premium instructions → produces high-quality animations with:
   - Premium vector design
   - 2.5D illusion techniques (scale.x/y flips, opacity, parallax)
   - Sprite sheets
   - Curated color palettes
   - Overshoot animations
   - Shadow layers

## Hypothesized Root Cause

### Bug 1 Root Cause

**Primary Issue**: Error recovery logic doesn't respect intent classification boundaries.

**Code Location**: `commands.rs` lines 437-443 (fallback logic in `assistant_message`)

**Mechanism**:
- The fallback path attempts to recover from parsing failures by returning raw text as `AssistantResult::Chat`
- This recovery logic executes even when `RequestIntent::Conversation` was detected
- The system attempts JSON parsing on conversational responses, which should never happen
- When parsing fails (as expected for chat responses), the raw LLM output is returned

**Why This Happens**:
- Chat mode handling is NOT positioned as an early-exit path
- Intent classification result is not enforced throughout the call stack
- Error recovery treats all unparseable responses the same way

### Bug 2 Root Cause

**Primary Issue**: System prompt parameter is passed but never used in local CLI pipeline.

**Code Locations**:
1. `generation.rs` line 116: `chat_with_local_adapter` ignores `system_prompt` parameter
2. `cli.rs` line 227: Gemini CLI uses hardcoded `--prompt` argument

**Mechanism**:
- `assistant_message()` constructs complete system prompt with premium instructions
- Passes to `call_provider_for_assistant()` → `chat_with_local_adapter()`
- `chat_with_local_adapter()` receives `system_prompt` parameter but calls `contextual_generation_prompt()` instead of using it
- `contextual_generation_prompt()` only receives user prompt, no system context
- `run_local_cli_command()` for Gemini CLI uses hardcoded minimal instructions
- Premium animation instructions never reach the LLM

**Why This Happens**:
- Function signature includes `system_prompt` parameter (suggests it should be used)
- Implementation doesn't reference this parameter
- Local CLI path was likely developed separately from BYOK path (which correctly uses system prompts)
- Hardcoded prompts in CLI adapter suggest early development without full prompt pipeline

## Fix Implementation

### Bug 1: Strengthen Chat Mode Boundary

**Strategy**: Move chat mode detection to the TOP of `assistant_message()` as an early-exit path.

**Changes Required**:

1. **commands.rs::assistant_message()** (Priority 1)
   - Restructure function to check chat mode FIRST
   - Add early-exit: if chat mode detected, call provider and return `AssistantResult::Chat` immediately
   - Remove JSON parsing from chat mode path
   - Move error recovery fallback AFTER chat mode check

```rust
// Pseudocode
async fn assistant_message(...) -> Result<AssistantResult> {
    // STEP 1: Check for chat mode FIRST
    let is_chat_mode = context_requests_chat_response(&context) 
                       || classify_request_intent(&prompt) == RequestIntent::Conversation;
    
    if is_chat_mode {
        // EARLY EXIT: Chat mode path
        let chat_prompt = build_chat_system_prompt();
        let response = call_provider_for_assistant(&prompt, provider, refs, &chat_prompt).await?;
        return Ok(AssistantResult::Chat { 
            message: response, 
            source: "chat" 
        });
    }
    
    // STEP 2: Generation mode (existing logic)
    let system_prompt = build_generation_system_prompt();
    let response = call_provider_for_assistant(&prompt, provider, refs, &system_prompt).await?;
    
    // STEP 3: Parse as Strut document
    match parse_assistant_result_from_text(&response) {
        Ok(result) => Ok(result),
        Err(_) => {
            // Apply repair/compact fallback ONLY for generation mode
            apply_fallback_generation(...)
        }
    }
}
```

2. **Validation**:
   - Ensure `classify_request_intent()` correctly identifies conversational inputs
   - Ensure `context_requests_chat_response()` correctly reads response_mode
   - Remove all JSON parsing attempts from chat mode path

### Bug 2: Propagate System Prompt Through Local CLI Pipeline

**Strategy**: Use the `system_prompt` parameter throughout the local CLI adapter chain.

**Changes Required**:

1. **generation.rs::chat_with_local_adapter()** (Priority 1)
   - Remove call to `contextual_generation_prompt()`
   - Use `system_prompt` parameter directly
   - Combine system prompt with user prompt

```rust
// Current (WRONG)
pub async fn chat_with_local_adapter(
    adapter_id: &str,
    prompt: &str,
    references: &[ReferenceImageInput],
    system_prompt: &str, // IGNORED
) -> Result<String, String> {
    let local_prompt = contextual_generation_prompt(
        prompt,
        None,
        GenerationStrategy::ProviderPlan,
    );
    // ...
}

// Fixed (CORRECT)
pub async fn chat_with_local_adapter(
    adapter_id: &str,
    prompt: &str,
    references: &[ReferenceImageInput],
    system_prompt: &str,
) -> Result<String, String> {
    // Combine system instructions with user prompt
    let combined_prompt = format!("{}\n\n{}", system_prompt, prompt);
    
    run_local_cli_command(
        &definition,
        &command,
        Some(&reference_dir),
        &combined_prompt, // Pass full prompt
        timeout,
    ).await
}
```

2. **cli.rs::local_generation_args()** (Priority 2)
   - Remove hardcoded `--prompt` argument for Gemini CLI
   - Let full prompt be passed via stdin

```rust
// Current (WRONG)
"gemini-cli" => vec![
    "--output-format".to_string(),
    "stream-json".to_string(),
    "--prompt".to_string(),
    "Generate exactly the requested JSON from stdin.".to_string(),
]

// Fixed (CORRECT)
"gemini-cli" => vec![
    "--output-format".to_string(),
    "stream-json".to_string(),
    // NO --prompt arg, use stdin for full prompt
]
```

3. **cli.rs::run_local_cli_command()** (Priority 3)
   - Ensure `prompt` parameter (now containing system_prompt + user_prompt) is passed to stdin
   - Verify all CLI adapters receive full combined prompt

**Validation**:
- Verify `GENERATION_PLAN_SYSTEM_PROMPT` appears in final prompt sent to CLI
- Verify no hardcoded prompts override system instructions
- Test with Gemini CLI, Codex, and other local adapters

### Preservation Requirements

**Document Generation Path**:
- MUST preserve: `RequestIntent::Generate` → document parsing → preview rendering
- MUST preserve: Repair and compact plan fallback logic
- MUST preserve: `AssistantResult::DocumentCreated` / `DocumentUpdated` returns

**Provider Routing**:
- MUST preserve: BYOK path system prompt handling (already correct)
- MUST preserve: Ollama HTTP adapter prompt construction
- MUST preserve: sprite-python generation pipeline

**Chat Detection**:
- MUST preserve: Brainstorming and "how does X work" queries use chat mode
- MUST preserve: Explicit `response_mode: "chat"` forces chat mode

## Glossary

- **Bug_Condition_ChatDump (C1)**: Request is classified as Conversation OR context response_mode is "chat", but the LLM output contains malformed JSON or the parsing fallback returns raw text instead of routing to chat mode
- **Bug_Condition_Quality (C2)**: Generation request uses local CLI adapter (especially Gemini CLI), but the `GENERATION_PLAN_SYSTEM_PROMPT` is not propagated through `chat_with_local_adapter` → `run_local_cli_command`
- **Property_ChatDump (P1)**: For all conversational inputs (C1), the system SHALL return natural language responses formatted as `AssistantResult::Chat` without JSON exposure
- **Property_Quality (P2)**: For all generation requests (C2), the system SHALL include the complete `GENERATION_PLAN_SYSTEM_PROMPT` containing premium animation instructions in the LLM input
- **Preservation_DocumentGeneration**: All explicit animation generation requests continue to parse and render Strut documents correctly
- **Preservation_ChatMode**: Existing brainstorming and "how does X work" questions continue using chat mode
- **Preservation_ProviderRouting**: BYOK, Ollama, and sprite-python generation paths remain unchanged
- **AssistantResult**: Rust enum with variants `Chat`, `DocumentCreated`, `DocumentUpdated`
- **RequestIntent**: Classification enum with variants `Generate`, `Conversation`
- **GenerationProvider**: Configuration object routing to local CLI, BYOK API, or Ollama HTTP adapter
- **system_prompt**: String parameter containing `GENERATION_PLAN_SYSTEM_PROMPT` with premium animation instructions (sprite sheets, 2.5D illusion techniques, curated palettes, overshoot animations)

## System Architecture

### Current Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (TypeScript)                    │
│  generationService.assistantMessage() → Tauri IPC               │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                    Backend Commands Layer (Rust)                 │
│  commands.rs::assistant_message()                               │
│    ├─ Intent Classification                                     │
│    ├─ System Prompt Construction                                │
│    └─ Provider Router                                           │
└─────┬────────────────────────────────────────────────┬──────────┘
      │                                                │
      │ Local CLI                                      │ BYOK/Ollama
      │                                                │
┌─────▼────────────────────────┐              ┌───────▼──────────┐
│  generation.rs                │              │  byok.rs         │
│  chat_with_local_adapter()   │              │  byok_generate() │
│    ├─ Reference File Prep    │              │  ollama_http()   │
│    ├─ Prompt Construction    │              └──────────────────┘
│    └─ CLI Invocation         │
└─────┬────────────────────────┘
      │
┌─────▼────────────────────────┐
│  cli.rs                       │
│  run_local_cli_command()     │
│    ├─ Command Executor        │
│    └─ Stream Parser           │
└───────────────────────────────┘
```

### Bug 1 Root Cause: Chat JSON Dump

**Problem Flow:**
```
User sends "hi"
  → classify_request_intent() returns RequestIntent::Conversation
  → call_provider_for_assistant() with chat_system_prompt
  → LLM responds with malformed JSON (instead of conversational text)
  → parse_assistant_result_from_text() fails parsing
  → Fallback logic in assistant_message() lines 437-443
  → Returns AssistantResult::Chat { message: raw_json_text, source: "raw" }
  → Frontend displays raw JSON in chat interface
```

**Issue:** The error recovery logic doesn't respect the intent classification boundary. When `RequestIntent::Conversation` is detected, the system should NEVER attempt JSON parsing or return JSON to the user.

### Bug 2 Root Cause: Animation Quality

**Problem Flow:**
```
User requests animation generation
  → assistant_message() constructs system_prompt with GENERATION_PLAN_SYSTEM_PROMPT
  → Calls chat_with_local_adapter(adapter_id, prompt, references, system_prompt)
  → chat_with_local_adapter() IGNORES system_prompt parameter
  → Calls contextual_generation_prompt(prompt, None, GenerationStrategy::ProviderPlan)
  → Calls run_local_cli_command() with minimal prompt
  → For Gemini CLI: Hardcoded "--prompt 'Generate exactly the requested JSON from stdin.'"
  → LLM receives only minimal instructions
  → Produces low-quality animations without premium techniques
```

**Issue:** The `system_prompt` parameter containing `GENERATION_PLAN_SYSTEM_PROMPT` is passed to `chat_with_local_adapter()` but never used. The local CLI pipeline constructs prompts independently without incorporating this critical context.

## Component Interaction Flows

### Bug 1 Fix: Strengthen Chat Mode Boundary

**New Flow:**
```
┌──────────────────────────────────────────────────────────────┐
│ assistant_message(prompt, provider, refs, context)           │
│                                                               │
│ 1. Check context_requests_chat_response(context)             │
│    OR classify_request_intent(prompt) == Conversation        │
│                                                               │
│ 2. IF chat mode detected:                                    │
│    ├─ Build chat_system_prompt                               │
│    ├─ Call provider with chat prompt                         │
│    └─ RETURN AssistantResult::Chat immediately               │
│       (NO JSON parsing, NO fallback logic)                   │
│                                                               │
│ 3. ELSE (generation mode):                                   │
│    ├─ Build system_prompt with GENERATION_PLAN_SYSTEM_PROMPT│
│    ├─ Call provider with generation prompt                   │
│    ├─ Parse response as Strut document                       │
│    └─ Apply repair/compact fallback ONLY if parsing fails   │
└──────────────────────────────────────────────────────────────┘
```

**Key Architectural Change:**
- Move chat mode handling to the TOP of `assistant_message()` as early-exit path
- Remove JSON parsing from error recovery when in chat mode
- Ensure `classify_request_intent()` result is respected throughout the call stack

### Bug 2 Fix: System Prompt Propagation Pipeline

**New Flow:**
```
┌──────────────────────────────────────────────────────────────┐
│ assistant_message()                                           │
│   system_prompt = ASSISTANT_ROUTER + GENERATION_PLAN_SYSTEM  │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│ call_provider_for_assistant(prompt, provider, refs, sys)    │
│   Route to: byok_generate_text()  OR                         │
│             chat_with_local_adapter()                        │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│ chat_with_local_adapter(id, prompt, refs, system_prompt)    │
│   NEW: Use system_prompt parameter instead of calling       │
│        contextual_generation_prompt()                        │
│                                                               │
│   Combined prompt = system_prompt + "\n\n" + user_prompt    │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│ run_local_cli_command(definition, command, refs, full_prompt)│
│   NEW: For Gemini CLI, pass full_prompt to stdin            │
│        Remove hardcoded "--prompt" arg                       │
│                                                               │
│   Input: system instructions + user request → LLM           │
└──────────────────────────────────────────────────────────────┘
```

**Key Architectural Changes:**
- **generation.rs::chat_with_local_adapter()**: Accept and USE the `system_prompt` parameter
- **cli.rs::local_generation_args()**: Remove hardcoded `--prompt` for Gemini CLI
- **cli.rs::run_local_cli_command()**: Pass full combined prompt through stdin for all CLI adapters

## Data Flow

### Bug 1: Request Intent Routing

```
┌─────────────────┐
│ User Input      │
│ "hi"            │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────┐
│ classify_request_intent()       │
│ Returns: Conversation           │
└────────┬────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│ Build chat_system_prompt        │
│ "You are a helpful assistant..." │
└────────┬────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│ call_provider_for_assistant()   │
│ With chat prompt                │
└────────┬────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│ LLM Response                    │
│ "Hello! How can I help?"        │
└────────┬────────────────────────┘
         │
         ▼
┌─────────────────────────────────┐
│ DIRECT RETURN                   │
│ AssistantResult::Chat           │
│ NO JSON PARSING                 │
└─────────────────────────────────┘
```

### Bug 2: System Prompt Propagation

```
┌──────────────────────────────────────────────┐
│ GENERATION_PLAN_SYSTEM_PROMPT               │
│ - Premium vector design                      │
│ - 2.5D illusion (scale.x/y flips, opacity)  │
│ - Sprite sheets                              │
│ - Curated color palettes                     │
│ - Overshoot animations                       │
│ - Shadow layers, parallax                    │
└────────────────┬─────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────┐
│ assistant_message()                          │
│ system_prompt = ROUTER + GENERATION_PLAN     │
└────────────────┬─────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────┐
│ call_provider_for_assistant()                │
│ Passes system_prompt unchanged               │
└────────────────┬─────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────┐
│ chat_with_local_adapter()                    │
│ NEW: combined = system_prompt + user_prompt  │
│ (Previously: only user_prompt passed)        │
└────────────────┬─────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────┐
│ run_local_cli_command()                      │
│ NEW: Full prompt → stdin                     │
│ (Previously: hardcoded minimal prompt)       │
└────────────────┬─────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────┐
│ Gemini CLI Process                           │
│ Receives: Complete premium instructions      │
│ + User request                               │
└────────────────┬─────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────┐
│ High-Quality Animation Output                │
│ - Vector graphics with sprite sheets         │
│ - 2.5D effects, overshoot animations         │
│ - Professional color palettes                │
└──────────────────────────────────────────────┘
```

## Integration Points

### Frontend → Backend (TypeScript → Rust)

**Interface:** `generationService.assistantMessage()`
```typescript
// apps/studio/src/features/chat/generationService.ts
await tauriInvoke<AssistantResult>("assistant_message", {
  prompt: string,
  provider: GenerationProvider,
  references: ReferenceAttachment[],
  context: GenerationContext,
})
```

**Impact:** NO CHANGES REQUIRED
- Frontend continues using existing interface
- Backend changes are transparent to TypeScript layer

### Backend Command → Provider Adapters

**Current:**
```rust
// commands.rs
async fn call_provider_for_assistant(
    prompt: &str,
    provider: &GenerationProvider,
    references: &[ReferenceImageInput],
    system_prompt: &str,
) -> Result<String, String>
```

**Change:**
- BYOK path: Already uses `system_prompt` correctly (NO CHANGE)
- Local CLI path: Currently ignores `system_prompt` (REQUIRES FIX)

### Local CLI Adapter Chain

**Current:**
```rust
// generation.rs
chat_with_local_adapter(adapter_id, prompt, references, system_prompt)
  ↓
contextual_generation_prompt(prompt, None, GenerationStrategy::ProviderPlan)
  // system_prompt parameter IGNORED
  ↓
run_local_cli_command(definition, command, reference_dir, local_prompt, timeout)
```

**Fixed:**
```rust
// generation.rs
chat_with_local_adapter(adapter_id, prompt, references, system_prompt)
  ↓
combined_prompt = format!("{}\n\n{}", system_prompt, prompt)
  // system_prompt parameter USED
  ↓
run_local_cli_command(definition, command, reference_dir, combined_prompt, timeout)
  ↓
For Gemini CLI: Pass combined_prompt via stdin (no hardcoded --prompt arg)
```

### CLI Command Construction

**Current (cli.rs:227):**
```rust
"gemini-cli" => vec![
    "--output-format".to_string(),
    "stream-json".to_string(),
    "--prompt".to_string(),
    "Generate exactly the requested JSON from stdin.".to_string(),
]
```

**Fixed:**
```rust
"gemini-cli" => vec![
    "--output-format".to_string(),
    "stream-json".to_string(),
    // NO --prompt arg, use stdin for full prompt
]
```

## Correctness Properties

Property 1: Bug Condition - Chat JSON Dump

_For any_ input where the request intent is classified as `Conversation` OR the context response_mode is set to "chat", the fixed assistant_message function SHALL return `AssistantResult::Chat` with natural language text and SHALL NOT attempt to parse JSON from the LLM response or return raw JSON text to the user.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4**

Property 2: Bug Condition - Animation Quality

_For any_ generation request using a local CLI adapter (Gemini CLI, Codex, etc.), the fixed chat_with_local_adapter function SHALL include the complete `GENERATION_PLAN_SYSTEM_PROMPT` in the prompt passed to the LLM, ensuring premium animation instructions are received.

**Validates: Requirements 2.5, 2.6, 2.7, 2.8**

Property 3: Preservation - Document Generation

_For any_ input where the request intent is `Generate` and the LLM produces valid Strut document JSON, the fixed code SHALL parse and return `AssistantResult::DocumentCreated` or `AssistantResult::DocumentUpdated` exactly as the original code did, preserving document generation functionality.

**Validates: Requirements 3.1, 3.2, 3.3**

Property 4: Preservation - Chat Mode Detection

_For any_ input where the user is brainstorming or asking "how does X work" questions, the fixed code SHALL continue using `chat_system_prompt` and provide conversational responses without animation generation, preserving existing chat behavior.

**Validates: Requirements 3.4, 3.5**

Property 5: Preservation - Provider Routing

_For any_ generation request using BYOK providers, Ollama HTTP, or sprite-python, the fixed code SHALL produce the same results as the original code, preserving system prompt propagation and generation quality for these paths.

**Validates: Requirements 3.6, 3.7, 3.8**

## Testing Strategy

### Validation Approach

The testing strategy uses a two-phase approach:
1. **Exploratory Bug Condition Checking**: Demonstrate bugs on unfixed code
2. **Fix + Preservation Checking**: Verify fixes work and existing behavior preserved

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples on UNFIXED code before implementing fixes.

**Test Plan - Bug 1 (Chat JSON Dump)**:
1. Send conversational message "hi" through assistant_message with local CLI provider
2. Mock LLM to return malformed JSON instead of conversational text
3. Observe that parse_assistant_result_from_text() fails
4. Confirm fallback logic returns raw JSON in `AssistantResult::Chat`
5. Expected: Bug reproduces, raw JSON exposed to frontend

**Test Plan - Bug 2 (Animation Quality)**:
1. Send animation generation request through local CLI (Gemini CLI)
2. Instrument chat_with_local_adapter to log received system_prompt
3. Instrument run_local_cli_command to log actual prompt sent to CLI
4. Observe that system_prompt is not included in CLI invocation
5. Observe hardcoded "--prompt" arg for Gemini CLI
6. Expected: System prompt lost, only minimal instructions reach LLM

### Fix Checking

**Goal**: Verify bugs are fixed for all buggy inputs.

**Bug 1 Fix Verification**:
```
FOR ALL input WHERE (classify_request_intent(input) == Conversation 
                     OR context.response_mode == "chat") DO
  result := assistant_message_fixed(input)
  ASSERT result is AssistantResult::Chat
  ASSERT result.message does NOT contain raw JSON
  ASSERT no JSON parsing was attempted on LLM response
END FOR
```

**Bug 2 Fix Verification**:
```
FOR ALL input WHERE (provider.mode == "local" 
                     AND classify_request_intent(input) == Generate) DO
  combined_prompt := chat_with_local_adapter_fixed(input, system_prompt)
  ASSERT combined_prompt CONTAINS GENERATION_PLAN_SYSTEM_PROMPT
  ASSERT run_local_cli_command receives full system_prompt
  ASSERT Gemini CLI does NOT receive hardcoded "--prompt" arg
END FOR
```

### Preservation Checking

**Goal**: Verify non-buggy inputs produce identical results before and after fix.

**Document Generation Preservation**:
```
FOR ALL input WHERE (classify_request_intent(input) == Generate 
                     AND LLM returns valid JSON) DO
  result_original := assistant_message_original(input)
  result_fixed := assistant_message_fixed(input)
  ASSERT result_original == result_fixed
  ASSERT document parsing still works
  ASSERT preview rendering still works
END FOR
```

**Provider Path Preservation**:
```
FOR ALL input WHERE (provider.mode == "byok" 
                     OR provider.local_adapter_id == "sprite-python"
                     OR uses Ollama HTTP) DO
  result_original := call_provider_for_assistant_original(input)
  result_fixed := call_provider_for_assistant_fixed(input)
  ASSERT result_original == result_fixed
END FOR
```

### Unit Tests

**Bug 1 Tests**:
- Test classify_request_intent with conversational inputs
- Test context_requests_chat_response with various response_mode values
- Test assistant_message early-exit for chat mode
- Test that JSON parsing is skipped in chat mode
- Mock LLM to return JSON when chat response expected, verify no JSON exposure

**Bug 2 Tests**:
- Test chat_with_local_adapter receives system_prompt parameter
- Test combined prompt includes system_prompt + user prompt
- Test local_generation_args does not include hardcoded --prompt for Gemini CLI
- Test run_local_cli_command passes full prompt via stdin

### Integration Tests

**Bug 1 Integration**:
- End-to-end test: User sends "hi" → receives conversational response
- End-to-end test: User sends "how does X work" → receives chat response
- End-to-end test: Context with response_mode="chat" → always returns chat

**Bug 2 Integration**:
- End-to-end test: Generate animation with Gemini CLI → verify high quality
- End-to-end test: Generate animation with local CLI → verify system prompt included
- Compare animation quality metrics before/after fix (sprite sheet presence, keyframe count, color palette richness)

### Property-Based Tests

**Chat Mode Boundary Property**:
- Generate random conversational messages
- Verify none trigger JSON parsing
- Verify all return AssistantResult::Chat

**System Prompt Propagation Property**:
- Generate random animation requests
- Verify all include GENERATION_PLAN_SYSTEM_PROMPT in final prompt
- Verify no hardcoded prompts override system instructions

**Preservation Property**:
- Generate random valid Strut documents from LLM
- Verify parsing works identically before/after fix
- Generate random BYOK requests
- Verify results identical before/after fix
