// Banner overlay window. Click-through (set_ignore_cursor_events in Rust) — it only
// listens for events and renders text; it never touches the game.
const { listen } = window.__TAURI__.event;

const bar = document.getElementById("bar");
const text = document.getElementById("text");

const BASE = "🌱 7Cav SEEDING — low graphics · settings auto-restore on quit";

// Population updates pushed from the main window every poll.
listen("population", (event) => {
  const { current, max, popped } = event.payload ?? {};
  if (popped) {
    bar.classList.add("popped");
    text.textContent = `🎉 Server popped (${current}/${max})! You can stop seeding anytime — thanks!`;
  } else {
    bar.classList.remove("popped");
    const count =
      typeof current === "number" && typeof max === "number"
        ? ` · ${current}/${max} players`
        : "";
    text.textContent = BASE + count;
  }
});

// If the session ends, the Rust side closes this window — but reset text defensively.
listen("seed-state", (event) => {
  if (event.payload !== "seeding") {
    bar.classList.remove("popped");
    text.textContent = BASE;
  }
});
