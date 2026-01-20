use duckdb::Connection;

pub fn init_database(conn: &Connection) -> Result<(), duckdb::Error> {
    // channels テーブル: 監視対象チャンネル設定
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS channels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            platform TEXT NOT NULL CHECK(platform IN ('twitch', 'youtube')),
            channel_id TEXT NOT NULL,
            channel_name TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            poll_interval INTEGER NOT NULL DEFAULT 60,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(platform, channel_id)
        )
        "#,
        [],
    )?;

    // streams テーブル: 配信基本情報
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS streams (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            channel_id INTEGER NOT NULL,
            stream_id TEXT NOT NULL,
            title TEXT,
            category TEXT,
            started_at TIMESTAMP NOT NULL,
            ended_at TIMESTAMP,
            FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
            UNIQUE(channel_id, stream_id)
        )
        "#,
        [],
    )?;

    // stream_stats テーブル: 定期収集統計データ
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stream_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            stream_id INTEGER NOT NULL,
            collected_at TIMESTAMP NOT NULL,
            viewer_count INTEGER,
            chat_rate_1min INTEGER DEFAULT 0,
            FOREIGN KEY (stream_id) REFERENCES streams(id) ON DELETE CASCADE
        )
        "#,
        [],
    )?;

    // インデックス作成
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_streams_channel_id ON streams(channel_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_streams_started_at ON streams(started_at)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_stream_id ON stream_stats(stream_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_collected_at ON stream_stats(collected_at)",
        [],
    )?;

    Ok(())
}
