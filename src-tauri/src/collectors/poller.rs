use crate::collectors::collector_trait::Collector;
use crate::database::{get_connection, models::Channel};
use duckdb::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tauri::AppHandle;

pub struct ChannelPoller {
    app_handle: AppHandle,
    collectors: HashMap<String, Arc<dyn Collector + Send + Sync>>,
    tasks: HashMap<i64, tokio::task::JoinHandle<()>>,
}

impl ChannelPoller {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            collectors: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn register_collector(&mut self, platform: String, collector: Arc<dyn Collector + Send + Sync>) {
        self.collectors.insert(platform, collector);
    }

    pub fn start_polling(&mut self, channel: Channel) -> Result<(), Box<dyn std::error::Error>> {
        if !channel.enabled {
            return Ok(());
        }

        let collector = self
            .collectors
            .get(&channel.platform)
            .ok_or_else(|| format!("No collector for platform: {}", channel.platform))?
            .clone();

        let channel_id = channel.id.unwrap();
        let app_handle = self.app_handle.clone();
        let poll_interval = Duration::from_secs(channel.poll_interval as u64);

        let task = tokio::spawn(async move {
            let mut interval = interval(poll_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            // 初回認証
            if let Err(e) = collector.start_collection(&channel).await {
                eprintln!("Failed to start collection for channel {}: {}", channel_id, e);
                return;
            }

            loop {
                interval.tick().await;

                // チャンネル情報を再取得（更新されている可能性があるため）
                let conn = match get_connection(&app_handle) {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("Failed to get database connection: {}", e);
                        continue;
                    }
                };

                let updated_channel = match Self::get_channel(&conn, channel_id) {
                    Ok(Some(ch)) => ch,
                    Ok(None) => {
                        // チャンネルが削除された場合はタスクを終了
                        break;
                    }
                    Err(e) => {
                        eprintln!("Failed to get channel: {}", e);
                        continue;
                    }
                };

                if !updated_channel.enabled {
                    // チャンネルが無効化された場合はタスクを終了
                    break;
                }

                // ポーリング実行
                match collector.poll_channel(&updated_channel).await {
                    Ok(Some(_stats)) => {
                        // TODO: ストリーム情報をデータベースに保存
                        // DatabaseWriter::insert_stream_stats(&conn, &stats)?;
                    }
                    Ok(None) => {
                        // 配信していない
                    }
                    Err(e) => {
                        eprintln!("Failed to poll channel {}: {}", channel_id, e);
                    }
                }
            }
        });

        self.tasks.insert(channel_id, task);
        Ok(())
    }

    pub fn stop_polling(&mut self, channel_id: i64) {
        if let Some(task) = self.tasks.remove(&channel_id) {
            task.abort();
        }
    }

    fn get_channel(conn: &Connection, channel_id: i64) -> Result<Option<Channel>, duckdb::Error> {
        let mut stmt = conn.prepare("SELECT id, platform, channel_id, channel_name, enabled, poll_interval, created_at, updated_at FROM channels WHERE id = ?")?;
        
        let rows: Result<Vec<_>, _> = stmt
            .query_map([channel_id], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    enabled: row.get(4)?,
                    poll_interval: row.get(5)?,
                    created_at: Some(row.get(6)?),
                    updated_at: Some(row.get(7)?),
                })
            })?
            .collect();

        match rows {
            Ok(mut channels) => Ok(channels.pop()),
            Err(e) => Err(e),
        }
    }
}
