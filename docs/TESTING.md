# Reveille — Windows test checklist

Reveille can only be fully tested on a real Windows box with Squad installed (Squad + EAC
don't run on macOS, and the launch/process-watch/overlay logic is Windows-specific).

## 0. Prerequisites (one-time, or CI will fail)

- [x] Icons added to `src-tauri/icons/` (sunrise icon generated; regenerate with
      `python3 scripts/make_icon.py` or replace with your own art).
- [x] Updater keypair generated; pubkey in `tauri.conf.json`; private key + password
      in repo secrets `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
      **Back up `~/.config/reveille/` — losing the private key means existing installs
      can't accept future updates.**
- [ ] Gist created and `GIST_URL` + `FALLBACK_CONFIG` filled in `src-ui/config.js` (see `docs/gist.md`).
- [ ] At least one server has a real `bmId`.
- [ ] (Ops) Proxy secret rotated + per-IP rate-limit added on Vercel.

## 1. Build

- [ ] `npm install`
- [ ] `npm run tauri build` (or download the CI artifact from a tagged release).
- [ ] Installer runs; app launches. (Unsigned → SmartScreen "More info → Run anyway" is expected.)

## 2. Settings safety (the non-negotiable part — test FIRST)

> Back up your real `GameUserSettings.ini` yourself before testing, just in case:
> `%LOCALAPPDATA%\SquadGame\Saved\Config\WindowsClient\GameUserSettings.ini`

- [ ] **Happy path:** note your current resolution/audio in Squad → click **Seed** → let Squad
      launch → quit Squad → confirm your **original** resolution/audio are back.
- [ ] **Panic button:** click **Seed**, then (before/while Squad runs) click **Restore my
      normal settings** → confirm the real INI is restored.
- [ ] **Crash self-heal:** click **Seed**, then **kill Reveille from Task Manager** while the
      seed INI is applied → reopen Reveille → confirm it auto-restores your real settings on
      startup (the `seeding.flag` should be gone afterward).
- [ ] **No-INI hard block:** rename/remove your `GameUserSettings.ini` (simulating "never
      launched Squad") → click **Seed** → expect a friendly "launch Squad once" message and
      **no** file written.
- [ ] **Already-running guard:** launch Squad normally first → click **Seed** → expect
      "Close Squad first" and no swap.

## 3. Launch + state machine

- [ ] Click **Seed** with Squad closed → state shows **Launching Squad…**.
- [ ] Squad starts in potato mode: **1024×768, borderless window, muted, low FPS**.
- [ ] Once the game process is up, state shows **🌱 Seeding**.
- [ ] Quit Squad → state shows **Squad closed · settings restored**, Seed re-enabled.
- [ ] **Timeout:** click **Seed**, then cancel/close the Steam launch prompt and don't start
      the game → after ~2 min, state shows **didn't start · settings restored**.

## 4. Overlay banner (highest-risk piece)

- [ ] While **Seeding**, a thin bar appears pinned to the **top of the screen** over Squad.
- [ ] It reads `🌱 7Cav SEEDING — low graphics …` and is **click-through** (mouse/keyboard go
      to the game, not the bar).
- [ ] It stays visible while in-game (this depends on borderless — if it vanishes, check
      `FullscreenMode=1` took effect).
- [ ] When the server crosses the popped threshold, the bar turns **green** and updates text.
- [ ] On quitting Squad, the bar disappears.

> If transparency/click-through misbehaves, note exactly how — the fallback is a solid
> (non-transparent) bar.

## 5. Server list + live population

- [ ] Picker lists your gist servers; the ⭐ priority one is auto-selected.
- [ ] `message` from the gist shows as a banner (if set).
- [ ] Population shows `X / Y players` for the selected server and refreshes ~every 60s.
- [ ] Switching servers updates the count.
- [ ] Edit the gist (change `message` / `priority`) → reopen app after a few minutes →
      change is reflected. Offline (no internet) → app still loads via cache/fallback.

## 6. Popped flow

- [ ] When the selected server is at/above the threshold, an **OS notification** fires once.
- [ ] The **✅ I'm done seeding** button appears; clicking it restores settings and shows a
      thank-you. (It should **not** force-close Squad.)

## 7. Auto-update (after a second release exists)

- [ ] Install an older version, publish a newer tagged release, reopen → it offers/installs
      the update.

---

### What to capture if something breaks
- The exact state label shown and what you did.
- Whether your real `GameUserSettings.ini` survived (most important).
- Contents of the app data dir: `%APPDATA%\com.7cav.reveille\` (look for `GameUserSettings.backup.ini` and `seeding.flag`).
- Any error text in the app's red error line.
