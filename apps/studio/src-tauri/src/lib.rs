pub mod models;
pub mod project;
pub mod providers;
pub mod generation;
pub mod commands;
pub mod prompts;
pub mod dynamic_prompts;
pub mod motion_prompts;
pub mod parsing;
pub mod validation;
pub mod api;
pub mod cli;
pub mod motion;
pub mod utils;
pub mod provider_clients;
pub mod openrouter_clients;
pub mod layout_normalizer;
pub mod quality_gate;
mod dice_repair;

pub use models::*;
pub use project::*;
pub use providers::*;
pub use generation::*;
pub use commands::*;
pub use prompts::*;
pub use dynamic_prompts::*;
pub use motion_prompts::*;
pub use parsing::*;
pub use validation::*;
pub use api::*;
pub use cli::*;
pub use motion::*;
pub use utils::*;
pub use provider_clients::*;
pub use openrouter_clients::*;
pub use layout_normalizer::*;
pub use quality_gate::*;

use base64::Engine as _;
use std::collections::HashMap;
use std::fs;
use std::path::Component;
use std::time::Duration;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            studio_status,
            default_project_location,
            create_project,
            save_project_snapshot,
            save_project_animation,
            delete_project_animation,
            load_project_snapshot,
            validate_scene_document,
            validate_generation_plan_batch,
            open_project_folder,
            assistant_message,
            assistant_message_v2,
            assistant_message_openrouter_v2,
            local_agent_adapters,
            test_local_adapter,
            test_local_adapter_v2,
            export_animation_to_react,
            motion_spec_route,
            test_byok_provider,
            test_byok_provider_v2,
            save_byok_provider,
            save_byok_provider_v2
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
