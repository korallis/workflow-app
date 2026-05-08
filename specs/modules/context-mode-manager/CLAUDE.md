# context-mode-manager — Implementation guide

## Patterns

- **`tokio::process::Command`** for subprocess; `stdio: Pipe` so we own stdin/stdout for MCP.
- **`rmcp::Client::connect_stdio`** wraps the subprocess's stdio in a JSON-RPC client.
- **Health task** as a `tokio::spawn` with a cancellation token; cancelled on `shutdown()`.
- **No-op manager** is a separate impl behind a trait so callers don't branch on `Option`.

## Conventions

- Crate name: `kit-context-mode-manager`.
- Subprocess working dir: project root.
- Logs to `~/.kit-workflow-app/context-mode/supervisor.log` with size-based rotation (10 MB).

## Module boundaries

This module **owns**: the context-mode subprocess + MCP client connection + telemetry caching.

This module **must NOT**: replace context-mode tools with our own implementations; modify context-mode's content store directly.

## Known gotchas

- **rmcp version pinning.** Tauri may pin a specific version via dep tree; match it.
- **Subprocess stdout buffering.** Use `--unbuffered`-equivalent flags on context-mode if available; otherwise consume stdout aggressively.
- **`Ctrl+C` propagation.** Tauri's `WindowEvent::CloseRequested` should call `manager.shutdown().await` — wire this up via a shutdown registry.
- **Stdin closure on shutdown.** Drop the rmcp client first (closes stdin); subprocess sees EOF and exits cleanly. SIGTERM is the fallback.
- **Health-task cancellation.** Use `CancellationToken` (from `tokio_util`); avoid `Drop`-based cancellation which can race.

## Test patterns

- Mock subprocess + rmcp client in unit tests; verify args.
- Real subprocess test gated on context-mode being installed.
- Crash-restart test: kill the supervised process, verify 1 restart + recovery.
- No-op manager test: assert all `ctx_*` calls return identity-like values without spawning anything.

## Cargo dependencies

- `tokio`, `rmcp` (1.6.0), `serde`, `serde_json`, `thiserror`, `tracing`, `tokio-util`
