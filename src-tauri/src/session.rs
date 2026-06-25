//! Launch + process-watch state machine, plus the always-on-top seed banner window.
//!
//! The Steam deeplink gives us no child handle — Steam (not us) spawns Squad, often
//! 20–60s later after the EAC handshake. So we watch the OS process list by name:
//!
//!   launching  -> fire deeplink, poll for SquadGame.exe to APPEAR (≤ APPEAR_TIMEOUT)
//!   seeding    -> process is up; open the banner; poll until it DISAPPEARS
//!   stopped    -> process gone; close banner; restore the real settings
//!   timeout    -> never appeared (user cancelled the Steam launch); restore anyway
//!
//! Every transition is emitted to the UI as a `seed-state` event. Settings are always
//! restored before we go idle, so the "never corrupt" guarantee holds on every path.
//!
//! The banner is a SEPARATE, click-through, always-on-top window — never injected into
//! the game (EAC ban risk). It stays visible because the seed INI forces borderless.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

use crate::settings::{is_squad_running, Vault};

/// Steam deeplink for Squad (app id 393380). Launching via Steam keeps EAC happy;
/// launching the bare exe would be kicked instantly.
pub const STEAM_DEEPLINK: &str = "steam://rungameid/393380";

/// How long to wait for the game process to show up before giving up.
const APPEAR_TIMEOUT: Duration = Duration::from_secs(120);
/// Process-list poll cadence.
const POLL_INTERVAL: Duration = Duration::from_secs(3);

const BANNER_LABEL: &str = "banner";
const BANNER_HEIGHT: f64 = 46.0;

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

        // Phase 2 — game is up; show the banner and wait for it to exit.
        open_banner(&app);
        emit(&app, "seeding");
        while is_squad_running() {
            tokio::time::sleep(POLL_INTERVAL).await;
        }

        finish(&app, "stopped");
    });

    Ok(())
}

/// Close the banner, restore the real settings, clear the watching guard, and report
/// the terminal state.
fn finish(app: &AppHandle, state: &str) {
    close_banner(app);
    if let Ok(v) = Vault::for_app(app) {
        let _ = v.restore();
    }
    app.state::<SessionState>()
        .watching
        .store(false, Ordering::SeqCst);
    emit(app, state);
}

/// Width of the primary monitor in logical pixels (falls back to a sane default).
/// Queried via the main window, which is the portable way to reach monitor info.
fn primary_logical_width(app: &AppHandle) -> f64 {
    if let Some(win) = app.get_webview_window("main") {
        if let Ok(Some(m)) = win.primary_monitor() {
            return (m.size().width as f64) / m.scale_factor();
        }
    }
    1280.0
}

/// Open the click-through, always-on-top seed banner spanning the top of the screen.
/// Window creation is dispatched to the main thread for cross-platform safety.
fn open_banner(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if handle.get_webview_window(BANNER_LABEL).is_some() {
            return;
        }
        let width = primary_logical_width(&handle);
        let built = WebviewWindowBuilder::new(
            &handle,
            BANNER_LABEL,
            WebviewUrl::App("banner.html".into()),
        )
        .title("Reveille")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .shadow(false)
        .inner_size(width, BANNER_HEIGHT)
        .position(0.0, 0.0)
        .build();

        if let Ok(win) = built {
            // Click-through: the banner never steals mouse/keyboard from the game.
            let _ = win.set_ignore_cursor_events(true);
        }
    });
}

/// Close the banner window if it exists.
fn close_banner(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(win) = handle.get_webview_window(BANNER_LABEL) {
            let _ = win.close();
        }
    });
}
