//! Safety / backup module — the heart of Reveille's "never corrupt your settings" promise.
//!
//! Lifecycle:
//!   begin_seeding()  -> recover_if_needed(); back up real INI; write seed INI; drop flag
//!   restore()        -> copy backup back over the real INI; remove flag
//!   recover_if_needed() (also called on app startup) -> if a flag exists, a previous
//!                       session was interrupted, so restore immediately and self-heal.
//!
//! Invariants:
//!   * The backup is captured from the user's REAL settings, and is NEVER overwritten
//!     while a seed session is active (the flag guards this). This is what prevents us
//!     from ever backing up the low-power config and losing the user's real one.
//!   * A real GameUserSettings.ini must already exist before we touch anything. If the
//!     user never launched Squad, we hard-block rather than improvise a config we can't
//!     cleanly undo (see preflight()).

use std::fs;
use std::path::{Path, PathBuf};

/// The low-power config, compiled into the binary.
const SEED_INI: &str = include_str!("../resources/SeedGameUserSettings.ini");

/// Process name we watch / guard against.
pub const SQUAD_PROCESS: &str = "SquadGame.exe";

/// Maps any error to a user-facing string (Tauri commands serialize errors as strings).
type Res<T> = Result<T, String>;

fn io<T>(r: std::io::Result<T>, ctx: &str) -> Res<T> {
    r.map_err(|e| format!("{ctx}: {e}"))
}

/// Owns every path we read or write. Constructed once from the app's data dir.
pub struct Vault {
    /// The user's real, live config that Squad reads at launch.
    game_settings: PathBuf,
    /// App-owned backup of the user's real config.
    backup: PathBuf,
    /// Presence of this file means "a seed session is in progress / was not cleaned up".
    flag: PathBuf,
}

impl Vault {
    /// `app_data_dir` is Tauri's per-app data directory (where we keep the backup + flag,
    /// deliberately OUTSIDE the game folder so a Squad reinstall/verify can't clobber it).
    pub fn new(app_data_dir: &Path) -> Res<Self> {
        let game_settings = windows_client_config_dir()?.join("GameUserSettings.ini");
        Ok(Self {
            game_settings,
            backup: app_data_dir.join("GameUserSettings.backup.ini"),
            flag: app_data_dir.join("seeding.flag"),
        })
    }

    fn is_seeding(&self) -> bool {
        self.flag.exists()
    }

    /// Pre-flight gate. Returns Ok(()) only if it is safe to begin a swap we can fully undo.
    pub fn preflight(&self) -> Res<()> {
        if !self.game_settings.exists() {
            return Err(
                "Squad's settings file wasn't found. Launch Squad once normally so it \
                 creates your settings, then come back and try again."
                    .into(),
            );
        }
        Ok(())
    }

    /// Begin a seed session: ensure a clean baseline, back up the real INI, write the seed INI.
    /// Does NOT launch Squad — that is wired separately so the swap is decoupled from the launch.
    pub fn begin_seeding(&self) -> Res<()> {
        // If a prior session was interrupted, restore first so we back up the REAL config,
        // never the seed one.
        self.recover_if_needed()?;
        self.preflight()?;

        if let Some(parent) = self.backup.parent() {
            io(fs::create_dir_all(parent), "create app data dir")?;
        }
        io(
            fs::copy(&self.game_settings, &self.backup),
            "back up your settings",
        )?;
        // Write the flag BEFORE swapping, so a crash mid-write still self-heals next launch.
        io(fs::write(&self.flag, "seeding"), "write seeding flag")?;
        io(
            fs::write(&self.game_settings, SEED_INI),
            "apply seed settings",
        )?;
        Ok(())
    }

    /// Restore the user's real settings and clear the seed session. Idempotent and safe to
    /// call even if nothing is in progress (the panic "Restore" button calls this directly).
    pub fn restore(&self) -> Res<()> {
        if self.backup.exists() {
            io(
                fs::copy(&self.backup, &self.game_settings),
                "restore your settings",
            )?;
        }
        if self.flag.exists() {
            io(fs::remove_file(&self.flag), "clear seeding flag")?;
        }
        Ok(())
    }

    /// Called on every app startup: if a flag survived, a previous session died without
    /// restoring — heal it immediately.
    pub fn recover_if_needed(&self) -> Res<()> {
        if self.is_seeding() {
            self.restore()?;
        }
        Ok(())
    }
}

/// `%LOCALAPPDATA%\SquadGame\Saved\Config\WindowsClient`
fn windows_client_config_dir() -> Res<PathBuf> {
    #[cfg(windows)]
    {
        let local = std::env::var_os("LOCALAPPDATA")
            .ok_or("Could not locate your Windows user folder (LOCALAPPDATA).")?;
        Ok(PathBuf::from(local)
            .join("SquadGame")
            .join("Saved")
            .join("Config")
            .join("WindowsClient"))
    }
    #[cfg(not(windows))]
    {
        Err("Reveille only runs on Windows — Squad and EAC are Windows-only.".into())
    }
}

/// Guard used by pre-flight: refuse to start seeding if Squad is already running, because
/// the INI swap would not take effect and our watcher would restore the moment the user's
/// real session ended.
pub fn is_squad_running() -> bool {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_processes();
    sys.processes()
        .values()
        .any(|p| p.name().eq_ignore_ascii_case(SQUAD_PROCESS))
}
