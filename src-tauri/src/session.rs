//! Launch + process-watch state machine.
//!
//! The Steam deeplink gives us no child handle — Steam (not us) spawns Squad, often
//! 20–60s later after the EAC handshake. So we watch the OS process list by name:
//!
//!   launching  -> fire deeplink, poll for SquadGame.exe to APPEAR (≤ APPEAR_TIMEOUT)
//!   seeding    -> process is up; poll until it DISAPPEARS
//!   stopped    -> process gone; restore the real settings
//!   timeout    -> never appeared (user cancelled the Steam launch); restore anyway
//!
//! Every transition is emitted to the UI as a `seed-state` event. Settings are always
//! restored before we go idle, so the "never corrupt" guarantee holds on every path.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;

use crate::settings::{is_squad_running, Vault};

/// Steam deeplink for Squad (app id 393380). Launching via Steam keeps EAC happy;
/// launching the bare exe would be kicked instantly.
pub const STEAM_DEEPLINK: &str = "steam://rungameid/393380";

/// How long to wait for the game process to show up before giving up.
const APPEAR_TIMEOUT: Duration = Duration::from_secs(120);
/// Process-list poll cadence.
const POLL_INTERVAL: Duration = Duration::from_secs(3);

/// Guards against a second session being started while one is already being watched.
#[derive(Default)]
pub struct SessionState {
    watching: AtomicBool,
}

fn emit(app: &AppHandle, state: &str) {
    let _ = app.emit("seed-state", state);
}

/// Fire the deeplink and spawn the watcher. Assumes the seed INI has already been
/// applied by `Vault::begin_seeding`. Returns immediately; progress arrives via events.
pub fn launch_and_watch(app: AppHandle) -> Result<(), String> {
    use tauri::Manager;
    // Scope the State borrow so `app` is free to move into the spawned task below.
    {
        let session = app.state::<SessionState>();
        if session.watching.swap(true, Ordering::SeqCst) {
            return Err("A seed session is already in progress.".into());
        }
    }

    if let Err(e) = app.opener().open_url(STEAM_DEEPLINK, None::<&str>) {
        // Roll back the swap so we never leave the user in seed config on a failed launch.
        finish(&app, "error");
        return Err(format!("Could not launch Squad via Steam: {e}"));
    }

    emit(&app, "launching");

    tauri::async_runtime::spawn(async move {
        // Phase 1 — wait for SquadGame.exe to appear.
        let mut waited = Duration::ZERO;
        let mut appeared = false;
        while waited < APPEAR_TIMEOUT {
            if is_squad_running() {
                appeared = true;
                break;
            }
            tokio::time::sleep(POLL_INTERVAL).await;
            waited += POLL_INTERVAL;
        }

        if !appeared {
            finish(&app, "timeout");
            return;
        }

        // Phase 2 — game is up; wait for it to exit.
        emit(&app, "seeding");
        while is_squad_running() {
            tokio::time::sleep(POLL_INTERVAL).await;
        }

        finish(&app, "stopped");
    });

    Ok(())
}

/// Restore the real settings, clear the watching guard, and report the terminal state.
fn finish(app: &AppHandle, state: &str) {
    use tauri::Manager;
    if let Ok(v) = Vault::for_app(app) {
        let _ = v.restore();
    }
    app.state::<SessionState>()
        .watching
        .store(false, Ordering::SeqCst);
    emit(app, state);
}
