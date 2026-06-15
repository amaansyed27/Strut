use crate::*;

pub const GENERATION_PLAN_SYSTEM_PROMPT: &str = r##"You are the Strut animation engine. Convert user prompts into a GenerationPlan JSON object.

CRITICAL RULES:
1. Output ONLY raw compact JSON — no markdown, no explanation, no code fences.
2. Use ONLY the exact property names listed in VALID TRACK PROPERTIES. Wrong names = broken animation.
3. Every track "target" MUST exactly match a part "id" from your parts array.
4. Design the visuals to actually look like the requested subject. Do NOT use placeholders.
   - If the user asks for a 3D style, construct objects using multiple overlapping faces (e.g., top, side, front) with varied brightness/shading to simulate 3D perspective, rather than a single flat shape.
   - Ensure ALL necessary details (e.g., all 6 faces of a die with their respective dots/pips, or all keys on a keyboard) are fully present as separate parts.
5. Invent states and animation names based on the prompt — never copy example names literally.

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
  "style": { "fill": "#hex", "stroke": "#hex", "stroke_width": 2, "opacity": 0.0_or_1.0 }
}

GEOMETRY kinds (canvas is 960×540, center = 480,270):
  rect:    { "kind":"rect",    "x":<left_edge>, "y":<top_edge>, "width":100, "height":100, "rx":12 }
  ellipse: { "kind":"ellipse", "cx":<center_x>, "cy":<center_y>, "rx":50, "ry":30 }
  path:    { "kind":"path",    "d":"<absolute SVG path data>" }
  text:    { "kind":"text",    "x":<anchor_x>, "y":<anchor_y>, "value":"text", "size":24 }

GEOMETRY RULES:
- rect x,y are the TOP-LEFT corner. The center of the rect is (x + width/2, y + height/2).
- rx is the corner radius for rects. CRITICAL: Apply a CONSISTENT rx value to ALL rect elements that form the same logical object (e.g., if the dice body uses rx:12, ALL face borders and overlays MUST also use rx:12, NOT rx:0 or different values).
- ellipse cx,cy are the CENTER of the ellipse.
- path d uses absolute SVG coordinates.
- Design geometry so all parts are consistently styled and aligned together.
- For 3D layered objects (dice, coins, cards, cubes), ensure overlapping faces have IDENTICAL or carefully proportioned dimensions to avoid visual gaps or clipping during animation.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PARENT-CHILD NESTING & GROUPED MOTION:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
- When visual details (e.g. pips/dots on a die, text/markings on a coin, symbols on a card, hands on a clock) should move/rotate together with a parent body:
  1. Set the child part's "parent" field to the "id" of the parent body part.
  2. Define the child's geometry using absolute coordinates (e.g. cx,cy) as if it were drawn directly on the canvas. The engine will automatically calculate its relative offset.
  3. When you animate the parent part (using translations, rotations, or scaling), all child parts nested under it will automatically inherit that animation, keeping the grouped element perfectly aligned.

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
Note: "value" must be a raw number, NOT a nested object.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
VALID TRACK PROPERTIES (use EXACTLY these strings):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  "translation.x"  — horizontal offset in px (additive to base position)
  "translation.y"  — vertical offset in px (additive to base position)
  "rotation"       — Z-axis rotation in degrees
  "rotation.x"     — X-axis rotation in degrees (3D tilt, top/bottom)
  "rotation.y"     — Y-axis rotation in degrees (3D spin, left/right)
  "scale"          — uniform scale multiplier (1.0 = normal size)
  "scale.x"        — horizontal scale only
  "scale.y"        — vertical scale only
  "opacity"        — visibility: 0.0 = invisible, 1.0 = fully visible

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STATE MACHINES & TIMELINE LOOPS:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
For dynamic multi-outcome designs (e.g., dice roll, coin flip, slot reels, wheel spinner):
1. IDLE timeline: Loops indefinitely ("loops": true). Typically a gentle floating, pulsing, or breathing effect.
2. ACTIVE timeline (e.g., roll, flip, spin): Loops indefinitely ("loops": true). Spins or cycles the parent part rapidly, creating a continuous active state.
3. SETTLE/OUTCOME timelines (e.g., face_1, heads, tails): One-shot ("loops": false). These timelines MUST:
   - Start (at 0ms) with the same high-velocity spin/scale as the active looping timeline to maintain a seamless transition.
   - Decelerate and settle smoothly (using ease_out easing) to their final resting position.
   - Use the Mutual Exclusion Pattern to toggle the opacity of state-specific details.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
MUTUAL EXCLUSION PATTERN (CRITICAL for multi-state visuals):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
When different states show different visual content (e.g., different faces of a die, different menu items,
different card suits, different icons), use opacity to control which content is visible:

1. In the part's base "style", set "opacity": 0 for every state-specific layer.
2. In each state's timeline, add an "opacity" track for its own layer that goes from 0 → 1.
3. Because the animation uses fill-mode "forwards", opacity 1 persists after the animation ends.
4. When the state changes, the old timeline stops and its layer returns to base opacity 0.

This means only the active state's content is ever visible. The shared body/frame animates separately.

Example pattern (generic — adapt to your subject):
  - Part "layer_a" base style: opacity 0
  - Part "layer_b" base style: opacity 0
  - Timeline "state_a": opacity track on "layer_a" → { 0ms: 0, duration_ms: 1 }
  - Timeline "state_b": opacity track on "layer_b" → { 0ms: 0, duration_ms: 1 }
  - The body/frame animates independently in whichever timeline controls it.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ANIMATION QUALITY RULES:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
- CRITICAL 3D ROTATION RULE: For 3D spin effects, use ONLY ONE rotation axis per timeline:
  * "rotation.y" for horizontal spin (coin flip, card flip left/right)
  * "rotation.x" for vertical tumble (dice roll forward/backward)
  * "rotation" for flat 2D spin (no 3D perspective)
  * NEVER mix multiple rotation properties (rotation, rotation.x, rotation.y) in the same timeline - this causes chaotic multi-axis tumbling that looks broken
- Use "scale" with overshoot (e.g., 1 → 1.15 → 0.9 → 1) for physical weight/bounce on landing.
- Use "translation.y" for gravity, bounce, lift, or float.
- Combine multiple properties on the same target for layered, expressive motion.
- Match easing to physics: ease_out for deceleration/settling, ease_in for acceleration, linear for mechanical.
- Each timeline should feel emotionally appropriate to the prompt.

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

Make the composition, named layers, palette, and motion match the request. Do NOT rely on preset character or mascot templates unless explicitly asked for a mascot. You have full freedom to utilize SVG shapes, Sprites, and 3D transforms (rotate_x, rotate_y). Return compact JSON; do not pretty-print."##;

pub fn prompt_with_reference_context(prompt: &str, _references: &[ReferenceImageInput]) -> String {
    // Basic append for now
    prompt.to_string()
}

pub fn chat_system_prompt(_prompt: &str, _context: Option<&GenerationContext>) -> String {
    "You are the Strut generation router. You output standard valid JSON.".to_string()
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
        "{GENERATION_PLAN_SYSTEM_PROMPT}\n\n{strategy}\n\nThe previous response could not be converted by Strut.\nValidation error:\n{parse_error}\n\nOriginal user request:\n{original_prompt}\n\nPrevious invalid response:\n{}\n\nRepair task: return one valid compact JSON object only in this exact shape: {{\"plan\": <GenerationPlan>, \"operations\": []}}. Keep the requested subject, use subject-specific semantic parts, include named states/timelines, and leave operations empty if unsure so Strut can derive validated operations. Do not explain, do not use markdown, do not return mascot anatomy unless the subject is a mascot.",
        response_preview(invalid_response)
    )
}

pub fn compact_plan_prompt(original_prompt: &str, previous_error: &str) -> String {
    let strategy = generation_strategy_instruction(classify_generation_strategy(original_prompt));
    format!(
        "{GENERATION_PLAN_SYSTEM_PROMPT}\n\n{strategy}\n\nConvert this motion design request into a compact Strut generation plan.\nOriginal request: {original_prompt}\nPrevious attempt failed: {previous_error}\n\nReturn JSON only in this exact shape: {{\"plan\": <GenerationPlan>, \"operations\": []}}.\nRules: include 6 to 14 visually distinct parts that match the requested subject. Use absolute artboard coordinates. Include states, timelines, tracks, and editable constraints. The motion must be calm and low-energy: subtle bob, tiny tilt, focused scan, restrained settle, soft reveal, progress sweep, or similar. Do not explain."
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
            "Engine strategy: SIMPLE_SVG_VECTOR. Build this as editable SVG/vector-style Strut parts: paths, rects, ellipses, text, masks, strokes, and restrained keyframes. Keep it lightweight and do not use mascot anatomy unless explicitly requested."
        }
        GenerationStrategy::SpritePython => {
            "Engine strategy: SPRITE_PYTHON_HEAVY. Build this as a sprite-python style semantic rig: more layered editable sprites, named motion roles, readable timelines, and low-energy lifelike motion. Do not use a fixed template; choose subject-specific parts."
        }
        GenerationStrategy::ProviderPlan => {
            "Engine strategy: PROVIDER_DYNAMIC_PLAN. Choose the simplest dynamic representation that fits the prompt, with subject-specific semantic parts and validated operations. Avoid fixed templates."
        }
    }
}

pub fn visual_quality_reflection_prompt(original_prompt: &str, current_document_json: &str) -> String {
    format!(
        "CRITICAL QUALITY REVIEW: Examine the Strut generation plan you created and fix ALL issues found.\n\nOriginal request: {}\n\nCurrent generation plan:\n{}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nMANDATORY VALIDATION CHECKLIST:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n1. CORNER RADIUS CONSISTENCY:\n   ✗ WRONG: Some rect parts use rx:12, others use rx:0 (sharp corners) on the same object\n   ✓ CORRECT: ALL rect parts that form the same logical object MUST use identical rx values\n   → Action: Set the same rx value (e.g., rx:12 or rx:0) for all rects that are part of the same shape\n\n2. SINGLE-AXIS 3D ROTATION:\n   ✗ WRONG: Animating both \"rotation.x\" AND \"rotation.y\" simultaneously = chaotic tumbling\n   ✗ WRONG: Mixing \"rotation\", \"rotation.x\", and \"rotation.y\" on the same timeline\n   ✓ CORRECT: For 3D spin effects, choose ONE dominant axis per timeline:\n     - rotation.y ONLY for horizontal spin (left/right flip, like a coin toss)\n     - rotation.x ONLY for vertical tumble (forward/backward, like a dice roll)\n     - rotation ONLY for 2D flat spin (no 3D effect)\n   → Action: Remove all multi-axis rotation tracks. Keep only the primary rotation axis.\n\n3. COMPLETE GEOMETRY:\n   ✗ WRONG: A dice is missing face_5 or face_6 parts\n   ✗ WRONG: A character is missing required body parts referenced in states\n   ✓ CORRECT: ALL visual elements mentioned in the prompt must exist as parts\n   → Action: Count the required parts (e.g., 6 faces for a die). Add any missing parts.\n\n4. GEOMETRY OVERLAP & DEPTH:\n   ✗ WRONG: Multiple rect parts at the exact same x,y,width,height with different fills = visual glitching\n   ✗ WRONG: Face outlines don't perfectly match the dice body dimensions\n   ✓ CORRECT: Overlapping parts must have identical or carefully offset dimensions to simulate depth\n   → Action: For layered 3D objects, ensure face dimensions match parent body, or use subtle offsets for depth\n\n5. ANIMATION SMOOTHNESS:\n   ✗ WRONG: Keyframes at 0ms → 50ms → 1200ms (huge time gap = stutter)\n   ✗ WRONG: Using \"linear\" easing for settle animations (looks mechanical, not organic)\n   ✓ CORRECT:\n     - For spin: use evenly distributed keyframes (0ms, 300ms, 600ms, 900ms, 1200ms)\n     - For settle: use \"ease_out\" easing, not linear\n     - For bounce: add intermediate scale keyframes (1.0 → 1.15 → 0.95 → 1.0)\n   → Action: Review all timelines for sparse keyframes or wrong easing. Fill gaps and fix easing.\n\n6. OPACITY MUTUAL EXCLUSION:\n   ✗ WRONG: State-specific layers (face_1_dots, face_2_dots, etc.) have \"opacity\": 1 in base style = all visible at once\n   ✓ CORRECT: State-specific layers MUST have \"opacity\": 0 in base style, then animated to 1 only in their corresponding timeline\n   → Action: Set base opacity to 0 for all state-specific parts. Add opacity tracks in each state timeline.\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nREQUIRED OUTPUT:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n1. Fix ALL issues identified above\n2. Return the COMPLETE corrected document in this exact JSON shape:\n   {{\"document\": <StrutDocument>}}\n   or\n   {{\"plan\": <GenerationPlan>, \"operations\": []}}\n3. DO NOT explain your changes\n4. DO NOT apologize\n5. DO NOT return the original flawed document\n\nIf NO issues are found, return the document unchanged but confirm quality passes all checks.",
        original_prompt,
        current_document_json
    )
}
