# Offline UI verification

Start `npm run dev -- --host 127.0.0.1 --port 1425` from the repository root.
Open `/tests/browser/diagnostic-states.html?mode=unverified` on that server.
Modes: `unverified`, `simulated`, `standard`, `stale`, `aging`, `coding`.

- Unverified: no numeric result or healthy badge, even with a malformed numeric input.
- Simulated: number visible, explicit simulation label, no healthy badge.
- Standard: number visible, reference-range wording (not a health guarantee).
- Stale / aging: no current health assessment after 30 seconds without a new sample.
- Coding: grouped, searchable candidates; searching `zrcatka` matches `zrcátka`; no execute button.

These fixtures contain synthetic data and call no Tauri IPC. They verify rendering,
not vehicle compatibility or the native IPC boundary. They are not production build
entry points. Backend tests exercise identity gating and production response parsers.

## Navigation regression

`/tests/browser/navigation-demo.html` mounts the actual App/session hook with a
strictly in-memory IPC stub and two sample catalog entries. At 980×680 and 1280×820:

1. Click Demo without scrolling the sidebar. The OFFLINE DEMO badge appears.
2. Click Kódování in the top main navigation, without scrolling. Both candidate cards appear.
3. Expand adapter details: the area list still has a nonzero, scrollable viewport.
4. Search for `zrcatek`, return to Diagnostika, then reopen Kódování.
5. The top navigation remains visible; no write action is offered.

`window.__demoNavigationCalls` records mocked IPC calls; only catalog, ports,
event subscriptions, demo connection/disconnection and optional read-only scan
commands are accepted. This is UI/IPC-contract verification, not a native car test.
