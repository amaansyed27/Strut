# Bugfix Requirements Document

## Introduction

This document addresses two critical bugs in the Strut animation application that severely impact user experience:

1. **Chat JSON Dump Bug**: When users engage in normal conversation (e.g., typing "hi"), the chat interface dumps raw Strut document JSON instead of providing conversational responses and rendering animations in the preview panel.

2. **Poor Animation Quality Bug**: The generation service, particularly when using Gemini CLI, produces low-quality, simplistic animations with minimal visual effects instead of the sophisticated, premium animations with sprite sheets, complex CSS/SVG animations, and rich visual effects that Strut is designed to create.

Both bugs prevent Strut from delivering its intended user experience: natural chat interactions combined with high-quality animation generation.

## Bug Analysis

### Current Behavior (Defect)

#### Bug 1: Chat JSON Dump

1.1 WHEN the user sends a conversational message like "hi" THEN the system returns raw JSON (the entire Strut document structure with all animation definitions) as plain text in the chat interface

1.2 WHEN the LLM response fails JSON parsing in `parse_assistant_result_from_text` THEN the system falls back to returning `AssistantResult::Chat` with `source: "raw"` and the full unparsed text as the message

1.3 WHEN the `RequestIntent` is classified as `Conversation` but the LLM output contains a malformed JSON document THEN the error recovery logic in `assistant_message` (lines 437-443 in commands.rs) returns the raw text instead of routing to chat mode

#### Bug 2: Poor Animation Quality

2.1 WHEN the generation service calls a local CLI adapter (especially Gemini CLI) THEN it ignores the `system_prompt` parameter containing `GENERATION_PLAN_SYSTEM_PROMPT` with premium animation instructions

2.2 WHEN `chat_with_local_adapter` constructs the prompt for local adapters THEN it calls `contextual_generation_prompt(prompt, None, GenerationStrategy::ProviderPlan)` which does not include the `system_prompt` parameter passed to the function (line 116 in generation.rs)

2.3 WHEN `run_local_cli_command` is invoked for Gemini CLI THEN it uses a hardcoded minimal prompt "--prompt 'Generate exactly the requested JSON from stdin.'" instead of the rich `GENERATION_PLAN_SYSTEM_PROMPT` containing instructions for premium design, 2.5D illusion techniques, sprite sheets, and complex animations (line 227 in cli.rs)

2.4 WHEN the LLM receives only minimal instructions THEN it produces basic animations with simple shapes, minimal keyframes, and no sprite sheets or advanced visual effects

### Expected Behavior (Correct)

#### Bug 1: Chat JSON Dump - Expected Behavior

2.1 WHEN the user sends a conversational message like "hi" THEN the system SHALL route the request to chat mode and return a natural language response without any JSON output

2.2 WHEN the `classify_request_intent` function identifies a message as `RequestIntent::Conversation` THEN the system SHALL use `chat_system_prompt` and SHALL NOT attempt to parse the response as a Strut document

2.3 WHEN the user asks conversational questions about the workspace or animations THEN the system SHALL provide markdown-formatted conversational responses and SHALL display them in the chat interface without attempting document generation

2.4 WHEN the context specifies `response_mode: "chat"` via `context_requests_chat_response` THEN the system SHALL always return `AssistantResult::Chat` regardless of LLM output format

#### Bug 2: Poor Animation Quality - Expected Behavior

2.5 WHEN the generation service calls any provider (local CLI, BYOK, or Ollama) for animation generation THEN the system SHALL include the complete `GENERATION_PLAN_SYSTEM_PROMPT` with all premium animation instructions

2.6 WHEN `chat_with_local_adapter` is called for generation (not chat) THEN the system SHALL use the `system_prompt` parameter containing `GENERATION_PLAN_SYSTEM_PROMPT` instead of calling `contextual_generation_prompt` with only the user prompt

2.7 WHEN `run_local_cli_command` is invoked for Gemini CLI for generation tasks THEN it SHALL pass the complete `system_prompt` as input to stdin along with the user prompt, not use a hardcoded minimal prompt

2.8 WHEN the LLM receives the complete `GENERATION_PLAN_SYSTEM_PROMPT` THEN it SHALL generate animations with premium vector design, 2.5D illusion techniques (scale.x/scale.y flips, opacity swaps, parallax layers), sprite sheets, curated color palettes, overshoot animations, shadow layers, and complex motion sequences

### Unchanged Behavior (Regression Prevention)

#### Document Generation Flow

3.1 WHEN the user explicitly requests animation generation (e.g., "create a coin flip animation") THEN the system SHALL CONTINUE TO classify the intent as `RequestIntent::Generate`, call the provider with generation prompts, parse the response as a Strut document, and update the preview panel with the rendered animation

3.2 WHEN the LLM successfully generates a valid Strut document JSON THEN the system SHALL CONTINUE TO parse it using `parse_assistant_result_from_text`, return `AssistantResult::DocumentCreated` or `AssistantResult::DocumentUpdated`, and render the animation in the preview panel

3.3 WHEN document generation fails on the first attempt THEN the system SHALL CONTINUE TO use the repair and compact plan fallback logic to retry document generation with improved prompts

#### Chat Mode Detection

3.4 WHEN the user is brainstorming or asking "how does X work" questions THEN the system SHALL CONTINUE TO use `chat_system_prompt` and provide conversational guidance without generating animations

3.5 WHEN the response mode in context is explicitly set to "chat", "chat_only", or "chat-only" THEN the system SHALL CONTINUE TO use `context_requests_chat_response` to force chat mode regardless of intent classification

#### Provider Routing

3.6 WHEN using BYOK providers (Anthropic, OpenAI-compatible, Gemini API) THEN the system SHALL CONTINUE TO pass the `system_prompt` parameter correctly via `byok_generate_text`

3.7 WHEN using Ollama HTTP adapter THEN the system SHALL CONTINUE TO include `GENERATION_PLAN_SYSTEM_PROMPT` in the prompt field of the API request

3.8 WHEN using sprite-python generation THEN the system SHALL CONTINUE TO use the deterministic Python generation pipeline with example-based prompts

## New Feature Requirements - Phase 5

### Export Functionality

4.1 WHEN the user clicks an "Export" button in the Studio UI THEN the system SHALL display an export dialog with format options (React) and output directory selection

4.2 WHEN the user confirms React export THEN the system SHALL generate three files in the output directory:
   - `StrutAnimation.tsx`: React component with inline SVG structure and CSS animations
   - `scene.json`: Complete validated Strut document as pretty-printed JSON
   - `README.md`: Usage instructions and integration guide

4.3 WHEN export completes successfully THEN the system SHALL display a success notification with an "Open folder" button that opens the export directory in the file manager

4.4 WHEN the user exports an animation from the animation list THEN the export SHALL use the animation's name as the default output directory name

4.5 WHEN the user exports the current main scene THEN the export SHALL use the project name as the default output directory name

### Chat Animation Generation

4.6 WHEN the user sends a message like "Make me 3d rolling die" or "Create a bouncing ball" THROUGH THE STUDIO UI THEN the system SHALL classify it as `RequestIntent::Generate` (not Conversation)

4.7 WHEN `classify_request_intent` receives imperative animation requests starting with verbs like "make", "create", "generate", "build" THEN it SHALL return `RequestIntent::Generate`

4.8 WHEN the chat interface is empty THEN it SHALL show placeholder text with generation examples like "Create a bouncing ball animation..."

4.9 WHEN the user interacts with the chat THEN quick action examples SHALL be available above the input for common animation types:
   - "Coin flip" → fills input with "Create a 3D coin flip animation"
   - "Dice roller" → fills input with "Create a rolling dice with all 6 faces"
   - "Loader" → fills input with "Create a smooth loader animation"
   - "Button" → fills input with "Create a button with hover and press states"

4.10 WHEN animation generation succeeds THEN the preview panel SHALL display the rendered animation and the animation SHALL be automatically added to the project's animation list

4.11 WHEN animation generation is tested THEN it MUST be tested through the Studio UI with a properly configured provider (not through external Gemini chat or other tools) because only the Studio UI sends the correct GENERATION_PLAN_SYSTEM_PROMPT

4.12 WHEN the LLM returns conversational text instead of JSON THEN the error message SHALL explain "Provider did not return valid Strut animation JSON" and suggest checking provider configuration

4.13 WHEN STRUT_DEBUG_GENERATION environment variable is set THEN the system SHALL log:
   - Request classification result
   - System prompt (first 300 chars)
   - Provider mode and adapter
   - LLM response (first 1000 chars)
   - Parse attempt results
   - Final AssistantResult type
