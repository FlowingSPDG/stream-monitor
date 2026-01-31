use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, Runtime};

const CLIENT_NAME: &[u8] = b"stream-stats-collector";

/// Stronghold-based secure storage for tokens and secrets
/// 
/// This implementation uses events to communicate with the frontend Stronghold JavaScript API
/// because Rust-side direct access to the plugin state is complex.
pub struct StrongholdStore;

impl StrongholdStore {
    /// Get the vault snapshot path
    pub fn snapshot_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = app
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?
            .join("vault.hold");
        Ok(path)
    }

    /// Check if vault is initialized (file exists)
    pub fn is_initialized<R: Runtime>(app: &AppHandle<R>) -> bool {
        if let Ok(path) = Self::snapshot_path(app) {
            path.exists()
        } else {
            false
        }
    }

    /// Save a token - will be handled by frontend Stronghold integration
    pub fn save_token_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
        token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("[StrongholdStore] Requesting frontend to save token for platform: '{}'", platform);
        
        // Emit event to frontend to save via JavaScript Stronghold API
        app.emit("stronghold:save-token", serde_json::json!({
            "platform": platform,
            "token": token
        }))?;
        
        eprintln!("[StrongholdStore] Token save request sent to frontend");
        Ok(())
    }

    pub fn save_token(
        _platform: &str,
        _token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("[StrongholdStore] WARNING: save_token called without AppHandle");
        Ok(())
    }

    /// Get a token - requires async frontend communication
    pub fn get_token_with_app<R: Runtime>(
        _app: &AppHandle<R>,
        platform: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        eprintln!("[StrongholdStore] WARNING: Synchronous token retrieval not supported for '{}'", platform);
        Err(format!("Token retrieval requires async frontend communication").into())
    }

    pub fn get_token(
        platform: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err(format!("Cannot retrieve token for '{}' without AppHandle", platform).into())
    }

    pub fn delete_token_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        app.emit("stronghold:delete-token", serde_json::json!({
            "platform": platform
        }))?;
        Ok(())
    }

    pub fn delete_token(
        _platform: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn has_token_with_app<R: Runtime>(_app: &AppHandle<R>, _platform: &str) -> bool {
        false // Cannot determine synchronously
    }

    pub fn has_token(_platform: &str) -> bool {
        false
    }

    pub fn save_oauth_secret_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
        secret: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        app.emit("stronghold:save-secret", serde_json::json!({
            "platform": platform,
            "secret": secret
        }))?;
        Ok(())
    }

    pub fn save_oauth_secret(
        _platform: &str,
        _secret: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn get_oauth_secret_with_app<R: Runtime>(
        _app: &AppHandle<R>,
        platform: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err(format!("OAuth secret retrieval for '{}' requires async frontend communication", platform).into())
    }

    pub fn get_oauth_secret(
        platform: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err(format!("Cannot retrieve OAuth secret for '{}' without AppHandle", platform).into())
    }

    pub fn delete_oauth_secret_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        app.emit("stronghold:delete-secret", serde_json::json!({
            "platform": platform
        }))?;
        Ok(())
    }

    pub fn delete_oauth_secret(
        _platform: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn has_oauth_secret_with_app<R: Runtime>(_app: &AppHandle<R>, _platform: &str) -> bool {
        false
    }

    pub fn has_oauth_secret(_platform: &str) -> bool {
        false
    }
}
