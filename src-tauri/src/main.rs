// Prevents an extra console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod settings;

use settings::{is_squad_running, Vault};
use tauri::Manager;

/// Builds a Vault rooted at the app's data dir.
fn vault(app: &tauri::AppHandle) -> Result<Vault, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("locate app data dir: {e}"))?;
    Vault::new(&dir)
}

/// Pre-flight + back up real settings + apply the seed config.
/// (Launching Squad via the Steam deeplink and watching the process are wired separately.)
#[tauri::command]
fn start_seeding(app: tauri::AppHandle) -> Result<(), String> {
    if is_squad_running() {
        return Err("Squad is already running. Close it first, then click Seed.".into());
    }
    vault(&app)?.begin_seeding()
}

/// Panic button + normal end-of-session restore.
#[tauri::command]
fn restore_settings(app: tauri::AppHandle) -> Result<(), String> {
    vault(&app)?.restore()
}

/// Self-heal any interrupted seed session. Called by the UI on launch.
#[tauri::command]
fn recover_on_startup(app: tauri::AppHandle) -> Result<(), String> {
    vault(&app)?.recover_if_needed()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Belt-and-braces: heal on startup even before the UI asks.
            if let Ok(v) = vault(&app.handle()) {
                let _ = v.recover_if_needed();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_seeding,
            restore_settings,
            recover_on_startup
        ])
        .run(tauri::generate_context!())
        .expect("error while running Reveille");
}
