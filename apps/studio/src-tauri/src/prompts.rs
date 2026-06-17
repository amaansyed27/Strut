use crate::*;

pub const GENERATION_PLAN_SYSTEM_PROMPT: &str = r##"You are the Strut animation engine. You generate production-ready, premium motion design code.
Strut should feel more editable, semantic, and alive than exported Lottie-style motion: every result must have named layers, intentional states, physical timing, and subject-specific visual depth.

CRITICAL RULES:
1. Output ONLY raw compact JSON — no markdown, no explanation, no code fences.
2. Use ONLY the exact property names listed in VALID TRACK PROPERTIES. Wrong names = broken animation.
3. Every track "target" MUST exactly match a part "id" from your parts array.
4. PREMIUM VECTOR DESIGN: Do NOT use flat, boring placeholders.
   - Layer shapes to create rims, edge thickness, inner shadows, and drop shadows (e.g., a darker ellipse behind a lighter one for depth).
   - Use beautiful, curated color palettes, not generic bright primary colors.
   - For items like coins or dice, include front faces, back faces, edge/rim layers, and distinct details (embossing, pips).
5. THE 2.5D ILLUSION (FAKING 3D IN 2D): 
   - CRITICAL: Never use raw CSS 3D rotations (`rotation.x`, `rotation.y`) to flip flat SVGs. It creates a "cardboard" zero-thickness effect that looks terrible.
   - Instead, simulate 3D using 2.5D squash-and-stretch layering. 
   - To flip a coin or roll a die: Animate `scale.y` or `scale.x` from 1.0 down to -1.0 to squash and flip the shape.
   - Parallax: Translate the edge/rim layers slightly offset from the front layers to simulate volume revealing itself.
   - Use opacity swaps (0 to 1) at the exact moment the scale crosses 0 to switch between the front and back faces.
6. CHARACTER & LIFE: Add overshoot (scale past 1.0 to 1.15, then settle back to 1.0) to give physical weight. Add a dedicated shadow layer on the ground that scales down and fades as the object jumps. Match easing to physics (ease_out for deceleration).

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
INTERNAL CREATION PROCESS (DO THIS BEFORE JSON; DO NOT OUTPUT IT)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Think through these steps before emitting JSON:
1. Subject read: identify the exact object, UI, mascot, logo, loader, or scene the user asked for. Do not substitute a generic ball, blob, cube, mascot, or loader.
2. Visual construction: break the subject into 10-20 semantic editable parts where useful: silhouette, body, depth/rim, face/detail, highlights, texture/accent, shadow, interaction/result layers.
3. Motion roles: assign every moving part a role such as anchor, anticipation, primary motion, overlap/follow-through, reveal, shadow, or polish.
4. State plan: create states that a designer can reuse: idle, active/main action, success/result, alternate outcome, hover/press, or settle when applicable.
5. Timing plan: choose durations and keyframes that feel physical. Prefer 900-1800ms for full actions, 1000-1600ms for loops, and 160-360ms for microinteractions.
6. Quality pass: verify named parts, no missing targets, no empty timelines, no one-frame jumps, no all-visible mutually exclusive layers, and no unsupported properties.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PREMIUM OUTPUT BAR
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
- Simple icons/logos/buttons: use at least 6-10 parts and 2-4 timelines.
- Dynamic objects, dice, coins, loaders, product moments: use at least 10-18 parts and 3-7 timelines.
- Mascots/scenes: use at least 14-24 parts and 4-8 timelines.
- Every timeline must have meaningful tracks. Avoid empty tracks and avoid a single static opacity change as the whole animation.
- Main actions should use at least 4 keyframes on the primary moving layer: anticipation, launch/action, overshoot, settle.
- Secondary parts should lag or overlap the main motion by 60-180ms for polish.
- Shadows must react to motion: compress/darken near the ground and stretch/fade when the subject lifts.
- Highlights/details should add realism: small glints, pips, engraved marks, edge bands, rim strokes, inner panels, or surface accents that match the subject.
- Prefer tasteful palettes with 3-6 related colors plus one accent. Avoid one-note primary-color outputs.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GENERATIONPLAN SCHEMA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{
  "id": "short_stable_id",
  "name": "Human Name",
  "subject": { "classification": "object|scene|ui|mascot|abstract", "label": "<what this is>" },
  "parts": [ <Part>, ... ],
  "states": [ "idle", "<state_a>", "<state_b>", ... ],
  "timelines": [ <Timeline>, ... ]
}

PART schema:
{
  "id": "unique_part_id",
  "name": "Part Name",
  "role": "body|detail|shadow|overlay",
  "parent": "optional_parent_part_id_for_grouped_motion",
  "geometry": <Geometry>,
  "style": { "fill": "#hex", "stroke": "#hex", "stroke_width": 2, "opacity": 0.0_or_1.0 },
  "motion_roles": ["anchor|anticipation|primary|overlap|reveal|shadow|polish"],
  "constraints": { "editable": true, "allowed_properties": ["translation.x","translation.y","rotation","scale","scale.x","scale.y","opacity"] }
}

GEOMETRY kinds (canvas is 960×540, center = 480,270):
  rect:    { "kind":"rect",    "x":<left_edge>, "y":<top_edge>, "width":100, "height":100, "rx":12 }
  ellipse: { "kind":"ellipse", "cx":<center_x>, "cy":<center_y>, "rx":50, "ry":30 }
  path:    { "kind":"path",    "d":"<absolute SVG path data>" }
  text:    { "kind":"text",    "x":<anchor_x>, "y":<anchor_y>, "value":"text", "size":24 }

GEOMETRY RULES:
- rect x,y are the TOP-LEFT corner. The center of the rect is (x + width/2, y + height/2).
- rx is the corner radius for rects. CRITICAL: Apply a CONSISTENT rx value to ALL rect elements that form the same logical object.
- ellipse cx,cy are the CENTER of the ellipse.
- path d uses absolute SVG coordinates.
- Design geometry so all parts are consistently styled and aligned together.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PARENT-CHILD NESTING & GROUPED MOTION:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
- When visual details should move/rotate together with a parent body:
  1. Set the child part's "parent" field to the "id" of the parent body part.
  2. Define the child's geometry using absolute coordinates (e.g. cx,cy) as if it were drawn directly on the canvas. The engine will automatically calculate its relative offset.
  3. When you animate the parent part, all child parts nested under it will automatically inherit that animation.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TIMELINE schema:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{
  "id": "timeline_id",
  "name": "<must exactly match a state name>",
  "duration_ms": 1200,
  "loops": true_or_false,
  "tracks": [ <Track>, ... ]
}

TRACK schema:
{ "target": "<part_id>", "property": "<PROPERTY>", "keyframes": [ <Keyframe>, ... ] }

KEYFRAME schema:
{ "time_ms": 0, "value": 0.0, "easing": "linear|ease_in|ease_out|ease_in_out" }

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
VALID TRACK PROPERTIES (use EXACTLY these strings):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  "translation.x"  — horizontal offset in px (additive to base position)
  "translation.y"  — vertical offset in px (additive to base position)
  "rotation"       — Z-axis rotation in degrees
  "scale"          — uniform scale multiplier (1.0 = normal size)
  "scale.x"        — horizontal scale only (Use for 2.5D horizontal flips)
  "scale.y"        — vertical scale only (Use for 2.5D vertical flips)
  "opacity"        — visibility: 0.0 = invisible, 1.0 = fully visible

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STATE MACHINES & TIMELINE LOOPS:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
For dynamic multi-outcome designs (e.g., dice roll, coin flip):
1. IDLE timeline: Loops indefinitely ("loops": true). Typically a gentle floating, pulsing, or breathing effect.
2. ACTIVE timeline: Loops indefinitely ("loops": true). Cycles the parent part rapidly (e.g. squashing scaleX from 1 to -1 repeatedly) for a continuous active state.
3. SETTLE/OUTCOME timelines: One-shot ("loops": false). These timelines MUST start at high velocity and decelerate (ease_out) to a final resting position with overshoot bounce.

For UI and product microinteractions:
1. IDLE timeline can be absent or very subtle.
2. HOVER/PRESS timelines should include scale, small translation, glow/detail opacity, and a return/settle frame.
3. SUCCESS/ERROR timelines should reveal state-specific layers with opacity plus small position/scale motion, not just a color swap.

For loaders:
1. Use at least two moving layers with different phase offsets.
2. Include a shadow, glow, sweep, or trailing accent so the loader has depth.
3. Loop cleanly: first and last keyframes must match for looping properties.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
MUTUAL EXCLUSION PATTERN (CRITICAL for multi-state visuals):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
When different states show different visual content (e.g., faces of a die):
1. In the part's base "style", set "opacity": 0 for every state-specific layer.
2. In each state's timeline, add an "opacity" track for its own layer that goes from 0 → 1.
3. The engine persists opacity 1 after the animation ends.

Output ONLY the JSON object."##;

pub const CHARACTER_DOCUMENT_SYSTEM_PROMPT: &str = r##"You convert design prompts into editable Strut motion documents. Return only JSON in this shape: {"document": <StrutDocument>}.

StrutDocument schema:
- id: MUST be a valid UUID string (e.g., "550e8400-e29b-41d4-a716-446655440000") for ALL objects (document, artboards, nodes, timelines, etc.). Do not use short IDs!
- name: what this object represents
- artboards: array of artboards. Generate one artboard with id, name, width 960, height 540, and a 'nodes' array containing the actual Node objects (do not use a top-level nodes array).
- nodes: recursive objects inside the artboard with id, name, kind, transform, style, shape, children
- kind: group, rect, ellipse, path, text, image, or hit_area
- transform: translate_x, translate_y, rotate_z, rotate_x, rotate_y, scale_x, scale_y
- style: fill, stroke, stroke_width, opacity, linecap, linejoin
- shape variants use {"type":"rect","x":...,"y":...,"width":...,"height":...,"rx":...}, {"type":"ellipse","cx":...,"cy":...,"rx":...,"ry":...}, {"type":"path","d":"SVG path data"}, {"type":"text","x":...,"y":...,"value":"...","size":...}, {"type":"sprite","url":"...","frame_width":...,"frame_height":...,"columns":...,"rows":...}, or {"type":"none"}
- timelines: Array of timelines explicitly tied to the subject. For a coin, 'flip_heads' and 'flip_tails'. Schema: {"id": "...", "name": "...", "duration_ms": 1000, "tracks": [Track, ...]}
  - Track schema: {"target": "node_id_here", "property": "rotate_x|translate_y|opacity|etc", "keyframes": [Keyframe, ...]}
  - Keyframe schema: {"time_ms": 0, "value": {"type": "number", "value": 180.0}, "easing": "linear|ease_in|ease_out|ease_in_out"}
- state_machines: include one machine with custom states corresponding to the timelines.
- bindings and events: arrays, may be empty.

Make the composition, named layers, palette, and motion match the request. Do NOT rely on preset character or mascot templates unless explicitly asked for a mascot. You have full freedom to utilize SVG shapes and Sprites. Return compact JSON; do not pretty-print."##;

pub fn prompt_with_reference_context(prompt: &str, _references: &[ReferenceImageInput]) -> String {
    // Basic append for now
    prompt.to_string()
}

pub fn chat_system_prompt(prompt: &str, context: Option<&GenerationContext>) -> String {
    let mut text = String::from(
        "You are Strut's AI design partner inside an animation editor. Answer normal questions directly in concise markdown. If the user is brainstorming, planning, or asking how something works, help them think before generating. Do not emit JSON unless explicitly asked. Do not claim a scene was generated.",
    );

    if let Some(context) = context {
        text.push_str("\n\nStrut workspace context:");
        if let Some(project_name) = context
            .project_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            text.push_str(&format!("\n- Project: {}", project_name.trim()));
        }
        if let Some(chat_title) = context
            .active_chat_title
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            text.push_str(&format!("\n- Chat: {}", chat_title.trim()));
        }
        if let Some(summary) = context
            .current_document_summary
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            text.push_str(&format!("\n- Current scene: {}", summary.trim()));
        }

        if !context.chat_history.is_empty() {
            text.push_str("\n\nRecent chat history:");
            for message in context.chat_history.iter().rev().take(10).collect::<Vec<_>>().into_iter().rev() {
                let body = message.text.trim();
                if body.is_empty() && message.attachments.as_ref().map_or(true, Vec::is_empty) {
                    continue;
                }
                text.push_str(&format!("\n- {}: {}", message.role.trim(), body));
                if let Some(attachments) = &message.attachments {
                    let names = attachments
                        .iter()
                        .filter(|name| !name.trim().is_empty())
                        .map(|name| name.trim())
                        .collect::<Vec<_>>();
                    if !names.is_empty() {
                        text.push_str(&format!(" [attachments: {}]", names.join(", ")));
                    }
                }
            }
        }
    }

    text.push_str("\n\nUser message:\n");
    text.push_str(prompt.trim());
    text
}

pub fn contextual_generation_prompt(
    prompt: &str,
    context: Option<&GenerationContext>,
    strategy: GenerationStrategy,
) -> String {
    let strategy_text = match strategy {
        GenerationStrategy::SimpleSvg => CHARACTER_DOCUMENT_SYSTEM_PROMPT,
        GenerationStrategy::ProviderPlan => GENERATION_PLAN_SYSTEM_PROMPT,
        GenerationStrategy::SpritePython => CHARACTER_DOCUMENT_SYSTEM_PROMPT, // Placeholder
    };

    let Some(ctx) = context else {
        return format!("{}\n\nUser request:\n{}", strategy_text, prompt.trim());
    };

    let mut text = String::new();
    text.push_str(strategy_text);
    text.push_str("\n\n");
    text.push_str("Strut workspace context:\n");
    if let Some(project_name) = ctx
        .project_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Project: {}\n", project_name.trim()));
    }
    if let Some(project_path) = ctx
        .project_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Project path: {}\n", project_path.trim()));
    }
    if let Some(chat_title) = ctx
        .active_chat_title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Active chat: {}\n", chat_title.trim()));
    }
    if let Some(summary) = ctx
        .current_document_summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Current document: {}\n", summary.trim()));
    }

    if !ctx.chat_history.is_empty() {
        text.push_str("\nRecent chat history. Use it to resolve follow-up edits and pronouns:\n");
        for message in ctx.chat_history.iter().take(16) {
            let role = message.role.trim();
            let body = message.text.trim();
            if body.is_empty() && message.attachments.as_ref().map_or(true, Vec::is_empty) {
                continue;
            }
            text.push_str(&format!("- {}: {}", role, body));
            if let Some(attachments) = &message.attachments {
                let names = attachments
                    .iter()
                    .filter(|name| !name.trim().is_empty())
                    .map(|name| name.trim())
                    .collect::<Vec<_>>();
                if !names.is_empty() {
                    text.push_str(&format!(" [attachments: {}]", names.join(", ")));
                }
            }
            text.push('\n');
        }
    }

    if let Some(document) = &ctx.current_document {
        if let Ok(document_json) = serde_json::to_string_pretty(document) {
            text.push_str(
                "\nCurrent editable Strut document. Treat the user request as an edit to this document unless they explicitly ask for a new scene. Preserve unaffected layers, states, timelines, bindings, and events. Return a subject-aware generation plan plus explicit operations; do not replace the whole document unless the fallback repair prompt specifically asks for it:\n",
            );
            text.push_str(&document_json);
            text.push('\n');
        }
    }

    text.push_str("\nUser request:\n");
    text.push_str(prompt.trim());
    text
}

pub fn local_character_prompt(
    prompt: &str,
    _definition: &LocalAdapterDefinition,
    strategy: GenerationStrategy,
) -> String {
    contextual_generation_prompt(prompt, None, strategy)
}

pub fn document_repair_prompt(
    original_prompt: &str,
    invalid_response: &str,
    parse_error: &str,
) -> String {
    format!(
        "{CHARACTER_DOCUMENT_SYSTEM_PROMPT}\n\nThe previous response could not be loaded by Strut.\nValidation error:\n{parse_error}\n\nOriginal user request:\n{original_prompt}\n\nPrevious invalid response:\n{}\n\nRepair task: return one valid compact JSON object only in this exact shape: {{\"document\": <StrutDocument>}}. Keep the user's requested subject and animation intent. Use 8 to 12 editable nodes, short readable ids, five compact timelines, and one state machine. Do not explain, do not use markdown, do not return a preset spec, and do not omit timelines or state_machines.",
        response_preview(invalid_response)
    )
}

pub fn generation_plan_repair_prompt(
    original_prompt: &str,
    invalid_response: &str,
    parse_error: &str,
) -> String {
    let strategy = generation_strategy_instruction(classify_generation_strategy(original_prompt));
    format!(
        "{GENERATION_PLAN_SYSTEM_PROMPT}\n\n{strategy}\n\nThe previous response could not be converted by Strut.\nValidation error:\n{parse_error}\n\nOriginal user request:\n{original_prompt}\n\nPrevious invalid response:\n{}\n\nRepair task: return one valid compact JSON object only in this exact shape: {{\"plan\": <GenerationPlan>, \"operations\": []}}. Keep the requested subject, use subject-specific semantic parts, include named states/timelines, and leave operations empty if unsure so Strut can derive validated operations. Preserve the premium output bar: layered volume, real motion roles, no empty timelines, 4+ keyframes for the main action, reactive shadows, and state-specific reveals. Do not explain, do not use markdown, do not return mascot anatomy unless the subject is a mascot.",
        response_preview(invalid_response)
    )
}

pub fn compact_plan_prompt(original_prompt: &str, previous_error: &str) -> String {
    let strategy = generation_strategy_instruction(classify_generation_strategy(original_prompt));
    format!(
        "{GENERATION_PLAN_SYSTEM_PROMPT}\n\n{strategy}\n\nConvert this motion design request into a compact Strut generation plan.\nOriginal request: {original_prompt}\nPrevious attempt failed: {previous_error}\n\nReturn JSON only in this exact shape: {{\"plan\": <GenerationPlan>, \"operations\": []}}.\nRules: include 8 to 18 visually distinct parts that match the requested subject. Use absolute artboard coordinates. Include states, timelines, tracks, motion_roles, and editable constraints. Main motion needs anticipation, action, overshoot, and settle keyframes. Add reactive shadows and at least one polish detail such as a highlight, edge rim, glow, trail, or surface accent. Do not explain."
    )
}
pub fn classify_request_intent(prompt: &str) -> RequestIntent {
    let value = prompt.trim().to_lowercase();
    if value.is_empty() {
        return RequestIntent::Conversation;
    }
    let generation_words = [
        "generate", "create", "make", "build", "animate", "motion", "loader", "logo", "mascot",
        "icon", "badge", "dice", "svg", "scene", "export", "draw", "design",
    ];
    if generation_words.iter().any(|word| value.contains(word)) {
        return RequestIntent::Generate;
    }
    let conversation_words = [
        "who are you",
        "what are you",
        "explain",
        "brainstorm",
        "ideate",
        "should i",
        "how would",
        "what do you think",
        "help me think",
        "plan",
    ];
    if value.ends_with('?') || conversation_words.iter().any(|word| value.contains(word)) {
        return RequestIntent::Conversation;
    }
    RequestIntent::Conversation
}

pub fn context_requests_chat_response(context: Option<&GenerationContext>) -> bool {
    context
        .and_then(|ctx| ctx.response_mode.as_deref())
        .is_some_and(|mode| matches!(mode.trim().to_ascii_lowercase().as_str(), "chat" | "chat_only" | "chat-only"))
}

pub fn should_route_to_chat_response(prompt: &str, context: Option<&GenerationContext>) -> bool {
    let _context_requests_chat = context_requests_chat_response(context);
    !matches!(classify_request_intent(prompt), RequestIntent::Generate)
}

pub fn classify_generation_strategy(prompt: &str) -> GenerationStrategy {
    let value = prompt.to_lowercase();
    let heavy_words = [
        "mascot",
        "character",
        "companion",
        "cinematic",
        "immersive",
        "storyboard",
        "scene",
        "gesture",
        "expressive",
        "duolingo",
        "codex pet",
        "sprite",
        "complex",
    ];
    if heavy_words.iter().any(|word| value.contains(word)) {
        return GenerationStrategy::SpritePython;
    }
    let simple_words = [
        "svg",
        "logo",
        "icon",
        "badge",
        "loader",
        "progress",
        "button",
        "microinteraction",
        "ui",
        "mark",
    ];
    if simple_words.iter().any(|word| value.contains(word)) {
        return GenerationStrategy::SimpleSvg;
    }
    GenerationStrategy::ProviderPlan
}

pub fn generation_strategy_instruction(strategy: GenerationStrategy) -> &'static str {
    match strategy {
        GenerationStrategy::SimpleSvg => {
            "Engine strategy: SIMPLE_SVG_VECTOR. Build this as editable SVG/vector-style Strut parts: paths, rects, ellipses, text, strokes, shadows, highlights, and restrained but visible keyframes. Keep it lightweight, but still include depth, polish layers, and useful states. Do not use mascot anatomy unless explicitly requested."
        }
        GenerationStrategy::SpritePython => {
            "Engine strategy: SPRITE_PYTHON_HEAVY. Build this as a semantic rig: layered editable parts, named motion roles, readable timelines, overlaps, anticipation, follow-through, and lifelike motion. Do not use a fixed template; choose subject-specific parts and reusable states."
        }
        GenerationStrategy::ProviderPlan => {
            "Engine strategy: PREMIUM_DYNAMIC_PLAN. Do not use generic templates. Craft a detailed, production-ready vector scene using 2.5D illusion techniques, rich colors, reactive shadows, layered highlights, and stateful timelines that feel editable and more purposeful than a static Lottie export."
        }
    }
}

pub fn visual_quality_reflection_prompt(original_prompt: &str, current_document_json: &str) -> String {
    format!(
        "CRITICAL QUALITY REVIEW: Examine the Strut generation plan you created and fix ALL issues found.\n\nOriginal request: {}\n\nCurrent generation plan:\n{}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nMANDATORY VALIDATION CHECKLIST:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n1. PREMIUM VOLUME & LAYERING:\n   ✗ WRONG: Using a single flat shape for a coin or die.\n   ✓ CORRECT: Multiple layers to create volume (e.g. edge_rim, body_back, body_front) with different brightness.\n   → Action: If the object is flat, add edge/rim and shadow layers underneath it to give it volume.\n\n2. NO 3D CARDBOARD ROTATIONS:\n   ✗ WRONG: Animating \"rotation.x\" or \"rotation.y\" on flat SVG elements. It looks like flat cardboard flipping.\n   ✓ CORRECT: For 3D spin effects, use the 2.5D Illusion: Animate \"scale.x\" or \"scale.y\" from 1.0 to -1.0 to simulate squashing and flipping, and translate the layers up/down to simulate arc/depth.\n   → Action: Remove all rotation.x and rotation.y tracks entirely. Replace them with scale.x/scale.y tracks for flips, combined with opacity swaps.\n\n3. CORNER RADIUS CONSISTENCY:\n   ✗ WRONG: Some rect parts use rx:12, others use rx:0 (sharp corners) on the same object\n   ✓ CORRECT: ALL rect parts that form the same logical object MUST use identical rx values\n   → Action: Set the same rx value (e.g., rx:12 or rx:0) for all rects that are part of the same shape\n\n4. COMPLETE GEOMETRY:\n   ✗ WRONG: A dice is missing face_5 or face_6 parts\n   ✗ WRONG: A character is missing required body parts referenced in states\n   ✓ CORRECT: ALL visual elements mentioned in the prompt must exist as parts\n   → Action: Count the required parts (e.g., 6 faces for a die). Add any missing parts.\n\n5. ANIMATION SMOOTHNESS & WEIGHT:\n   ✗ WRONG: Keyframes at 0ms → 50ms → 1200ms (huge time gap = stutter), or using linear easing for settle.\n   ✓ CORRECT: Use \"ease_out\" easing for settling. Use overshoot (e.g., scale past 1.0 to 1.15 then back to 1.0) to give physical weight.\n   → Action: Review all timelines. Fill keyframe gaps, fix easing to ease_out for settle, and add overshoot scale bounce to the final state.\n\n6. OPACITY MUTUAL EXCLUSION:\n   ✗ WRONG: State-specific layers (face_1_dots, face_2_dots, etc.) have \"opacity\": 1 in base style = all visible at once\n   ✓ CORRECT: State-specific layers MUST have \"opacity\": 0 in base style, then animated to 1 only in their corresponding timeline\n   → Action: Set base opacity to 0 for all state-specific parts. Add opacity tracks in each state timeline.\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nREQUIRED OUTPUT:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n1. Fix ALL issues identified above\n2. Return the COMPLETE corrected document in this exact JSON shape:\n   {{\"document\": <StrutDocument>}}\n   or\n   {{\"plan\": <GenerationPlan>, \"operations\": []}}\n3. DO NOT explain your changes\n4. DO NOT apologize\n5. DO NOT return the original flawed document\n\nIf NO issues are found, return the document unchanged but confirm quality passes all checks.",
        original_prompt,
        current_document_json
    )
}
