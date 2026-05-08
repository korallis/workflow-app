# codex-bridge — Implementation guide

## Patterns

- **`reqwest::Client`** for all HTTPS (token endpoint + Codex Responses API).
- **`tiny_http`** or **`axum`** for the local OAuth callback server. `axum` is overkill but matches the rest of our stack; `tiny_http` is one-shot and simpler for v1.
- **PKCE** hand-rolled with `ring` or `sha2` + `base64::URL_SAFE_NO_PAD`. The `oauth2` crate is overkill (lots of features we don't need).
- **JWT account_id extraction** via base64-decode + `serde_json` of the middle segment. We never verify the signature — we trust the TLS connection that delivered it.
- **SSE parsing** via [`eventsource-stream`](https://crates.io/crates/eventsource-stream) crate (Tokio-friendly).
- **Auth file storage** with `serde_json` + atomic writes (`write to .tmp` → `fsync` → `rename`). Permissions set via `std::os::unix::fs::PermissionsExt` (chmod 0600) on Linux/macOS.

## Conventions

- Crate name: `kit-codex-bridge`.
- API endpoint constant: `CODEX_API_URL = "https://chatgpt.com/backend-api/codex/responses"`.
- Token URL constant: `OPENAI_TOKEN_URL = "https://auth.openai.com/oauth/token"`.
- Auth path: derived once at app start; passed into `CodexBridge::new`.
- All HTTP requests have a 30s connect + read timeout.
- All public types `Send + Sync` for Tauri.

## Module boundaries

This module **owns**: the Codex OAuth flow, token storage + refresh, Codex Responses API client, SSE parsing, schema validation.

This module **must NOT**: spawn `codex` CLI; assemble prompts (skill-runner does that); persist state beyond auth.json (session-store does that); invoke Anthropic.

## Known gotchas

- **`store: false` is REQUIRED.** Per ChatGPT backend behaviour. The backend rejects `true` for OAuth auth. Hard-code it; never expose as an option.
- **`stream: true` is REQUIRED.** Hard-code it.
- **Message IDs must be stripped** when sending prior messages back. The backend rejects payloads containing message IDs in stateless mode. Strip recursively.
- **`reasoning.encrypted_content`** from prior turns is the way to maintain reasoning context across stateless calls. Pass it back when present.
- **`instructions` is the system prompt.** Codex CLI uses cached/bundled prompts; we provide our own from skill-runner.
- **PKCE verifier length.** OpenAI accepts 43–128 URL-safe base64 chars. Use 32 random bytes → 43 chars after encoding. Don't go shorter.
- **State validation.** Always check returned `state` matches; mismatch = abort + warn (CSRF protection).
- **Local server port 1455 may be in use.** Detect EADDRINUSE; fall back to manual-code-paste flow with a clear error.
- **Browser-can't-reach-localhost (SSH/headless).** Show manual-code-paste UI immediately alongside the URL; race the two paths.
- **JWT extraction without verification.** Don't use `jsonwebtoken::decode` (requires key); use `decode_header` + manual base64 of the middle segment, or just split on `.` and decode segment 1.
- **Token refresh race.** Two parallel `execute()` calls when token is near expiry both try to refresh. Wrap auth state in `Arc<RwLock<...>>`; the refresher acquires the write lock and other callers wait.
- **429 quota errors.** Backend returns these when subscription quota exhausts ([openai-python#2951](https://github.com/openai/openai-python/issues/2951)). Don't auto-retry; surface to the user.

## Test patterns

- **Mock OAuth server** via `wiremock` crate for unit tests of the auth flow.
- **Mock Codex Responses API** via `wiremock` returning canned SSE; assert client parses correctly.
- **Real OAuth integration test** (gated on `CODEX_INTEGRATION_TEST=1`) — opens a real browser, requires manual completion. Skip in CI.
- **Real inference test** (gated on valid auth.json) — round-trip a 1-token prompt; assert response.
- **Schema test:** feed a request with `output_schema` requiring fields the model can't produce; assert violations.
- **Token refresh test:** auth.json with `expires_at` in the past; call `execute()`; assert refresh happens; assert auth.json updated.
- **Auth-file permissions test:** create auth.json; assert mode is 0600 (Unix only).

## Cargo dependencies (load-bearing)

```toml
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
eventsource-stream = "0.2"             # SSE parsing
sha2 = "0.10"                          # PKCE challenge
base64 = "0.22"                        # PKCE + JWT segment
rand = "0.8"                           # state + verifier
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonschema = "0.18"                    # output_schema validation
url = "2"                              # authorize URL building
open = "5"                             # cross-platform browser open
tiny_http = "0.12"                     # callback server (or axum if we already have it)
chrono = "0.4"
thiserror = "1"
tracing = "0.1"
```

## Implementation order

1. PKCE + state generation + authorize URL builder.
2. Local callback server (one-shot listener; cancellable).
3. Manual-code-paste fallback path.
4. Token exchange + refresh.
5. Auth file (load/save/atomic-rename + chmod 0600).
6. JWT account_id extraction.
7. Codex Responses API request builder (with all the mandatory invariants).
8. SSE parser + event normalisation.
9. Schema validator.
10. Wire up `LoginCallbacks` and `execute()`.
11. Tests.

## Reference implementations

- **Pi (TypeScript reference):** [`badlogic/pi-mono` `packages/ai/src/utils/oauth/openai-codex.ts`](https://github.com/badlogic/pi-mono/blob/main/packages/ai/src/utils/oauth/openai-codex.ts) and [`packages/ai/src/providers/openai-codex-responses.ts`](https://cdn.jsdelivr.net/npm/@oh-my-pi/pi-ai@13.18.0/src/providers/openai-codex-responses.ts).
- **Python (reference for stripped-down impl):** [`codex-auth` PyPI](https://pypi.org/project/codex-auth/) and [`langchain-codex-oauth`](https://pypi.org/project/langchain-codex-oauth/).
- **Swift (auth headers + usage endpoint):** [`steipete/CodexBar` `docs/codex-oauth.md`](https://github.com/steipete/CodexBar/blob/v0.20/docs/codex-oauth.md).
- **TypeScript (architecture notes):** [`ndycode/oc-codex-multi-auth`](https://github.com/ndycode/oc-codex-multi-auth/blob/main/docs/development/ARCHITECTURE.md) — covers `store: false`, message-ID stripping, reasoning encrypted_content.

When in doubt about request shape, port from Pi's TypeScript verbatim.
