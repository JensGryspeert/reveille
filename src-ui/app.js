// Uses Tauri's global API (withGlobalTauri=true) so no bundler is needed.
const { invoke } = window.__TAURI__.core;
const { listen, emit } = window.__TAURI__.event;
const C = window.REVEILLE;

const $ = (id) => document.getElementById(id);
const seedBtn = $("seed-btn");
const doneBtn = $("done-btn");
const restoreBtn = $("restore-btn");
const serverSel = $("server");
const stateEl = $("state");
const popEl = $("population");
const msgEl = $("message");
const errorEl = $("error");

let config = null;
let poppedNotified = false;

function showError(msg) {
  errorEl.textContent = msg;
  errorEl.classList.remove("hidden");
}
function clearError() {
  errorEl.classList.add("hidden");
}

// ---- State machine display -------------------------------------------------
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
  if (s.seedable) {
    // Returned to idle-ish: clear the popped UI for the next session.
    doneBtn.classList.add("hidden");
    poppedNotified = false;
  }
}

// ---- Config (gist) ---------------------------------------------------------
const CACHE_KEY = "reveille:config";

async function loadConfig() {
  try {
    const res = await fetch(C.GIST_URL, { cache: "no-store" });
    if (!res.ok) throw new Error("gist " + res.status);
    const data = await res.json();
    localStorage.setItem(CACHE_KEY, JSON.stringify(data));
    return data;
  } catch (e) {
    const cached = localStorage.getItem(CACHE_KEY);
    if (cached) return JSON.parse(cached);
    return C.FALLBACK_CONFIG;
  }
}

function renderServers() {
  serverSel.innerHTML = "";
  (config.servers ?? []).forEach((s, i) => {
    const opt = document.createElement("option");
    opt.value = String(i);
    opt.textContent = s.priority ? `⭐ ${s.name}` : s.name;
    serverSel.appendChild(opt);
  });
  // Auto-select the priority server, else the first.
  const idx = (config.servers ?? []).findIndex((s) => s.priority);
  serverSel.value = String(idx >= 0 ? idx : 0);

  if (config.message) {
    msgEl.textContent = config.message;
    msgEl.classList.remove("hidden");
  } else {
    msgEl.classList.add("hidden");
  }
}

function selectedServer() {
  return (config.servers ?? [])[Number(serverSel.value)] ?? null;
}

function threshold(server) {
  return (
    server?.poppedThreshold ??
    config?.poppedThreshold ??
    C.DEFAULT_POPPED_THRESHOLD
  );
}

// ---- Population polling -----------------------------------------------------
async function fetchPopulation(bmId) {
  const url = `${C.PROXY_URL}?path=${encodeURIComponent("/servers/" + bmId)}`;
  const res = await fetch(url, { headers: { "x-proxy-secret": C.PROXY_SECRET } });
  if (!res.ok) throw new Error("proxy " + res.status);
  const a = (await res.json())?.data?.attributes ?? {};
  return {
    name: a.name,
    current: a.players ?? 0,
    max: a.maxPlayers ?? 0,
    online: a.status === "online",
  };
}

async function poll() {
  const server = selectedServer();
  if (!server) return;
  if (!server.bmId || server.bmId.includes("REPLACE")) {
    popEl.textContent = "Live population not configured for this server.";
    popEl.classList.remove("hidden");
    return;
  }

  try {
    const p = await fetchPopulation(server.bmId);
    const limit = threshold(server);
    const popped = p.current >= limit;

    popEl.textContent = p.online
      ? `${p.current} / ${p.max} players${popped ? " 🎉 popped!" : ""}`
      : "Server offline";
    popEl.classList.remove("hidden");

    // Tell the banner window.
    emit("population", { name: server.name, current: p.current, max: p.max, popped });

    // Fire the "popped" notification + reveal the done button once per session.
    if (popped && !poppedNotified) {
      poppedNotified = true;
      doneBtn.classList.remove("hidden");
      invoke("notify", {
        title: "Server popped! 🎉",
        body: `${server.name} hit ${p.current}/${p.max}. You can stop seeding whenever — thanks!`,
      }).catch(() => {});
    }
  } catch (e) {
    popEl.textContent = "Couldn't reach the server tracker.";
    popEl.classList.remove("hidden");
  }
}

// ---- Actions ----------------------------------------------------------------
async function startSeeding() {
  clearError();
  seedBtn.disabled = true;
  poppedNotified = false;
  doneBtn.classList.add("hidden");
  try {
    await invoke("start_seeding");
    // Backend drives state via `seed-state` events from here.
  } catch (e) {
    showError(String(e));
    setState("idle");
  }
}

async function finishSeeding() {
  clearError();
  try {
    await invoke("restore_settings");
    stateEl.textContent = "Done — thanks for seeding! 🌱";
    seedBtn.disabled = false;
    doneBtn.classList.add("hidden");
  } catch (e) {
    showError(String(e));
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
doneBtn.addEventListener("click", finishSeeding);
restoreBtn.addEventListener("click", restoreSettings);
serverSel.addEventListener("change", () => {
  poppedNotified = false;
  poll();
});

// Backend pushes transitions: launching → seeding → stopped/timeout/error.
listen("seed-state", (event) => {
  clearError();
  setState(event.payload);
});

// ---- Boot -------------------------------------------------------------------
(async () => {
  config = await loadConfig();
  renderServers();
  await invoke("recover_on_startup").catch((e) => showError(String(e)));
  setState("idle");
  poll();
  setInterval(poll, C.POLL_MS);
})();
