pub mod aggregation;
pub mod models;
pub mod schema;
pub mod utils;
pub mod writer;

use duckdb::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tokio::sync::watch;

// データベース接続を共有するための管理構造体
#[derive(Clone)]
pub struct DatabaseManager {
    memory_conn: Arc<Mutex<Option<Connection>>>,  // インメモリDB接続
    file_path: PathBuf,  // 永続化ファイルのパス
    shutdown_tx: Arc<Mutex<Option<watch::Sender<bool>>>>,  // シャットダウンシグナル送信
    sync_handle: Arc<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>>,  // 定期同期タスクハンドル
}

impl DatabaseManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // データベースファイルパスの取得
        let file_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
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

        eprintln!("Initializing in-memory database with periodic sync to: {}", file_path.display());
        
        // インメモリDB接続を作成
        let memory_conn = Self::create_memory_connection()?;
        
        // 既存のファイルからデータをロード
        Self::load_from_file(&memory_conn, &file_path)?;

        let memory_conn_arc = Arc::new(Mutex::new(Some(memory_conn)));
        
        // 定期同期タスクを開始
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let sync_handle = Self::start_periodic_sync(
            memory_conn_arc.clone(),
            file_path.clone(),
            shutdown_rx,
        );

        Ok(DatabaseManager {
            memory_conn: memory_conn_arc,
            file_path,
            shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
            sync_handle: Arc::new(Mutex::new(Some(sync_handle))),
        })
    }

    // インメモリDB接続を取得
    pub fn get_connection(&self) -> Result<duckdb::Connection, Box<dyn std::error::Error>> {
        let mut conn_guard = self.memory_conn.lock()
            .map_err(|e| format!("Failed to lock memory connection: {}", e))?;

        // 接続が既に存在する場合はそれを返す
        if let Some(ref conn) = *conn_guard {
            // 接続が有効か確認（簡単なクエリを実行）
            match conn.execute("SELECT 1", []) {
                Ok(_) => {
                    return Ok((*conn)
                        .try_clone()
                        .map_err(|e| format!("Failed to clone connection: {}", e))?);
                }
                Err(_) => {
                    eprintln!("In-memory database connection is invalid, recreating...");
                    *conn_guard = None;
                }
            }
        }

        // 新しいインメモリ接続を作成
        eprintln!("Creating new in-memory database connection");
        let conn = Self::create_memory_connection()?;
        *conn_guard = Some(
            conn.try_clone()
                .map_err(|e| format!("Failed to clone connection: {}", e))?,
        );

        Ok(conn)
    }

    // インメモリDB接続を作成
    fn create_memory_connection() -> Result<Connection, Box<dyn std::error::Error>> {
        eprintln!("Creating in-memory DuckDB connection...");
        
        let conn = Connection::open(":memory:")
            .map_err(|e| format!("Failed to open in-memory database: {}", e))?;
        
        // DuckDBのメモリ設定
        if let Err(e) = conn.execute("PRAGMA memory_limit='2GB'", []) {
            eprintln!("Warning: Failed to set memory limit: {}", e);
        }
        if let Err(e) = conn.execute("PRAGMA threads=4", []) {
            eprintln!("Warning: Failed to set thread count: {}", e);
        }
        
        eprintln!("In-memory database connection created successfully");
        Ok(conn)
    }

    // 既存のファイルDBからインメモリDBへデータをロード
    fn load_from_file(memory_conn: &Connection, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if !file_path.exists() {
            eprintln!("No existing database file found at: {}", file_path.display());
            eprintln!("Initializing fresh database schema...");
            // スキーマを初期化
            schema::init_database(memory_conn)?;
            return Ok(());
        }

        eprintln!("Loading existing data from: {}", file_path.display());
        
        // ファイルDBをアタッチ（READ_ONLY）
        let attach_sql = format!("ATTACH '{}' AS file_db (READ_ONLY)", file_path.display());
        memory_conn.execute(&attach_sql, [])
            .map_err(|e| format!("Failed to attach file database: {}", e))?;

        // テーブルが存在するか確認してからコピー
        let tables = vec!["channels", "streams", "stream_stats", "chat_messages"];
        for table in &tables {
            let check_sql = format!("SELECT COUNT(*) FROM file_db.{}", table);
            match memory_conn.query_row(&check_sql, [], |row| row.get::<_, i64>(0)) {
                Ok(count) => {
                    eprintln!("Loading {} rows from table: {}", count, table);
                    let copy_sql = format!("INSERT INTO memory.main.{} SELECT * FROM file_db.{}", table, table);
                    memory_conn.execute(&copy_sql, [])
                        .map_err(|e| format!("Failed to copy table {}: {}", table, e))?;
                }
                Err(_) => {
                    eprintln!("Table {} not found in file database, skipping", table);
                }
            }
        }

        // デタッチ
        memory_conn.execute("DETACH file_db", [])
            .map_err(|e| format!("Failed to detach file database: {}", e))?;

        eprintln!("Data loaded successfully from file database");
        Ok(())
    }

    // インメモリDBをファイルに同期
    fn sync_to_file(memory_conn: &Connection, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("Starting database sync to file: {}", file_path.display());
        
        // 一時ファイルに書き出し（アトミックリネームのため）
        let temp_path = file_path.with_extension("db.tmp");
        
        // 一時ファイルが既に存在する場合は削除
        if temp_path.exists() {
            std::fs::remove_file(&temp_path)
                .map_err(|e| format!("Failed to remove existing temp file: {}", e))?;
        }

        // 一時ファイルにアタッチ
        let attach_sql = format!("ATTACH '{}' AS file_db", temp_path.display());
        memory_conn.execute(&attach_sql, [])
            .map_err(|e| format!("Failed to attach temp file: {}", e))?;

        // データベース全体をコピー
        memory_conn.execute("COPY FROM DATABASE memory TO file_db", [])
            .map_err(|e| format!("Failed to copy database: {}", e))?;

        // CHECKPOINTを実行してWALをフラッシュ
        memory_conn.execute("CHECKPOINT file_db", [])
            .map_err(|e| format!("Failed to checkpoint: {}", e))?;

        // デタッチ
        memory_conn.execute("DETACH file_db", [])
            .map_err(|e| format!("Failed to detach file database: {}", e))?;

        // アトミックにリネーム
        std::fs::rename(&temp_path, file_path)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        eprintln!("Database synced successfully to: {}", file_path.display());
        Ok(())
    }

    // 定期同期タスクを開始
    fn start_periodic_sync(
        memory_conn: Arc<Mutex<Option<Connection>>>,
        file_path: PathBuf,
        mut shutdown_rx: watch::Receiver<bool>,
    ) -> tauri::async_runtime::JoinHandle<()> {
        // Tauriの非同期ランタイムを使用してspawn
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // 定期同期を実行
                        if let Ok(conn_guard) = memory_conn.lock() {
                            if let Some(ref conn) = *conn_guard {
                                if let Err(e) = Self::sync_to_file(conn, &file_path) {
                                    eprintln!("Periodic sync error: {}", e);
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        // シャットダウンシグナルを受信
                        eprintln!("Shutdown signal received, performing final sync...");
                        if let Ok(conn_guard) = memory_conn.lock() {
                            if let Some(ref conn) = *conn_guard {
                                if let Err(e) = Self::sync_to_file(conn, &file_path) {
                                    eprintln!("Final sync error: {}", e);
                                } else {
                                    eprintln!("Final sync completed successfully");
                                }
                            }
                        }
                        break;
                    }
                }
            }
            
            eprintln!("Periodic sync task terminated");
        })
    }

    // 明示的なシャットダウン処理
    pub fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("DatabaseManager shutdown initiated...");
        
        // シャットダウンシグナルを送信
        if let Ok(mut tx_guard) = self.shutdown_tx.lock() {
            if let Some(tx) = tx_guard.take() {
                let _ = tx.send(true);
            }
        }

        // 同期タスクの完了を待機
        if let Ok(mut handle_guard) = self.sync_handle.lock() {
            if let Some(handle) = handle_guard.take() {
                // ブロッキング待機（Tauriの非同期ランタイムを使用）
                tauri::async_runtime::block_on(async {
                    let _ = handle.await;
                });
            }
        }

        eprintln!("DatabaseManager shutdown completed");
        Ok(())
    }

    // 実際の接続作成処理（インスタンスメソッド）- 後方互換性のため保持
    #[allow(dead_code)]
    fn create_connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        Self::create_connection_internal(&self.file_path)
    }

    // 静的な接続作成処理（インスタンス不要）- 後方互換性のため保持
    #[allow(dead_code)]
    fn create_connection_internal(db_path: &PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
        // データベースファイルの存在チェック
        let file_exists = db_path.exists();
        eprintln!("Opening DuckDB connection at: {}", db_path.display());
        eprintln!("Database file exists: {}", file_exists);

        // ファイルが存在するが読み取り不可の場合、破損の可能性がある
        if file_exists {
            match std::fs::metadata(&db_path) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        eprintln!("Warning: Database file exists but is empty (0 bytes)");
                    } else {
                        eprintln!("Database file size: {} bytes", metadata.len());
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Cannot read database file metadata: {}", e);
                }
            }
        } else {
            eprintln!("Database file does not exist, will be created");
        }

        // Use a thread with larger stack for DuckDB connection
        let db_path_clone = db_path.clone();
        let conn_result = std::thread::Builder::new()
            .stack_size(512 * 1024 * 1024) // 512MB stack
            .spawn(move || {
                eprintln!("[Thread] Opening DuckDB connection...");
                let result = Connection::open(&db_path_clone);
                eprintln!("[Thread] Connection result obtained");
                result
            })
            .expect("Failed to spawn thread for DuckDB connection")
            .join();

        eprintln!("Connection thread joined");
        let conn = match conn_result {
            Ok(Ok(c)) => {
                eprintln!("DuckDB connection opened successfully");
                c
            }
            Ok(Err(e)) => {
                return Err(format!(
                    "Failed to open database at {}: {}",
                    db_path.display(),
                    e
                )
                .into());
            }
            Err(_) => {
                return Err("Thread panicked while opening database".into());
            }
        };

        // DuckDBのメモリ設定（2GBに設定）
        eprintln!("Setting DuckDB memory limit...");
        if let Err(e) = conn.execute("PRAGMA memory_limit='2GB'", []) {
            eprintln!("Warning: Failed to set memory limit: {}", e);
        }
        eprintln!("Memory limit set");
        if let Err(e) = conn.execute("PRAGMA threads=4", []) {
            eprintln!("Warning: Failed to set thread count: {}", e);
        }
        eprintln!("Thread count set");

        Ok(conn)
    }
}

impl Drop for DatabaseManager {
    fn drop(&mut self) {
        eprintln!("DatabaseManager dropping, initiating shutdown...");
        if let Err(e) = self.shutdown() {
            eprintln!("Error during DatabaseManager drop: {}", e);
        }
    }
}

// 後方互換性のための関数（DatabaseManagerを使用）
pub fn get_connection(app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();
    db_manager.get_connection()
}

#[allow(dead_code)]
pub fn get_connection_with_path(path: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;
    // Note: init_database is now called only once at application startup
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    #[cfg_attr(
        target_os = "windows",
        ignore = "Database tests are unstable on Windows local environment"
    )]
    fn test_database_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection_with_path(db_path.clone()).unwrap();
        // テスト用に明示的にスキーマ初期化を実行
        schema::init_database(&conn).unwrap();

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
    #[cfg_attr(
        target_os = "windows",
        ignore = "Database tests are unstable on Windows local environment"
    )]
    fn test_database_schema_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_schema.db");

        // 2回初期化してもエラーにならないことを確認
        let conn1 = get_connection_with_path(db_path.clone()).unwrap();
        let conn2 = get_connection_with_path(db_path.clone()).unwrap();

        // 明示的にスキーマ初期化を2回実行
        schema::init_database(&conn1).unwrap();
        schema::init_database(&conn2).unwrap();

        assert!(db_path.exists());
    }
}
