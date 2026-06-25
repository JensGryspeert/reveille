// Shared runtime config. Loaded as a plain <script> (no bundler) so it must define
// globals on window.REVEILLE before app.js / banner.js run.
//
// Network calls (gist + battlemetrics-proxy) happen in JS on purpose: the webview has
// csp:null so fetch to external hosts is allowed, and it keeps the Rust binary tiny.
// The proxy secret is shipped here — per design decision A it's weak obfuscation only
// (server population is public), and the Vercel endpoint is rate-limited per IP.
window.REVEILLE = {
  // Always-latest raw gist URL. Replace REPLACE_GIST_ID with the real gist id.
  GIST_URL:
    "https://gist.githubusercontent.com/JensGryspeert/REPLACE_GIST_ID/raw/servers.json",

  // battlemetrics-proxy (generic BM pass-through): GET ?path=<bm api path>
  PROXY_URL: "https://battlemetrics-proxy-eight.vercel.app/api/bm",
  PROXY_SECRET: "bm-proxy-2026-grayson-j", // TODO: rotate; treat as obfuscation only.

  POLL_MS: 60000,
  DEFAULT_POPPED_THRESHOLD: 50,

  // Used when the gist can't be fetched and there's no cache yet. Edit before shipping.
  FALLBACK_CONFIG: {
    poppedThreshold: 50,
    message: "",
    servers: [
      {
        name: "7Cav Squad #1",
        ip: "0.0.0.0",
        port: 27015,
        bmId: "REPLACE_BM_SERVER_ID",
        priority: true,
      },
    ],
  },
};
