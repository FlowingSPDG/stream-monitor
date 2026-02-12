use crate::database::{
    models::StreamStats,
    repositories::{
        chat_message_repository::ChatMessageRepository,
        stream_stats_repository::StreamStatsRepository,
    },
    DatabaseManager,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamStatsQuery {
    pub stream_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[tauri::command]
pub async fn get_stream_stats(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: StreamStatsQuery,
) -> Result<Vec<StreamStats>, String> {
    db_manager
        .with_connection(|conn| {
            StreamStatsRepository::get_stream_stats_filtered(
                conn,
                query.stream_id,
                query.channel_id,
                query.start_time.as_deref(),
                query.end_time.as_deref(),
                false, // ORDER BY collected_at DESC
            )
            .map_err(|e| e.to_string())
        })
        .await
}

#[tauri::command]
pub async fn get_realtime_chat_rate(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
) -> Result<i64, String> {
    db_manager
        .with_connection(|conn| {
            ChatMessageRepository::get_realtime_chat_rate(conn).map_err(|e| e.to_string())
        })
        .await
}
