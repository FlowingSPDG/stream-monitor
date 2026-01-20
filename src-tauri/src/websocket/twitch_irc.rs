use crate::database::models::ChatMessage;
use crate::database::writer::DatabaseWriter;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tungstenite::connect;
use tungstenite::protocol::Message;
use url::Url;

/// Twitch IRC接続管理構造体
pub struct TwitchIrcClient {
    stream_id: i64,
    channel_name: String,
    access_token: String,
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
}

impl TwitchIrcClient {
    pub fn new(stream_id: i64, channel_name: String, access_token: String) -> Self {
        Self {
            stream_id,
            channel_name,
            access_token,
            shutdown_tx: None,
        }
    }

    /// Twitch IRCに接続し、チャットメッセージを収集する
    pub async fn connect_and_collect(
        &mut self,
        db_conn: Arc<Mutex<duckdb::Connection>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Twitch IRCサーバーに接続
        let url = Url::parse("wss://irc-ws.chat.twitch.tv:443")?;
        let (mut socket, _response) = connect(url)?;

        // 認証
        socket.write_message(Message::Text(format!("PASS oauth:{}", self.access_token)))?;
        socket.write_message(Message::Text("NICK justinfan12345".to_string()))?;
        socket.write_message(Message::Text(format!("JOIN #{}", self.channel_name)))?;

        println!("Connected to Twitch IRC for channel: {}", self.channel_name);

        loop {
            tokio::select! {
                message = tokio::task::spawn_blocking(move || socket.read_message()) => {
                    match message?? {
                        Message::Text(text) => {
                            // デバッグログ（本番環境では削除）
                            if text.contains("PRIVMSG") {
                                println!("Received chat message for channel: {}", self.channel_name);
                            }

                            if let Some(chat_message) = self.parse_irc_message(&text) {
                                let conn = db_conn.lock().await;
                                match DatabaseWriter::insert_chat_message(&conn, &chat_message) {
                                    Ok(_) => {
                                        // 正常に保存された場合は何もしない
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to save chat message for channel {}: {}", self.channel_name, e);
                                        // エラーが発生しても接続を継続
                                    }
                                }
                            }

                            // PINGに応答して接続を維持
                            if text.starts_with("PING") {
                                let pong_response = "PONG :tmi.twitch.tv\r\n".to_string();
                                if let Err(e) = socket.write_message(Message::Text(pong_response)) {
                                    eprintln!("Failed to send PONG for channel {}: {}", self.channel_name, e);
                                    break;
                                }
                            }
                        }
                        Message::Close(_) => {
                            println!("Twitch IRC connection closed for channel: {}", self.channel_name);
                            break;
                        }
                        _ => {}
                    }
                }
                _ = shutdown_rx.recv() => {
                    println!("Shutting down Twitch IRC connection for channel: {}", self.channel_name);
                    break;
                }
            }
        }

        Ok(())
    }

    /// IRCメッセージをパースしてChatMessageに変換
    fn parse_irc_message(&self, message: &str) -> Option<ChatMessage> {
        // PRIVMSG形式: :user!user@user.tmi.twitch.tv PRIVMSG #channel :message
        if let Some(privmsg_start) = message.find("PRIVMSG") {
            let after_privmsg = &message[privmsg_start..];

            // ユーザー名の抽出
            if let Some(user_end) = message.find('!') {
                let user_name = message[1..user_end].to_string();

                // メッセージ内容の抽出
                if let Some(msg_start) = after_privmsg.find(" :") {
                    let message_content = after_privmsg[msg_start + 2..].to_string();

                    return Some(ChatMessage {
                        id: None,
                        stream_id: self.stream_id,
                        timestamp: Utc::now().to_rfc3339(),
                        platform: "twitch".to_string(),
                        user_id: None, // Twitch IRCではユーザーIDを取得できない
                        user_name,
                        message: message_content,
                        message_type: "normal".to_string(),
                    });
                }
            }
        }

        None
    }

    /// 接続をシャットダウンする
    pub fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}

/// 複数のTwitch IRC接続を管理するマネージャー
pub struct TwitchIrcManager {
    connections: Arc<Mutex<std::collections::HashMap<String, TwitchIrcClient>>>,
    db_conn: Arc<Mutex<duckdb::Connection>>,
}

impl TwitchIrcManager {
    pub fn new(db_conn: Arc<Mutex<duckdb::Connection>>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
            db_conn,
        }
    }

    /// 指定したチャンネルのIRC接続を開始
    pub async fn start_channel_collection(
        &self,
        stream_id: i64,
        channel_name: &str,
        access_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.lock().await;

        if connections.contains_key(channel_name) {
            return Ok(()); // 既に接続中
        }

        let mut client = TwitchIrcClient::new(
            stream_id,
            channel_name.to_string(),
            access_token.to_string(),
        );

        let db_conn_clone = Arc::clone(&self.db_conn);
        let channel_name_clone = channel_name.to_string();

        tokio::spawn(async move {
            if let Err(e) = client.connect_and_collect(db_conn_clone).await {
                eprintln!("Twitch IRC collection failed for {}: {}", channel_name_clone, e);
            }
        });

        connections.insert(channel_name.to_string(), client);
        Ok(())
    }

    /// 指定したチャンネルのIRC接続を停止
    pub async fn stop_channel_collection(&self, channel_name: &str) {
        let mut connections = self.connections.lock().await;
        if let Some(client) = connections.remove(channel_name) {
            client.shutdown();
        }
    }

    /// 全てのIRC接続を停止
    pub async fn stop_all_collections(&self) {
        let mut connections = self.connections.lock().await;
        for client in connections.values() {
            client.shutdown();
        }
        connections.clear();
    }
}