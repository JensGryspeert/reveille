// Prevents an extra console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod session;
mod settings;

use session::{launch_and_watch, SessionState};
use settings::{is_squad_running, Vault};

/// Pre-flight + back up real settings + apply the seed config, then launch Squad via the
/// Steam deeplink and start watching the process. Progress arrives as `seed-state` events.
#[tauri::command]
fn start_seeding(app: tauri::AppHandle) -> Result<(), String> {
    if is_squad_running() {
        return Err("Squad is already running. Close it first, then click Seed.".into());
    }
    Vault::for_app(&app)?.begin_seeding()?;
    launch_and_watch(app)
}

/// Panic button + normal end-of-session restore.
#[tauri::command]
fn restore_settings(app: tauri::AppHandle) -> Result<(), String> {
    Vault::for_app(&app)?.restore()
}

/// Self-heal any interrupted seed session. Called by the UI on launch.
#[tauri::command]
fn recover_on_startup(app: tauri::AppHandle) -> Result<(), String> {
    Vault::for_app(&app)?.recover_if_needed()
}

/// Show an OS notification. The UI calls this once when the server "pops" so the seeder
/// knows their job is done. (Done in Rust so the webview needs no bundled plugin JS.)
#[tauri::command]
fn notify(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| format!("notification failed: {e}"))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .manage(SessionState::default())
        .setup(|app| {
            // Belt-and-braces: heal on startup even before the UI asks.
            if let Ok(v) = Vault::for_app(&app.handle()) {
                let _ = v.recover_if_needed();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_seeding,
            restore_settings,
            recover_on_startup,
            notify
        ])
        .run(tauri::generate_context!())
        .expect("error while running Reveille");
}
