/// StreamRepository - 配信一覧・タイムライン用レポジトリ
///
/// streams / stream_stats / channels / chat_messages を用いた
/// 配信一覧・MW計算・タイムラインポイント取得を提供します。
use chrono::Local;
use duckdb::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub id: i64,
    pub stream_id: String,
    pub channel_id: i64,
    pub channel_name: String,
    pub title: String,
    pub category: String,
    pub started_at: String,
    pub ended_at: String,
    pub peak_viewers: i32,
    pub avg_viewers: i32,
    pub duration_minutes: i32,
    pub minutes_watched: i64,
    pub follower_gain: i32,
    pub total_chat_messages: i64,
    pub engagement_rate: f64,
    pub last_collected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub collected_at: String,
    pub viewer_count: i32,
    pub chat_rate_1min: i32,
    pub category: String,
    pub title: String,
    pub follower_count: i32,
}

fn row_to_stream_info(row: &duckdb::Row) -> Result<StreamInfo, duckdb::Error> {
    Ok(StreamInfo {
        id: row.get::<_, i64>(0)?,
        stream_id: row.get::<_, String>(1)?,
        channel_id: row.get::<_, i64>(2)?,
        channel_name: row.get::<_, String>(3)?,
        title: row.get::<_, String>(4)?,
        category: row.get::<_, String>(5)?,
        started_at: row.get::<_, String>(6)?,
        ended_at: row.get::<_, String>(7).unwrap_or_default(),
        peak_viewers: row.get::<_, i32>(8)?,
        avg_viewers: row.get::<_, i32>(9)?,
        duration_minutes: row.get::<_, i32>(10)?,
        minutes_watched: row.get::<_, i64>(11)?,
        follower_gain: row.get::<_, i32>(12)?,
        total_chat_messages: row.get::<_, i64>(13)?,
        engagement_rate: row.get::<_, f64>(14)?,
        last_collected_at: row.get::<_, String>(15).unwrap_or_default(),
    })
}

const STREAM_METRICS_CTE: &str = r#"
    WITH stream_metrics AS (
        SELECT 
            s.id,
            s.stream_id,
            s.channel_id,
            s.title,
            s.category,
            s.started_at,
            s.ended_at,
            COALESCE(MAX(ss.viewer_count), 0) as peak_viewers,
            COALESCE(AVG(ss.viewer_count), 0) as avg_viewers,
            COALESCE(
                EXTRACT(EPOCH FROM (
                    COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at
                )) / 60,
                0
            ) as duration_minutes,
            MAX(ss.collected_at) as last_collected_at
        FROM streams s
        LEFT JOIN stream_stats ss ON s.id = ss.stream_id
"#;

const STREAM_SELECT_TAIL: &str = r#"
        SELECT 
            sm.id,
            sm.stream_id,
            sm.channel_id,
            c.channel_name,
            COALESCE(sm.title, '') as title,
            COALESCE(sm.category, '') as category,
            CAST(sm.started_at AS VARCHAR) as started_at,
            CAST(sm.ended_at AS VARCHAR) as ended_at,
            sm.peak_viewers,
            sm.avg_viewers,
            sm.duration_minutes,
            COALESCE(mw.minutes_watched, 0) as minutes_watched,
            COALESCE(fc.follower_gain, 0) as follower_gain,
            COALESCE(cc.total_chat_messages, 0) as total_chat_messages,
            CASE 
                WHEN COALESCE(mw.minutes_watched, 0) > 0 
                THEN (COALESCE(cc.total_chat_messages, 0)::DOUBLE / mw.minutes_watched::DOUBLE) * 1000.0
                ELSE 0.0
            END as engagement_rate,
            CAST(sm.last_collected_at AS VARCHAR) as last_collected_at
        FROM stream_metrics sm
        JOIN channels c ON sm.channel_id = c.id
        LEFT JOIN mw_calc mw ON sm.id = mw.stream_id
        LEFT JOIN follower_calc fc ON sm.id = fc.stream_id
        LEFT JOIN chat_calc cc ON sm.id = cc.id
"#;

pub struct StreamRepository;

impl StreamRepository {
    /// チャンネル別の配信一覧を取得
    pub fn get_channel_streams(
        conn: &Connection,
        channel_id: i64,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<StreamInfo>, duckdb::Error> {
        let limit_clause = limit.unwrap_or(50);
        let offset_clause = offset.unwrap_or(0);
        let query = format!(
            r#"
        {}
        WHERE s.channel_id = ?
        GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT ss.stream_id, ss.viewer_count, ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
        ),
        mw_calc AS (
            SELECT stream_id,
                COALESCE(SUM(COALESCE(viewer_count, 0) * EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60), 0)::BIGINT as minutes_watched
            FROM stats_with_next WHERE next_collected_at IS NOT NULL GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT ss.stream_id, COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id) AND ss.follower_count IS NOT NULL
            GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT s.id, COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s LEFT JOIN chat_messages cm ON s.id = cm.stream_id
            WHERE s.channel_id = ?
            GROUP BY s.id
        )
        {} ORDER BY sm.started_at DESC LIMIT {} OFFSET {}
        "#,
            STREAM_METRICS_CTE, STREAM_SELECT_TAIL, limit_clause, offset_clause
        );
        let mut stmt = conn.prepare(&query)?;
        let channel_id_str = channel_id.to_string();
        let rows = stmt.query_map([&channel_id_str, &channel_id_str], row_to_stream_info)?;
        rows.collect::<Result<Vec<_>, _>>()
    }

    /// 日付範囲で配信一覧を取得（全チャンネル）
    pub fn get_streams_by_date_range(
        conn: &Connection,
        date_from: &str,
        date_to: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<StreamInfo>, duckdb::Error> {
        let limit_clause = limit.unwrap_or(100);
        let offset_clause = offset.unwrap_or(0);
        let query = format!(
            r#"
        {}
        WHERE CAST(s.started_at AS DATE) >= CAST(? AS DATE) AND CAST(s.started_at AS DATE) <= CAST(? AS DATE)
        GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT ss.stream_id, ss.viewer_count, ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
        ),
        mw_calc AS (
            SELECT stream_id,
                COALESCE(SUM(COALESCE(viewer_count, 0) * EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60), 0)::BIGINT as minutes_watched
            FROM stats_with_next WHERE next_collected_at IS NOT NULL GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT ss.stream_id, COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id) AND ss.follower_count IS NOT NULL
            GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT s.id, COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s LEFT JOIN chat_messages cm ON s.id = cm.stream_id
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = s.id)
            GROUP BY s.id
        )
        {} ORDER BY sm.started_at DESC LIMIT {} OFFSET {}
        "#,
            STREAM_METRICS_CTE, STREAM_SELECT_TAIL, limit_clause, offset_clause
        );
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([date_from, date_to], row_to_stream_info)?;
        rows.collect::<Result<Vec<_>, _>>()
    }

    /// 単一配信の詳細情報を取得
    pub fn get_stream_info_by_id(
        conn: &Connection,
        stream_id: i64,
    ) -> Result<StreamInfo, duckdb::Error> {
        let query = format!(
            r#"
        {}
        WHERE s.id = ?
        GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT ss.stream_id, ss.viewer_count, ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss WHERE ss.stream_id = ?
        ),
        mw_calc AS (
            SELECT stream_id,
                COALESCE(SUM(COALESCE(viewer_count, 0) * EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60), 0)::BIGINT as minutes_watched
            FROM stats_with_next WHERE next_collected_at IS NOT NULL GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT ss.stream_id, COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss WHERE ss.stream_id = ? AND ss.follower_count IS NOT NULL GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT s.id, COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s LEFT JOIN chat_messages cm ON s.id = cm.stream_id WHERE s.id = ? GROUP BY s.id
        )
        {}
        "#,
            STREAM_METRICS_CTE, STREAM_SELECT_TAIL
        );
        let stream_id_str = stream_id.to_string();
        conn.query_row(
            &query,
            [
                &stream_id_str,
                &stream_id_str,
                &stream_id_str,
                &stream_id_str,
            ],
            row_to_stream_info,
        )
    }

    /// 比較用：基準配信と時間帯が重なる配信をサジェスト
    pub fn get_suggested_streams_for_comparison(
        conn: &Connection,
        base_stream_id: i64,
        limit: Option<i32>,
    ) -> Result<Vec<StreamInfo>, duckdb::Error> {
        let base = Self::get_stream_info_by_id(conn, base_stream_id)?;
        let limit_clause = limit.unwrap_or(50);
        let base_start = base.started_at.clone();
        let base_end = if base.ended_at.is_empty() {
            Local::now().to_rfc3339()
        } else {
            base.ended_at.clone()
        };
        let query = format!(
            r#"
        WITH base_stream AS (
            SELECT id, channel_id, started_at, ended_at, category FROM streams WHERE id = ?
        ),
        stream_metrics AS (
            SELECT s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at,
                COALESCE(MAX(ss.viewer_count), 0) as peak_viewers,
                COALESCE(AVG(ss.viewer_count), 0) as avg_viewers,
                COALESCE(EXTRACT(EPOCH FROM (COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at)) / 60, 0) as duration_minutes,
                MAX(ss.collected_at) as last_collected_at
            FROM streams s LEFT JOIN stream_stats ss ON s.id = ss.stream_id
            WHERE s.id != ? AND s.started_at < CAST(? AS TIMESTAMP)
              AND COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) > CAST(? AS TIMESTAMP)
            GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT ss.stream_id, ss.viewer_count, ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
        ),
        mw_calc AS (
            SELECT stream_id,
                COALESCE(SUM(COALESCE(viewer_count, 0) * EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60), 0)::BIGINT as minutes_watched
            FROM stats_with_next WHERE next_collected_at IS NOT NULL GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT ss.stream_id, COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id) AND ss.follower_count IS NOT NULL
            GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT s.id, COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s LEFT JOIN chat_messages cm ON s.id = cm.stream_id
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = s.id) GROUP BY s.id
        )
        {} ORDER BY CASE WHEN sm.category = (SELECT category FROM base_stream) THEN 0 ELSE 1 END, sm.started_at ASC LIMIT {}
        "#,
            STREAM_SELECT_TAIL, limit_clause
        );
        let base_id_str = base_stream_id.to_string();
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(
            [&base_id_str, &base_id_str, &base_end, &base_start],
            row_to_stream_info,
        )?;
        rows.collect::<Result<Vec<_>, _>>()
    }

    /// 配信のタイムラインポイント一覧を取得
    pub fn get_timeline_stats(
        conn: &Connection,
        stream_id: i64,
    ) -> Result<Vec<TimelinePoint>, duckdb::Error> {
        let query = r#"
        SELECT 
            CAST(ss.collected_at AS VARCHAR) as collected_at,
            ss.viewer_count,
            COALESCE((
                SELECT COUNT(*) FROM chat_messages cm
                WHERE cm.stream_id = ss.stream_id
                  AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                  AND cm.timestamp < ss.collected_at
            ), 0) AS chat_rate_1min,
            ss.category,
            ss.title,
            ss.follower_count
        FROM stream_stats ss
        WHERE ss.stream_id = ?
        ORDER BY ss.collected_at ASC
        "#;
        let mut stmt = conn.prepare(query)?;
        let stream_id_str = stream_id.to_string();
        let rows = stmt.query_map([&stream_id_str], |row| {
            Ok(TimelinePoint {
                collected_at: row.get::<_, String>(0)?,
                viewer_count: row.get::<_, i32>(1).unwrap_or_default(),
                chat_rate_1min: row.get::<_, i32>(2)?,
                category: row.get::<_, String>(3).unwrap_or_default(),
                title: row.get::<_, String>(4).unwrap_or_default(),
                follower_count: row.get::<_, i32>(5).unwrap_or_default(),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    }
}
