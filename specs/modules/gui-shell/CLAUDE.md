# gui-shell — Implementation guide

## Patterns

- **Tauri 2 standard layout** per `create-tauri-app v2`: `src-tauri/` (Rust), `src/` (React), `tauri.conf.json` at root.
- **Pi subprocess management** in `src-tauri/src/pi.rs`: `Mutex<RpcChannel>` shared state, spawn at app start, reap on shutdown. ELVES `DECISIONS.md` pattern adapted.
- **Tauri commands** are thin shims — most business logic happens in Pi (the engine). The Rust backend handles only: subprocess management, direct SQLite reads (not writes), tree-sitter direct calls (cheaper than RPC), auth detection.
- **Event channels:** `pi://event` for Pi-originated events; `kit://track-update` for track-state Tauri events.

## Conventions

- Rust crate: `kit-gui-shell`.
- React app: `kit-gui-app` in `src/`. Strict TS (per ELVES decisions).
- File naming: kebab-case for files, PascalCase for components.
- Zustand stores in `src/stores/` (one file per slice).
- Components in `src/components/` (organised by feature, not type).
- All Tauri commands typed in shared `src/types/tauri.ts` mirroring `src-tauri` types via `tauri-specta` or hand-written.

## Module boundaries

This module **owns**: the desktop window, all UI, Pi process supervision, direct filesystem reads for the file picker, auth detection.

This module **must NOT**: write to SQLite (engine does); spawn Codex/Claude directly (bridges do); modify spec files (kit-engine does).

## Known gotchas

- **Pi subprocess shutdown.** Use Tauri's `Window::on_event(WindowEvent::CloseRequested)` to send the shutdown RPC before tearing down the runtime. Otherwise Pi gets SIGKILLed without flushing session JSONL.
- **Tauri v2 capabilities.** The capabilities/* files allow specific commands; missing one = silent failure in production builds (works in dev). Audit on every release.
- **Tailwind v4 token leak.** `@theme` tokens defined in CSS leak globally; namespace component-specific overrides via CSS modules or scoped Tailwind layers.
- **CodeMirror 6 + Tauri.** Diff merge view requires CSS that conflicts with Tailwind preflight; isolate via shadow DOM or CSS layer.
- **xterm.js + WebGL.** WebGL renderer is faster but Wayland (some Linux setups) can have issues; fall back to canvas renderer if WebGL fails to initialise.
- **macOS code signing** required for distribution. Document the signing key path; never commit certs.

## Test patterns

- **Vitest + jsdom + @testing-library/react** for component tests (per ELVES `DECISIONS.md`).
- Mock `invoke` and Tauri events in unit tests via `@tauri-apps/api/mocks`.
- **`cargo test`** for Rust backend (subprocess management, auth detection, SQLite reads).
- E2E via `tauri-driver` + WebDriver: spawn the app, click through onboarding, assert Plan board renders.
- **Bundle size CI check.** Fail if macOS aarch64 .dmg > 30 MB or Linux AppImage > 40 MB.
- **Visual regression via Playwright** screenshots for the four primary screens.
