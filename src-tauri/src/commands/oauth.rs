use crate::config::credentials::CredentialManager;
use crate::config::settings::SettingsManager;
use crate::oauth::{server::OAuthServer, twitch::TwitchOAuth, youtube::YouTubeOAuth};
use tauri::AppHandle;


#[tauri::command]
pub async fn login_with_twitch(app_handle: AppHandle, port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(8080);
    let redirect_uri = format!("http://localhost:{}/callback", port);

    // 設定ファイルからClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    let client_id = settings.twitch.client_id
        .ok_or_else(|| "Twitch Client ID is not configured. Please set it in settings first.".to_string())?;

    // keyringからClient Secretを取得
    let client_secret = CredentialManager::get_oauth_secret("twitch")
        .map_err(|e| format!("Failed to get Twitch Client Secret: {}. Please configure OAuth settings first.", e))?;

    let oauth = TwitchOAuth::new(client_id, client_secret, redirect_uri);
    let server = OAuthServer::new(port);

    oauth
        .authenticate(server)
        .await
        .map_err(|e| format!("Twitch authentication failed: {}", e))
}

#[tauri::command]
pub async fn login_with_youtube(app_handle: AppHandle, port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(8081);
    let redirect_uri = format!("http://localhost:{}/callback", port);

    // 設定ファイルからClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    let client_id = settings.youtube.client_id
        .ok_or_else(|| "YouTube Client ID is not configured. Please set it in settings first.".to_string())?;

    // keyringからClient Secretを取得
    let client_secret = CredentialManager::get_oauth_secret("youtube")
        .map_err(|e| format!("Failed to get YouTube Client Secret: {}. Please configure OAuth settings first.", e))?;

    let oauth = YouTubeOAuth::new(client_id, client_secret, redirect_uri);
    let server = OAuthServer::new(port);

    oauth
        .authenticate(server)
        .await
        .map_err(|e| format!("YouTube authentication failed: {}", e))
}
