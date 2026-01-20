pub mod aggregation;
pub mod models;
pub mod schema;
pub mod utils;
pub mod writer;

use duckdb::Connection;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn get_connection(app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    // Tauri 2.xでの適切なデータディレクトリ取得
    let db_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        app_data_dir.join("stream_stats.db")
    } else {
        // フォールバック：現在のディレクトリを使用
        eprintln!("Warning: Using current directory for database (app_data_dir not available)");
        let db_dir = std::env::current_dir()
            .or_else(|_| std::path::PathBuf::from(".").canonicalize())
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        std::fs::create_dir_all(&db_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        db_dir.join("stream_stats.db")
    };

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {}: {}", db_path.display(), e))?;

    // 初回接続時のみスキーマを初期化
    schema::init_database(&conn)
        .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

    Ok(conn)
}

#[allow(dead_code)]
pub fn get_connection_with_path(path: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;
    schema::init_database(&conn).map_err(|e| format!("Failed to initialize schema: {}", e))?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection_with_path(db_path.clone()).unwrap();

        // データベースが作成されていることを確認
        assert!(db_path.exists());

        // テーブルが作成されていることを確認
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"channels".to_string()));
        assert!(tables.contains(&"streams".to_string()));
        assert!(tables.contains(&"stream_stats".to_string()));
    }

    #[test]
    fn test_database_schema_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_schema.db");

        // 2回初期化してもエラーにならないことを確認
        let _conn1 = get_connection_with_path(db_path.clone()).unwrap();
        let _conn2 = get_connection_with_path(db_path.clone()).unwrap();

        assert!(db_path.exists());
    }
}
