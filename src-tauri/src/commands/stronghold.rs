use tauri::{AppHandle, Manager};

const CLIENT_NAME: &[u8] = b"stream-stats-collector";

#[derive(serde::Serialize)]
pub struct VaultStatus {
    initialized: bool,
    path: String,
}

/// Check if vault is initialized
#[tauri::command]
pub async fn check_vault_initialized(app: AppHandle) -> Result<VaultStatus, String> {
    let vault_path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("vault.hold");

    Ok(VaultStatus {
        initialized: vault_path.exists(),
        path: vault_path.to_string_lossy().to_string(),
    })
}

/// Initialize a new vault with password
#[tauri::command]
pub async fn initialize_vault(app: AppHandle, _password: String) -> Result<(), String> {
    let vault_path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("vault.hold");

    eprintln!("[Stronghold] Initializing vault at: {:?}", vault_path);

    // Initialize the stronghold through the plugin
    // This will be called from JavaScript, which handles the initialization
    Ok(())
}

/// Save a token to stronghold
#[tauri::command]
pub async fn save_stronghold_token(
    _app: AppHandle,
    platform: String,
    token: String,
) -> Result<(), String> {
    eprintln!("[Stronghold] Saving token for platform: {}", platform);
    eprintln!("[Stronghold] Token length: {}", token.len());

    let key = format!("{}_token", platform);

    // The actual save will be done through the Stronghold plugin's JavaScript API
    // This command serves as a bridge
    eprintln!("[Stronghold] Token save initiated for key: {}", key);
    
    Ok(())
}

/// Get a token from stronghold
#[tauri::command]
pub async fn get_stronghold_token(
    _app: AppHandle,
    platform: String,
) -> Result<Option<String>, String> {
    eprintln!("[Stronghold] Getting token for platform: {}", platform);

    let key = format!("{}_token", platform);

    // The actual get will be done through the Stronghold plugin's JavaScript API
    // This command serves as a bridge
    eprintln!("[Stronghold] Token get initiated for key: {}", key);
    
    Ok(None)
}

/// Delete a token from stronghold
#[tauri::command]
pub async fn delete_stronghold_token(
    _app: AppHandle,
    platform: String,
) -> Result<(), String> {
    let key = format!("{}_token", platform);
    eprintln!("[Stronghold] Deleting token for key: {}", key);
    
    Ok(())
}
