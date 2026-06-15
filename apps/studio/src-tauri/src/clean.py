import re

with open('generation.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Remove constants
text = re.sub(r'pub const GENERATION_PLAN_SYSTEM_PROMPT.*?\"##;', '', text, flags=re.DOTALL)
text = re.sub(r'pub const CHARACTER_DOCUMENT_SYSTEM_PROMPT.*?\"##;', '', text, flags=re.DOTALL)

# Let's remove functions we moved to prompts.rs and parsing.rs
funcs_to_remove = [
    'prompt_with_reference_context',
    'chat_system_prompt',
    'contextual_generation_prompt',
    'local_character_prompt',
    'extract_json_objects',
    'document_from_generation_plan_text',
    'document_from_generation_plan_value',
    'parse_provider_response_document',
]

for func in funcs_to_remove:
    # Match `pub fn NAME(args) -> Ret { ... }` where it can span lines.
    # Note: Regex matching curly braces nested is hard. We will use a simpler approach.
    pass

val_funcs = [
    'validate_generated_document',
    'normalize_generated_document_value',
    'normalize_generated_ids',
    'fill_generated_defaults',
    'normalize_track_property',
    'normalize_keyframe_value',
    'normalize_easing',
    'normalize_state_list',
    'fill_style_defaults',
    'fill_transform_defaults',
    'normalize_none_string',
    'planned_from_compact_document',
    'apply_generation_style_safety',
    'validate_generation_plan',
    'validate_part_geometry',
    'operations_from_generation_plan',
    'semantic_timeline_needs_repair',
    'semantic_timeline_tracks',
    'numeric_track',
    'semantic_motion_targets',
    'semantic_shadow_target',
    'semantic_reveal_targets',
    'semantic_opacity_track',
    'part_is_reveal_candidate',
    'part_is_primary_motion_candidate',
    'semantic_outcome_key_for_timeline',
    'semantic_part_matches_outcome',
    'semantic_timeline_stopword',
    'semantic_tokens',
    'semantic_variation',
    'part_text',
    'validate_scene_operations',
    'document_from_scene_operations',
    'validate_plan_geometry',
    'plan_style_value',
    'plan_geometry_shape',
    'set_node_property',
    'add_keyframe_to_timeline',
    'keyframe_value',
    'normalized_node_kind',
    'node_kind_from_geometry',
    'normalized_state_set',
    'normalized_state_name',
    'active_state_from_timeline_name',
    'normalize_motion_property',
    'normalize_bind_property',
    'normalized_easing_name',
    'allowed_timeline_property',
    'allowed_edit_property',
    'subject_allows_mascot_anatomy',
    'is_mascot_anatomy_name',
    'semantic_token',
    'semantic_label',
    'push_unique',
    'document_from_compact_plan_text',
    'document_from_compact_plan_value',
    'compact_part_node',
    'compact_timelines_value',
    'number_field',
    'string_field',
    'default_style_value',
    'default_transform_value',
    'looks_like_uuid',
    'count_document_nodes',
    'count_nodes',
    'parse_character_spec',
    'role_is_reveal_like',
    'colors_too_close',
    'contrasting_ink_for',
    'normalize_color_token',
    'hex_luminance'
]

# We will remove them using regex carefully by assuming they end with "\n}\n" followed by "pub " or end of file
funcs = funcs_to_remove + val_funcs
for func in funcs:
    pattern = r'^(?:pub )?(?:async )?fn ' + func + r'\b.*?\n}(?=\n|$|\n(?:pub )?(?:async )?fn|\n(?:pub )?const|\n(?:pub )?struct|\n(?:pub )?enum)'
    text = re.sub(pattern, '', text, flags=re.DOTALL | re.MULTILINE)

with open('generation.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print('Cleaned generation.rs')
