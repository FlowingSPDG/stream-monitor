pub mod models;
pub mod schema;
pub mod utils;
pub mod writer;

use duckdb::Connection;
use std::path::PathBuf;
use tauri::AppHandle;

pub fn get_connection(_app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    // Tauri 2.xでは、app_data_dir()は別の方法で取得する必要がある
    // 一時的な解決策として、現在のディレクトリにデータベースを作成
    let db_dir = std::env::current_dir()
        .or_else(|_| std::path::PathBuf::from(".").canonicalize())
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    std::fs::create_dir_all(&db_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    let db_path = db_dir.join("stream_stats.db");
    
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // 初回接続時のみスキーマを初期化
    schema::init_database(&conn)
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;
    
    Ok(conn)
}

pub fn get_connection_with_path(path: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    schema::init_database(&conn)
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;
    Ok(conn)
}
