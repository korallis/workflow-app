# codex-bridge — SPEC

> **Layer:** Core (Rust crate `kit-codex-bridge`)
> **Last updated:** 2026-05-08 (Pi-removal pivot — clarified that this is **direct OAuth + Responses API**, not a subprocess wrapper around `codex` CLI)

## 1. Purpose

**Direct integration** with OpenAI's Codex Responses API using PKCE OAuth against the user's ChatGPT Plus/Pro subscription. We do NOT shell out to `codex` CLI. We replicate Pi's [`openai-codex-responses`](https://github.com/badlogic/pi-mono/blob/main/packages/ai/src/utils/oauth/openai-codex.ts) provider in Rust.

**Why direct, not subprocess:** Pi's pattern works because [OpenAI explicitly endorses third-party OAuth use](https://developers.openai.com/community/codex-for-oss) ("Codex for Open Source" program) and lists Pi/OpenCode/Cline/OpenClaw as supported tools. This contrasts with Anthropic, whose ToS forbids OAuth tokens being used outside Claude Code itself (which is why claude-bridge MUST stay as a subprocess wrapper). The asymmetry is not a design choice — it's mandated by the two providers' policies.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CXB-1 | First-run user with ChatGPT Plus | Click "Connect ChatGPT" in onboarding; complete OAuth in browser | My subscription powers Codex calls without typing API keys |
| CXB-2 | skill-runner running `/project-execute` | Send a Codex inference call with our system prompt + schema-validated output | Final reports are machine-readable; bleed-through fails structurally |
| CXB-3 | skill-runner | Stream SSE deltas back as ClaudeBridge-equivalent events | UI shows live progress |
| CXB-4 | track-engine via DispatchFn | Spawn one Codex inference per track with a per-track session ID | Prompt caching applies per track; sessions are isolated |
| CXB-5 | The bridge itself | Auto-refresh expired access tokens | Long-running app sessions don't break |

## 3. Public API

```rust
pub struct CodexBridge {
    http: reqwest::Client,
    auth: Arc<RwLock<CodexAuth>>,            // serialised JWT credentials
    config: CodexConfig,
}

impl CodexBridge {
    pub fn new(config: CodexConfig, auth_path: PathBuf) -> Result<Self>;

    // Auth flow
    pub async fn login(&self, callbacks: LoginCallbacks) -> Result<()>;
    pub async fn logout(&self) -> Result<()>;
    pub fn is_authenticated(&self) -> bool;
    pub fn account_id(&self) -> Option<String>;
    pub fn email(&self) -> Option<String>;

    // Inference
    pub async fn execute(&self, opts: CodexOpts) -> Result<CodexResult>;
}

pub struct CodexOpts {
    pub instructions: String,                    // system prompt; required by backend
    pub messages: Vec<Message>,                  // conversation in Responses API format
    pub model: String,                           // default "gpt-5.3-codex" or "gpt-5.5"
    pub reasoning_effort: ReasoningEffort,       // low/medium/high/xhigh
    pub schema: Option<JsonSchema>,              // forces structured output via Responses API output_schema
    pub session_id: Option<String>,              // for prompt caching; derive from track id
    pub max_output_tokens: Option<u32>,
    pub timeout: Duration,
    pub on_event: Box<dyn Fn(CodexEvent) + Send + Sync>,
}

pub struct CodexResult {
    pub ok: bool,
    pub jsonl_path: PathBuf,
    pub final_message: serde_json::Value,        // schema-validated when schema is set
    pub events: Vec<CodexEvent>,
    pub proposed_commits: Vec<ProposedCommit>,
    pub schema_violations: Vec<String>,
    pub usage: Usage,                            // input/output/cached tokens
}

pub struct LoginCallbacks {
    pub on_auth_url: Box<dyn Fn(&str) + Send + Sync>,           // present URL to user
    pub on_manual_code: Option<Box<dyn Fn() -> BoxFuture<'static, String> + Send + Sync>>,  // for headless fallback
    pub on_progress: Box<dyn Fn(&str) + Send + Sync>,
}
```

## 4. OAuth flow (constants + sequence)

### Constants

| Name | Value |
|---|---|
| `CLIENT_ID` | `app_EMoamEEZ73f0CkXaXp7hrann` (public; same as Pi/Codex CLI/CodexBar) |
| `AUTHORIZE_URL` | `https://auth.openai.com/oauth/authorize` |
| `TOKEN_URL` | `https://auth.openai.com/oauth/token` |
| `REDIRECT_URI` | `http://localhost:1455/auth/callback` (OpenAI-whitelisted) |
| `SCOPE` | `openid profile email offline_access` |
| Account-ID JWT claim path | `https://api.openai.com/auth.chatgpt_account_id` (in `access_token`) |

### Authorize URL params

```
?response_type=code
&client_id=app_EMoamEEZ73f0CkXaXp7hrann
&redirect_uri=http%3A%2F%2Flocalhost%3A1455%2Fauth%2Fcallback
&scope=openid%20profile%20email%20offline_access
&code_challenge=<S256(verifier)>
&code_challenge_method=S256
&state=<random-32-bytes>
&id_token_add_organizations=true
&codex_cli_simplified_flow=true
&originator=workflow-app
```

### Sequence

1. Generate PKCE: 32-byte verifier (URL-safe base64), `code_challenge = base64(sha256(verifier))`.
2. Generate `state` (32 bytes random).
3. Build authorize URL.
4. Spawn local HTTP server on `localhost:1455` listening for `/auth/callback`.
5. Open browser to authorize URL via `open` crate.
6. **Race two paths:**
   - Browser callback hits `/auth/callback?code=...&state=...`.
   - User pastes the redirect URL manually (for headless/SSH).
   The first to complete wins; cancel the other.
7. Validate `state` matches.
8. POST to `TOKEN_URL` with `grant_type=authorization_code`, `client_id`, `code`, `code_verifier=<verifier>`, `redirect_uri`.
9. Response: `access_token`, `refresh_token`, `id_token`, `expires_in`.
10. Decode `access_token` JWT (without verifying — we trust OpenAI's TLS); extract `chatgpt_account_id` from claim path.
11. Persist credentials to `~/.kit-workflow-app/auth.json` (chmod 0600).

### Token refresh

```
POST https://auth.openai.com/oauth/token
Content-Type: application/x-www-form-urlencoded
Body: grant_type=refresh_token&client_id=<CLIENT_ID>&refresh_token=<refresh>
```

Refresh proactively when `expires_at - now < 1 hour`. Hard refresh after 8 days regardless.

## 5. Codex Responses API request

```
POST https://chatgpt.com/backend-api/codex/responses
Authorization: Bearer <access_token>
ChatGPT-Account-Id: <account_id>
Content-Type: application/json
Accept: text/event-stream

{
  "model": "gpt-5.3-codex",
  "instructions": "<our system prompt>",
  "input": [
    { "type": "message", "role": "user", "content": [ ... ] },
    ...
  ],
  "stream": true,
  "store": false,
  "reasoning": { "effort": "medium" },
  "prompt_cache_key": "<session_id>",
  "max_output_tokens": <opts.max_output_tokens>,
  "output_schema": <opts.schema>     // when set
}
```

**Mandatory invariants enforced by the backend:**
- `instructions` MUST be present (system prompt).
- `store` MUST be `false` (stateless mode; ChatGPT backend rejects `true`).
- `stream` MUST be `true` (backend always streams via SSE).
- Message IDs (e.g. `id` fields on items) MUST be stripped from prior messages.
- `reasoning.encrypted_content` from prior assistant turns MUST be included for context continuity (per Codex CLI's behaviour).

Response is SSE; events are parsed into the `CodexEvent` enum.

## 6. Event shape (mirrors claude-bridge for skill-runner consistency)

```rust
pub enum CodexEvent {
    StreamStarted { response_id: String },
    OutputItemAdded { item_type: String, item_id: String },
    OutputTextDelta { item_id: String, delta: String },
    ReasoningSummaryDelta { item_id: String, delta: String },
    FunctionCall { item_id: String, name: String, arguments: serde_json::Value, call_id: String },
    StreamCompleted { usage: Usage },
    StreamFailed { error: String },
    UsageLimitWarning { remaining: u32 },        // backend sometimes returns these
}
```

## 7. Business rules

- **Direct integration only.** No `codex` CLI subprocess. No `codex-codes` Rust crate (that wraps the CLI). We implement the Responses API client ourselves.
- **`store: false` is non-negotiable.** Backend rejects `true` for ChatGPT auth.
- **`stream: true` is non-negotiable.**
- **`instructions` is non-negotiable.** Caller (skill-runner) provides our system prompt; bridge passes it through.
- **Per-session prompt caching.** When `opts.session_id` is set, pass as `prompt_cache_key`. skill-runner derives session_id from `track_id`.
- **Schema enforcement** via `output_schema` when set (Codex Responses API supports JSON Schema natively, mirrors `--output-schema` CLI flag).
- **429 quota errors.** Backend returns 429 when subscription quota exhausts. Bridge surfaces `UsageLimitWarning` events; skill-runner can render to UI.
- **JWT decoded without signature verification.** We trust OpenAI's TLS to bind the token to its source; we only need claims for `chatgpt_account_id`. Use `jsonwebtoken` crate's `decode_unverified` or hand-decode with `base64` + `serde_json`.
- **Auth file at `~/.kit-workflow-app/auth.json`**, chmod 0600. Format mirrors Pi's:

```json
{
  "openai-codex": {
    "type": "oauth",
    "access": "eyJ...",
    "refresh": "...",
    "id_token": "eyJ...",
    "expires": 1747353600000,
    "account_id": "acct_...",
    "email": "user@example.com"
  }
}
```

## 8. Integration points

| Module | Relationship |
|---|---|
| skill-runner | Caller |
| track-engine | Indirect via DispatchFn |
| claude-bridge | Sibling (asymmetric: Claude=subprocess, Codex=direct OAuth) |
| gui-shell | Onboarding triggers `login()`; status indicator polls `is_authenticated()` |
| `kit-scrub` | Read-path scrubbing on `.jsonl` |

## 9. Acceptance criteria

- [ ] `cargo test --package kit-codex-bridge` — unit tests for OAuth URL construction, PKCE, JWT account_id extraction, request body shape, SSE parsing, schema validation, store/stream/instructions enforcement.
- [ ] Real OAuth flow gated on a manual integration test: open browser, complete login, assert auth.json populated correctly.
- [ ] Real Codex inference call gated on valid auth: round-trip a small prompt; assert events stream; assert final message present.
- [ ] Schema-violation test: feed bad-JSON-producing prompt; assert violations.
- [ ] 429 handling test: mock backend; assert UsageLimitWarning surfaces; bridge doesn't crash.
- [ ] Token refresh: mock expired token; assert proactive refresh before expiry; assert refreshed credentials persisted.
- [ ] Manual-code-paste fallback works (for headless/SSH).
- [ ] Auth file is chmod 0600 on Linux/macOS.

## 10. Out of scope

- Codex CLI integration. Permanently out — direct integration only.
- API key fallback (Console PAYG). v1 supports OAuth only; API key path could be added in v1.1.
- Multi-account rotation (some plugins do this; we don't in v1).
- WebSocket transport (Pi supports this for prompt caching; v1 SSE-only).
- Custom `originator` per project (we use `workflow-app` everywhere).

## 11. Open questions

- [ ] **Default model.** `gpt-5.3-codex` or `gpt-5.5`? Pi defaults to `gpt-5-mini`; we should pick a Codex-flagship.
- [ ] **`reasoning.encrypted_content` continuity.** Codex CLI threads this through across turns. Do we replicate, or rely on stateless prompts each time? v1: stateless; v1.1 if compaction becomes painful.
- [ ] **Quota detection.** When 429 fires, do we abort the run or continue with a warning?
- [ ] **OAuth `originator` value.** "workflow-app" is fine, but Pi uses "pi" and OpenAI may track originators. Confirm acceptable.
