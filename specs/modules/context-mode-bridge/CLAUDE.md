# context-mode-bridge — Implementation guide

## Patterns

- Pi extension factory pattern: `default async function (ctx: PiExtensionContext) { ... }`.
- Subprocess via `child_process.spawn`; never `exec`.
- MCP handshake using a shared `@modelcontextprotocol/sdk` Node client.
- PreToolUse hook receives `(toolName, args)` and rewrites `args` to route through `ctx_execute` for bash/web tools. Pure transform; no global state.
- Health check via `setInterval(30000, …)` + `clearInterval` on deactivate.

## Conventions

- File layout: `extensions/context-mode-bridge/{index.ts, supervisor.ts, hook.ts, telemetry.ts}`.
- All paths absolute. Subprocess working dir: project root.
- Logs to `~/.kit-workflow-app/context-mode/supervisor.log` with rotation at 10 MB.

## Module boundaries

This module **owns**: context-mode subprocess lifecycle, MCP client connection, PreToolUse hook routing, telemetry caching.

This module **must NOT**: modify context-mode's content store directly (use `ctx_*` tools); replace context-mode tools with our own implementations.

## Known gotchas

- **Subprocess pipe buffering.** Use `stdin/stdout/stderr: 'pipe'` and explicit consumption; otherwise pipes can fill and block.
- **MCP handshake timeout.** Give 10s; surface a clear error if context-mode doesn't initialise.
- **Hook chaining.** If Pi already has PreToolUse handlers, register ours via the chain API, don't replace.
- **`Ctrl+C` propagation.** Pi's SIGINT handler should call our `deactivate`; verify on Linux + macOS.
- **`KIT_CONTEXT_MODE_DISABLE=1` must be checked synchronously** at activate-time; lazy checks are too late.

## Test patterns

- Mock subprocess + MCP client in unit tests; verify routing logic.
- Real subprocess test gated on context-mode being installed.
- Hook routing test: feed a bash tool call, assert the wrapped form is `ctx_execute(command=...)`.
- Crash-restart test: kill the supervised process, verify 1 restart + recovery.
