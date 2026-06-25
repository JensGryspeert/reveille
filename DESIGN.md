# Reveille — 7Cav Squad Seeder

> *Reveille* — the bugle call that wakes the unit at dawn. This tool wakes up sleeping Squad servers.

A tiny Windows desktop app that makes it effortless for non-technical 7Cav members to **seed** Squad servers — by running the game in an ultra-low-resource "seed" config, pointed at the right server, with a live population display and bulletproof settings restoration.

## What it is NOT

- **Not a seeder box.** Automated 24/7 fake-player boxes need one paid account + Steam copy *per* body, invite anti-cheat / AFK-kick problems, and are frowned upon. Rejected.
- **Not headless.** Squad has no headless client, and EAC (Easy Anti-Cheat) refuses connections from anything not launched through the Steam/EAC path. The game window *always* runs.
- **Not resource magic.** The "low resources" comes entirely from a low-power graphics INI (1024×768, all quality at 0, audio muted, low FPS cap). Nothing more exotic is possible.

## Core mechanism

1. Back up the user's real `GameUserSettings.ini` to an **app-owned** copy.
2. Write a low-power **seed INI** over it — 1024×768, all scalability 0, audio muted, low FPS cap, **forced borderless-windowed**.
3. Launch via Steam deeplink `steam://rungameid/393380` (mandatory — keeps EAC happy).
4. Watch for the `SquadGame.exe` process; **restore the backup the moment it exits**.

The real INI lives at:
`%LOCALAPPDATA%\SquadGame\Saved\Config\WindowsClient\GameUserSettings.ini`

## Decisions (locked)

| Area | Decision |
|------|----------|
| **Audience** | Non-technical Windows users. Double-click `.exe`, zero prerequisites. Must *never* corrupt real settings. |
| **Safety model** | App-owned backup of the real INI + a self-healing "seeding-in-progress" flag (restores on next launch if anything crashed) + auto-restore when Squad exits + a visible panic **"Restore my normal settings"** button. The backup is never overwritten while a seed session is active. |
| **Launch — Tier A** | (ships first) Swap INI + fire deeplink. Squad opens to the menu in potato mode; user joins the server from the browser/favorites. Dead reliable. |
| **Launch — Tier B** | (best-effort follow-up) Auto-connect via `+connect IP:port` in Steam launch options, with automatic fallback to Tier A if it fails. |
| **Launch — Tier C** | Launching the bare EAC exe directly. **Ruled out** — bypasses EAC handshake, instant kick. |
| **Stack** | **Tauri** (tiny binary). Rust backend + minimal webview UI. Builds on a **GitHub Actions Windows runner** (cannot cross-build from macOS). WebView2 bootstrapper embedded in the installer for true zero-prereq. |
| **Server config** | Remote **gist** JSON, always-latest raw URL, CDN-cached (~minutes). Schema: `{ servers: [{name, ip, port, priority}], message }`. Baked-in offline fallback list; cache last-known. |
| **Live population** | Yes in v1. Poll the Vercel battlemetrics-proxy **every 60s, active server only, while app is open**. |
| **Proxy secret** | **Option A** — a client-shipped secret can't stay secret, so treat it as weak obfuscation and **rate-limit the Vercel endpoint per-IP**. (Data behind it — server population — is already public.) |
| **Restore detection** | Process-name polling state machine: `WAITING_FOR_LAUNCH → SEEDING → (process gone) → restore → IDLE`. ~2 min appear-timeout. Guard: refuse to start if `SquadGame.exe` is **already running** ("Close Squad first"). |
| **Overlay banner** | External **click-through, always-on-top** window floating *over* the game. **NO injection / no render hooking — EVER** (EAC ban risk). Seed config forces **borderless-windowed** so the banner stays visible. |
| **Auto-update** | **Tauri updater** vs **GitHub Releases** from day one. Updater signing keypair lives in CI secrets; public key ships in the app. |
| **Code signing** | **Unsigned v1** + an illustrated Discord "More info → Run anyway" guide. Azure Trusted Signing as a fast-follow if the SmartScreen scare-screen actually costs adoption. |
| **Pre-flight gate** | On "Seed" click: verify Steam present, Squad installed, and a real `GameUserSettings.ini` exists to back up. **Hard-block if no INI exists** (user never launched Squad) — never improvise a config we can't cleanly undo. |
| **"Server popped"** | **Notify + one-click graceful exit.** Desktop notification + green banner; convenience button to restore & quit. **Never force-quit** a live match. |

## Action items (operational)

- [ ] **Rotate** the battlemetrics-proxy secret (was pasted in plaintext during design).
- [ ] **Add per-IP rate-limiting** to the Vercel proxy endpoint.
- [ ] Add app icons before the first CI build (Tauri bundle requires them).
- [ ] Generate the Tauri updater signing keypair; store private key in GitHub Actions secrets.

## Seed INI

The seed config (`src-tauri/resources/SeedGameUserSettings.ini`) is the **battle-tested
config from [Matsozetex/Squad-Seed-Tool](https://github.com/Matsozetex/Squad-Seed-Tool),
used verbatim**, with one deliberate Reveille change: `FullscreenMode=1` (borderless
windowed) pinned in `[/Script/Engine.GameUserSettings]` so the always-on-top banner stays
visible (the source left fullscreen mode to chance). It also confirms `PlayerNamePrefix="SEED"`
is a real, supported Squad setting — kept so admins can spot seeders; customizable for 7Cav.

Because the user's real INI is backed up and restored, the seed INI being a fixed blob
(including the source author's benchmark/emote/sensitivity values) is harmless — it only
applies during a seed session and is reverted on exit. Still worth re-verifying against the
current Squad version on Windows after major game updates.

## Deferred to later versions

- Tier B auto-connect.
- Code signing (Azure Trusted Signing).

## Build & test loop

Develop Rust/JS on macOS → push → **CI builds the Windows `.exe`** → test the *actual* launch/connect/restore behavior on a **real Windows box** (Squad + EAC won't run on macOS at all).
