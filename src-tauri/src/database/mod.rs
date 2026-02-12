pub mod aggregation;
pub mod analytics;
pub mod chat_analytics;
pub mod data_science_analytics;
pub mod models;
pub mod query_helpers;
pub mod repositories;
pub mod schema;
pub mod utils;
pub mod writer;

use crate::error::ResultExt;
use duckdb::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

/// DuckDB の WAL ファイルパス（DB が stream_stats.db のとき stream_stats.db.wal）
fn wal_path(db_path: &Path) -> PathBuf {
    db_path.with_extension("db.wal")
}

/// 起動前のリカバリ処理（一時ファイルのクリーンアップ）
/// WALが存在する場合は先に退避する。Connection::open が WAL ありでクラッシュ/パニックするため、
/// 開く前に WAL を外してから開くことで起動クラッシュを防ぐ。
fn cleanup_stale_files(db_path: &Path) {
    let wal_path = wal_path(db_path);
    let tmp_path = db_path.with_extension("tmp");

    // .wal が残っている＝前回が異常終了。DuckDB の open が WAL ありでクラッシュするため、先に退避してから開く
    if wal_path.exists() {
        eprintln!(
            "[DB Recovery] WAL file found at {}, backing up before open to avoid crash",
            wal_path.display()
        );
        if let Err(e) = recover_from_corrupted_wal(db_path) {
            eprintln!("[DB Recovery] Failed to backup WAL: {}", e);
        }
    }

    // .tmp ファイルは削除（不完全な操作の残骸）
    if tmp_path.exists() {
        eprintln!(
            "[DB Recovery] Removing stale tmp file: {}",
            tmp_path.display()
        );
        if let Err(e) = std::fs::remove_file(&tmp_path) {
            eprintln!("[DB Recovery] Failed to remove tmp file: {}", e);
        }
    }
}

/// WALファイルが破損している場合のリカバリ処理
fn recover_from_corrupted_wal(db_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Local;

    let w_path = wal_path(db_path);

    if w_path.exists() {
        // WALファイルをバックアップとしてリネーム（stream_stats.db.wal → stream_stats.db.wal.backup.<ts>）
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_wal_path = db_path.with_extension(format!("db.wal.backup.{}", timestamp));

        eprintln!(
            "[DB Recovery] Moving corrupted WAL file to backup: {}",
            backup_wal_path.display()
        );

        std::fs::rename(&w_path, &backup_wal_path)?;
        eprintln!("[DB Recovery] WAL file backed up successfully");
    }

    Ok(())
}

/// データベース接続を共有するための管理構造体
#[derive(Clone)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // データベースファイルパスの取得
        let db_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
            std::fs::create_dir_all(&app_data_dir)
                .io_context("create app data directory")
                .map_err(|e| e.to_string())?;
            app_data_dir.join("stream_stats.db")
        } else {
            eprintln!("Warning: Using current directory for database");
            PathBuf::from("stream_stats.db")
        };

        // 起動時のリカバリ処理
        cleanup_stale_files(&db_path);

        // 開発環境と本番環境で統一してファイルベースDBを使用
        eprintln!("Opening DuckDB at: {}", db_path.display());
        // 1プロセス内で2回 open すると、初回失敗時に DuckDB がファイルを握ったまま Err を返すため
        // 2回目の open が「使用中」で必ず失敗する。よって open は1回だけ行い、失敗時は再起動を促す。
        let conn = match Connection::open(&db_path) {
            Ok(conn) => conn,
            Err(e) => {
                let error_msg = format!("{:?}", e);
                if error_msg.contains("WAL file") || error_msg.contains("replaying WAL") {
                    eprintln!(
                        "[DB Recovery] Database open failed (WAL). WAL was already backed up before open. Please restart the application once."
                    );
                }
                return Err(format!("Database error: {}", error_msg).into());
            }
        };

        // DuckDBの設定
        conn.execute("PRAGMA memory_limit='1GB'", []).ok();
        conn.execute("PRAGMA threads=4", []).ok();
        conn.execute("PRAGMA wal_autocheckpoint='1000'", []).ok(); // 1000ページごとに自動チェックポイント

        // スキーマ初期化
        schema::init_database(&conn)?;

        eprintln!("Database initialized successfully");

        Ok(DatabaseManager {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Exclusive access to database connection via closure.
    /// The lock is held only for the duration of the closure execution.
    /// Connection reference cannot escape the closure scope.
    pub async fn with_connection<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Connection) -> R + Send,
        R: Send,
    {
        let guard = self.conn.lock().await;
        f(&guard)
    }

    /// データベースファイルのパスを取得
    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// グレースフルシャットダウン - WALをフラッシュ
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("[DB Shutdown] Starting graceful shutdown...");

        let conn = self.conn.lock().await;

        // WALチェックポイントを強制実行（全データをメインDBにフラッシュ）
        match conn.execute("CHECKPOINT", []) {
            Ok(_) => eprintln!("[DB Shutdown] CHECKPOINT completed successfully"),
            Err(e) => eprintln!("[DB Shutdown] CHECKPOINT failed: {}", e),
        }

        eprintln!("[DB Shutdown] Shutdown completed");
        Ok(())
    }

    /// 定期的なチェックポイント（データ安全性向上）
    #[allow(dead_code)]
    pub async fn checkpoint(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.with_connection(|conn| {
            conn.execute("CHECKPOINT", [])?;
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // テスト用のヘルパー関数
    fn get_connection_with_path(
        db_path: PathBuf,
    ) -> Result<Connection, Box<dyn std::error::Error>> {
        let conn = Connection::open(&db_path)
            .db_context("open test database")
            .map_err(|e| e.to_string())?;
        Ok(conn)
    }

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
