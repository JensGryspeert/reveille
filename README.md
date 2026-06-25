# Reveille 🌅

**One-click Squad server seeder for 7Cav.** Launches Squad in an ultra-low-resource "seed" mode, points you at the server that needs people, shows live population, and safely restores all your normal settings the moment you quit.

> *Reveille* is the bugle call that wakes the unit at dawn — this tool wakes up sleeping servers.

## For members

1. Download the latest `Reveille_x.y.z_x64-setup.exe` from [Releases](../../releases).
2. Run it. (Windows may show "unknown publisher" → click **More info → Run anyway**. It's safe — source is right here.)
3. Click **Seed**. Squad launches in potato mode; join the highlighted server.
4. Play until the banner turns green — then quit whenever. Your normal graphics/audio settings come back automatically.

**Your real settings are never lost.** Reveille backs them up first and restores them on exit; if anything ever crashes, it self-heals the next time you open it. There's also a **Restore my normal settings** button if you ever need it.

## For developers

See [DESIGN.md](./DESIGN.md) for the full spec and rationale.

- **Stack:** [Tauri 2](https://tauri.app) — Rust backend + minimal webview UI.
- **Build:** Windows binaries are produced on **GitHub Actions** (you cannot cross-build Tauri for Windows from macOS, and Squad/EAC only run on Windows anyway).
- **Local dev:** requires Rust + Node. `npm install && npm run tauri dev`. Note: the seeding flow can only be *fully* tested on a real Windows box with Squad installed.

### Layout

```
src/                      # webview UI (vanilla HTML/JS)
src-tauri/
  src/
    main.rs               # Tauri entrypoint, commands, recovery-on-startup
    settings.rs           # SAFETY/BACKUP MODULE — backup, swap, restore, self-heal
  resources/
    SeedGameUserSettings.ini   # the low-power config written over the real one
  tauri.conf.json
  Cargo.toml
.github/workflows/release.yml  # CI: build Windows .exe + publish release
DESIGN.md
```

### Before the first CI build

- Add icons to `src-tauri/icons/` (see DESIGN.md action items).
- Generate the updater keypair and add `TAURI_SIGNING_PRIVATE_KEY` to repo secrets.
