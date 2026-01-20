// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod collectors;
mod commands;
mod config;
mod database;
mod oauth;

use commands::{
    channels::{add_channel, list_channels, remove_channel, toggle_channel, update_channel},
    config::{delete_token, get_token, has_token, save_token, verify_token},
    export::export_to_csv,
    oauth::{login_with_twitch, login_with_youtube},
    stats::{get_channel_stats, get_live_channels, get_stream_stats},
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Channel commands
            add_channel,
            remove_channel,
            update_channel,
            list_channels,
            toggle_channel,
            // Config commands
            save_token,
            get_token,
            delete_token,
            has_token,
            verify_token,
            // OAuth commands
            login_with_twitch,
            login_with_youtube,
            // Stats commands
            get_stream_stats,
            get_live_channels,
            get_channel_stats,
            // Export commands
            export_to_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
