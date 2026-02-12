use crate::database::repositories::{StreamInfo, StreamRepository, TimelinePoint};
use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryChange {
    pub timestamp: String,
    pub from_category: String,
    pub to_category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleChange {
    pub timestamp: String,
    pub from_title: String,
    pub to_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamTimelineData {
    pub stream_info: StreamInfo,
    pub stats: Vec<TimelinePoint>,
    pub category_changes: Vec<CategoryChange>,
    pub title_changes: Vec<TitleChange>,
}

/// チャンネルの配信一覧を取得
#[tauri::command]
pub async fn get_channel_streams(
    channel_id: i64,
    limit: Option<i32>,
    offset: Option<i32>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<StreamInfo>, String> {
    db_manager
        .with_connection(|conn| {
            StreamRepository::get_channel_streams(conn, channel_id, limit, offset)
                .map_err(|e| format!("Failed to get channel streams: {}", e))
        })
        .await
}

/// 日付範囲で配信一覧を取得（全チャンネル・カレンダー用）
#[tauri::command]
pub async fn get_streams_by_date_range(
    date_from: String,
    date_to: String,
    limit: Option<i32>,
    offset: Option<i32>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<StreamInfo>, String> {
    db_manager
        .with_connection(|conn| {
            StreamRepository::get_streams_by_date_range(conn, &date_from, &date_to, limit, offset)
                .map_err(|e| format!("Failed to get streams by date range: {}", e))
        })
        .await
}

/// 比較用：基準配信と時間帯が重なる配信をサジェスト（全チャンネル・カテゴリ・時間帯）
#[tauri::command]
pub async fn get_suggested_streams_for_comparison(
    base_stream_id: i64,
    limit: Option<i32>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<StreamInfo>, String> {
    db_manager
        .with_connection(|conn| {
            StreamRepository::get_suggested_streams_for_comparison(conn, base_stream_id, limit)
                .map_err(|e| format!("Failed to get suggested streams: {}", e))
        })
        .await
}

/// 特定配信のタイムラインデータを取得
#[tauri::command]
pub async fn get_stream_timeline(
    stream_id: i64,
    db_manager: State<'_, DatabaseManager>,
) -> Result<StreamTimelineData, String> {
    db_manager
        .with_connection(|conn| {
            get_stream_timeline_internal(conn, stream_id)
                .map_err(|e| format!("Failed to get stream timeline: {}", e))
        })
        .await
}

fn get_stream_timeline_internal(
    conn: &duckdb::Connection,
    stream_id: i64,
) -> Result<StreamTimelineData, Box<dyn std::error::Error + Send + Sync>> {
    let stream_info = StreamRepository::get_stream_info_by_id(conn, stream_id)?;
    let stats = StreamRepository::get_timeline_stats(conn, stream_id)?;
    let category_changes = detect_category_changes(&stats);
    let title_changes = detect_title_changes(&stats);

    Ok(StreamTimelineData {
        stream_info,
        stats,
        category_changes,
        title_changes,
    })
}

fn detect_category_changes(stats: &[TimelinePoint]) -> Vec<CategoryChange> {
    let mut changes = Vec::new();
    let mut prev_category: Option<String> = None;

    for stat in stats {
        if !stat.category.is_empty() {
            if let Some(ref prev) = prev_category {
                if prev != &stat.category {
                    changes.push(CategoryChange {
                        timestamp: stat.collected_at.clone(),
                        from_category: prev.clone(),
                        to_category: stat.category.clone(),
                    });
                }
            }
            prev_category = Some(stat.category.clone());
        }
    }

    changes
}

fn detect_title_changes(stats: &[TimelinePoint]) -> Vec<TitleChange> {
    let mut changes = Vec::new();
    let mut prev_title: Option<String> = None;

    for stat in stats {
        if !stat.title.is_empty() {
            if let Some(ref prev) = prev_title {
                if prev != &stat.title {
                    changes.push(TitleChange {
                        timestamp: stat.collected_at.clone(),
                        from_title: prev.clone(),
                        to_title: stat.title.clone(),
                    });
                }
            }
            prev_title = Some(stat.title.clone());
        }
    }

    changes
}
