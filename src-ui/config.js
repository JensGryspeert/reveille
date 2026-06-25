// Shared runtime config. Loaded as a plain <script> (no bundler) so it must define
// globals on window.REVEILLE before app.js / banner.js run.
//
// Network calls (gist + battlemetrics-proxy) happen in JS on purpose: the webview has
// csp:null so fetch to external hosts is allowed, and it keeps the Rust binary tiny.
// The proxy secret is shipped here — per design decision A it's weak obfuscation only
// (server population is public), and the Vercel endpoint is rate-limited per IP.
window.REVEILLE = {
  // Always-latest raw gist URL (no revision hash, so edits propagate).
  GIST_URL:
    "https://gist.githubusercontent.com/JensGryspeert/a3eb42238a9a9b69ddaa0664853eface/raw/servers.json",

  // battlemetrics-proxy (generic BM pass-through): GET ?path=<bm api path>
  PROXY_URL: "https://battlemetrics-proxy-eight.vercel.app/api/bm",
  PROXY_SECRET: "bm-proxy-2026-grayson-j", // TODO: rotate; treat as obfuscation only.

  POLL_MS: 60000,
  DEFAULT_POPPED_THRESHOLD: 50,

  // Used when the gist can't be fetched and there's no cache yet. Mirrors the gist.
  FALLBACK_CONFIG: {
    poppedThreshold: 40,
    message: "",
    servers: [
      {
        name: "=7Cav= Squad Tactical Realism | RAAS & Invasion | Discord.gg/7Cav",
        ip: "148.113.198.221",
        port: 7787,
        bmId: "23497207",
        priority: true,
      },
    ],
  },
};
