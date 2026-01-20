use crate::database::{get_connection, models::StreamStats, utils};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportQuery {
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[tauri::command]
pub async fn export_to_csv(
    app_handle: AppHandle,
    query: ExportQuery,
    file_path: String,
) -> Result<String, String> {
    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let stats = get_stream_stats_internal(&conn, &query)
        .map_err(|e| format!("Failed to query stats: {}", e))?;

    let stats_len = stats.len();

    // CSV生成
    let mut csv = String::from("id,stream_id,collected_at,viewer_count,chat_rate_1min\n");

    for stat in &stats {
        csv.push_str(&format!(
            "{},{},{},{},{}\n",
            stat.id.unwrap_or(0),
            stat.stream_id,
            stat.collected_at,
            stat.viewer_count.unwrap_or(0),
            stat.chat_rate_1min
        ));
    }

    // ファイルに書き込み
    std::fs::write(&file_path, csv)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(format!("Exported {} records to {}", stats_len, file_path))
}

fn get_stream_stats_internal(
    conn: &Connection,
    query: &ExportQuery,
) -> Result<Vec<StreamStats>, duckdb::Error> {
    let mut sql = String::from(
        "SELECT ss.id, ss.stream_id, ss.collected_at, ss.viewer_count, ss.chat_rate_1min 
         FROM stream_stats ss
         INNER JOIN streams s ON ss.stream_id = s.id
         WHERE 1=1",
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(channel_id) = query.channel_id {
        sql.push_str(" AND s.channel_id = ?");
        params.push(channel_id.to_string());
    }

    if let Some(start_time) = &query.start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start_time.clone());
    }

    if let Some(end_time) = &query.end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end_time.clone());
    }

    sql.push_str(" ORDER BY ss.collected_at ASC");

    let mut stmt = conn.prepare(&sql)?;

    let stats: Result<Vec<StreamStats>, _> = utils::query_map_with_params(
        &mut stmt,
        &params,
        |row| {
                Ok(StreamStats {
                    id: Some(row.get(0)?),
                    stream_id: row.get(1)?,
                    collected_at: row.get(2)?,
                    viewer_count: row.get(3)?,
                    chat_rate_1min: row.get(4)?,
                })
            },
        )?
        .collect();

    stats
}
