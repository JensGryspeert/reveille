# Server config gist

Reveille reads its server list from a public GitHub gist at startup, so you can change
servers (or push a "seed this one tonight" message) **without** shipping a new build.

## Set it up

1. Create a public gist at <https://gist.github.com> with a file named `servers.json`.
   Paste the contents of [`servers.example.json`](./servers.example.json) and edit it.
2. Grab the **always-latest raw URL** (drop the revision hash so it always serves the
   newest content):
   `https://gist.githubusercontent.com/<your-user>/<gist-id>/raw/servers.json`
3. Put that URL in `src-ui/config.js` → `GIST_URL`, and mirror the same servers into
   `FALLBACK_CONFIG` (used only when offline with no cache yet).

> Raw gist content is CDN-cached for a few minutes, so edits propagate with a short delay.

## Schema

| Field | Where | Meaning |
|-------|-------|---------|
| `poppedThreshold` | top-level | Player count at which a server counts as "popped" (default 50). |
| `message` | top-level | Optional banner shown in the app. Empty/omit to hide. |
| `servers[].name` | per server | Display name in the picker. |
| `servers[].ip` / `port` | per server | Game server address (display now; used by Tier B auto-connect later). |
| `servers[].bmId` | per server | **BattleMetrics server id** — required for live population. |
| `servers[].priority` | per server | `true` marks tonight's target; it's auto-selected and shown with ⭐. |
| `servers[].poppedThreshold` | per server | Optional per-server override of the top-level threshold. |

## Finding a server's `bmId`

Open the server on <https://www.battlemetrics.com>; the id is the number in the URL:
`https://www.battlemetrics.com/servers/squad/`**`1234567`** → `bmId: "1234567"`.

You can sanity-check it through your proxy:
`…/api/bm?path=/servers/1234567` (with the `x-proxy-secret` header) should return JSON
whose `data.attributes.players` / `maxPlayers` reflect the live server.
