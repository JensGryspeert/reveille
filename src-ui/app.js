// Uses Tauri's global API (withGlobalTauri=true) so no bundler is needed.
const { invoke } = window.__TAURI__.core;

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

async function startSeeding() {
  clearError();
  seedBtn.disabled = true;
  try {
    // Pre-flight + backup + INI swap happen in Rust. Launch is wired separately.
    await invoke("start_seeding");
    stateEl.textContent = "Launching Squad…";
  } catch (e) {
    showError(String(e));
    seedBtn.disabled = false;
  }
}

async function restoreSettings() {
  clearError();
  try {
    await invoke("restore_settings");
    stateEl.textContent = "Idle";
    seedBtn.disabled = false;
  } catch (e) {
    showError(String(e));
  }
}

seedBtn.addEventListener("click", startSeeding);
restoreBtn.addEventListener("click", restoreSettings);

// On launch, the backend self-heals any interrupted seed session (see settings.rs).
invoke("recover_on_startup").catch((e) => showError(String(e)));
