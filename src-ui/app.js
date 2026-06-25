// Uses Tauri's global API (withGlobalTauri=true) so no bundler is needed.
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const $ = (id) => document.getElementById(id);
const seedBtn = $("seed-btn");
const restoreBtn = $("restore-btn");
const stateEl = $("state");
const errorEl = $("error");

function showError(msg) {
  errorEl.textContent = msg;
  errorEl.classList.remove("hidden");
}
function clearError() {
  errorEl.classList.add("hidden");
}

// Reflect the Rust state machine. `seedable` controls whether the Seed button is live.
const STATES = {
  idle: { text: "Idle", seedable: true },
  launching: { text: "Launching Squad…", seedable: false },
  seeding: { text: "🌱 Seeding — thanks!", seedable: false },
  stopped: { text: "Squad closed · settings restored", seedable: true },
  timeout: { text: "Squad didn't start · settings restored", seedable: true },
  error: { text: "Launch failed · settings restored", seedable: true },
};

function setState(name) {
  const s = STATES[name] ?? STATES.idle;
  stateEl.textContent = s.text;
  seedBtn.disabled = !s.seedable;
}

async function startSeeding() {
  clearError();
  seedBtn.disabled = true;
  try {
    await invoke("start_seeding");
    // The backend now drives state via `seed-state` events.
  } catch (e) {
    showError(String(e));
    setState("idle");
  }
}

async function restoreSettings() {
  clearError();
  try {
    await invoke("restore_settings");
    setState("idle");
  } catch (e) {
    showError(String(e));
  }
}

seedBtn.addEventListener("click", startSeeding);
restoreBtn.addEventListener("click", restoreSettings);

// Backend pushes transitions: launching → seeding → stopped/timeout/error.
listen("seed-state", (event) => {
  clearError();
  setState(event.payload);
});

// On launch, the backend self-heals any interrupted seed session (see settings.rs).
invoke("recover_on_startup")
  .then(() => setState("idle"))
  .catch((e) => showError(String(e)));
