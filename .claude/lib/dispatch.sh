#!/usr/bin/env bash
# .claude/lib/dispatch.sh — dual-harness Codex dispatcher
#
# Runs Codex CLI as a child process, tees output to a log under
# .kit-orchestration/, and (when an attached tmux session is reachable)
# splits a viewer pane that runs `tail -f` on the log so the user sees
# the run live in their existing terminal.
#
# Usage: dispatch.sh <phase> <id> <model> <effort> <prompt-file>
#   phase: validate | execute | review | <free-form-tag>
#   id:    stable identifier (PR number, module name, etc.) — kebab-case
#   model: Codex model (e.g. gpt-5.5)
#   effort: low | medium | high | xhigh   (passed via -c model_reasoning_effort=...)
#   prompt-file: path to a file whose contents are streamed to Codex via stdin
#
# Env knobs:
#   KIT_TMUX_SESSION         override which tmux session to split into
#   KIT_TMUX_SPLIT           'h' (default) or 'v'
#   KIT_NO_TMUX=1            force inline streaming (no tmux pane)
#   KIT_CODEX_TIMEOUT        hard timeout in seconds for the main exec (default 1800)
#   KIT_AUTH_PREFLIGHT_SECONDS  preflight timeout (default 15)
#   KIT_ALLOW_CONCURRENT=1   bypass single-flight lock
#   KIT_CODEX_SANDBOX        override sandbox mode (default workspace-write)
#   KIT_PARALLEL_TRACK       module name for per-track locks/logs/pane labels
#   KIT_PARALLEL_PORT        port exported to the executor for this track
#
# Portability: works on Linux and macOS. Requires either `timeout`
# (coreutils, default on Linux) or `gtimeout` (Homebrew coreutils on
# macOS). Lock is mkdir-based, no flock dependency.

set -euo pipefail

# --- self-relocation ---------------------------------------------------------
# Copy this script to a temp path and re-exec from there, so live edits to
# .claude/lib/dispatch.sh during the run (e.g. when Codex modifies the kit
# itself) don't cause bash to re-read garbled offsets at trap time. The
# relocated process owns the cleanup. Idempotent via env-var guard.
#
# REPO_ROOT must be captured BEFORE relocating, because after exec the
# relocated copy can't introspect its origin (BASH_SOURCE will point to /tmp).

if [[ "${KIT_DISPATCH_RELOCATED:-}" != "1" ]]; then
  _kit_origin_repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  _kit_self="$(mktemp -t kit-dispatch.XXXXXX)" || { echo "error: mktemp failed" >&2; exit 1; }
  if ! cp -- "${BASH_SOURCE[0]}" "$_kit_self"; then
    rm -f "$_kit_self"
    echo "error: failed to copy dispatch.sh to $_kit_self" >&2
    exit 1
  fi
  chmod +x "$_kit_self"
  export KIT_DISPATCH_RELOCATED=1
  export KIT_DISPATCH_TEMP="$_kit_self"
  export KIT_DISPATCH_REPO_ROOT="$_kit_origin_repo"
  exec bash "$_kit_self" "$@"
fi
trap 'rm -f "${KIT_DISPATCH_TEMP:-}"' EXIT

readonly REPO_ROOT="${KIT_DISPATCH_REPO_ROOT:?KIT_DISPATCH_REPO_ROOT must be set by the relocation block}"
readonly SCRIPT_DIR="$REPO_ROOT/.claude/lib"
readonly KIT_DIR="$REPO_ROOT/.kit-orchestration"
readonly PARALLEL_DIR="$REPO_ROOT/.claude/parallel"

die()  { printf '\033[31merror:\033[0m %s\n' "$*" >&2; exit 1; }
warn() { printf '\033[33mwarn:\033[0m %s\n'  "$*" >&2; }
info() { printf '\033[36minfo:\033[0m %s\n'  "$*" >&2; }

# --- portable timeout wrapper ------------------------------------------------

kit_timeout_cmd=""
if command -v timeout >/dev/null 2>&1; then
  kit_timeout_cmd="timeout"
elif command -v gtimeout >/dev/null 2>&1; then
  kit_timeout_cmd="gtimeout"
fi

kit_timeout() {
  # Usage: kit_timeout <seconds> <cmd> [args...]
  if [[ -z "$kit_timeout_cmd" ]]; then
    die "neither 'timeout' nor 'gtimeout' on PATH. Install GNU coreutils (Linux: apt/dnf install coreutils; macOS: brew install coreutils)."
  fi
  "$kit_timeout_cmd" "$@"
}

# --- arg parsing -------------------------------------------------------------

(( $# == 5 )) || die "usage: dispatch.sh <phase> <id> <model> <effort> <prompt-file>"
readonly PHASE="$1" ID="$2" MODEL="$3" EFFORT="$4" PROMPT_FILE="$5"

case "$PHASE" in
  validate|execute|review) ;;
  *) [[ "$PHASE" =~ ^[a-z][a-z0-9-]*$ ]] || die "phase must be kebab-case alphanumeric, got: $PHASE" ;;
esac
[[ "$ID" =~ ^[a-z0-9][a-z0-9-]*$ ]] || die "id must be kebab-case alphanumeric, got: $ID"
[[ "$EFFORT" =~ ^(low|medium|high|xhigh)$ ]] || die "effort must be low|medium|high|xhigh, got: $EFFORT"
[[ -f "$PROMPT_FILE" ]] || die "prompt file not found: $PROMPT_FILE"

readonly TRACK_ID="${KIT_PARALLEL_TRACK:-}"
if [[ -n "$TRACK_ID" ]]; then
  [[ "$TRACK_ID" =~ ^[a-z0-9][a-z0-9-]*$ ]] || die "KIT_PARALLEL_TRACK must be kebab-case alphanumeric, got: $TRACK_ID"
fi

# --- preflight: codex CLI presence + auth + model availability ---------------

command -v codex >/dev/null 2>&1 || die "codex CLI not found on PATH. Install with: npm install -g @openai/codex"

readonly KIT_CODEX_TIMEOUT="${KIT_CODEX_TIMEOUT:-1800}"
readonly KIT_CODEX_SANDBOX="${KIT_CODEX_SANDBOX:-workspace-write}"
readonly KIT_AUTH_PREFLIGHT_SECONDS="${KIT_AUTH_PREFLIGHT_SECONDS:-15}"

# Auth + model preflight. Uses the actual target model so we surface
# "model not available for this auth tier" before opening tmux panes.
auth_check_log="$(mktemp)"
trap 'rm -f "$auth_check_log" "${KIT_DISPATCH_TEMP:-}"' EXIT
if ! kit_timeout "$KIT_AUTH_PREFLIGHT_SECONDS" codex exec \
     -m "$MODEL" \
     --skip-git-repo-check \
     -s read-only \
     -c model_reasoning_effort=low \
     - <<<'echo dispatch.sh auth preflight' >"$auth_check_log" 2>&1; then
  warn "codex preflight failed (model=$MODEL, timeout=${KIT_AUTH_PREFLIGHT_SECONDS}s). Output:"
  sed 's/^/    /' "$auth_check_log" >&2
  die "Auth or model availability error. Try \`codex login\` (ChatGPT auth — required for gpt-5.5 access without API tier) or set OPENAI_API_KEY for a tier that supports model '$MODEL'. Increase KIT_AUTH_PREFLIGHT_SECONDS if the network is slow."
fi

# --- single-flight lock (mkdir-based, portable) ------------------------------

mkdir -p "$KIT_DIR"
if [[ -n "$TRACK_ID" ]]; then
  mkdir -p "$PARALLEL_DIR/locks"
  LOCK_DIR="$PARALLEL_DIR/locks/$TRACK_ID"
else
  LOCK_DIR="$KIT_DIR/.lock"
fi
readonly LOCK_DIR

if [[ -z "${KIT_ALLOW_CONCURRENT:-}" ]]; then
  if ! mkdir "$LOCK_DIR" 2>/dev/null; then
    die "another dispatch.sh run is in progress (lock dir at $LOCK_DIR). If a previous run died, remove it manually. Set KIT_ALLOW_CONCURRENT=1 to bypass."
  fi
  trap 'rm -rf "$LOCK_DIR"; rm -f "$auth_check_log" "${KIT_DISPATCH_TEMP:-}"' EXIT
elif [[ -n "$TRACK_ID" ]]; then
  info "KIT_ALLOW_CONCURRENT=1 set; using track-scoped lock path $LOCK_DIR without acquiring it"
fi

# --- log path (PID-suffixed to avoid 1s timestamp collisions) ----------------
# When the orchestrator (e.g. /project-execute) computes a $TS up-front and
# advertises paths derived from it, it must export KIT_DISPATCH_TS=$TS so we
# reuse the same token here. Without this, dispatch.sh forks a new TIMESTAMP
# and the advertised .jsonl/-report.json/.log paths become non-deterministic.

readonly TIMESTAMP="${KIT_DISPATCH_TS:-$(date +%Y%m%d-%H%M%S)-$$}"
if [[ -n "$TRACK_ID" ]]; then
  readonly TRACK_LOG_DIR="$KIT_DIR/tracks/${TIMESTAMP}-${TRACK_ID}"
  mkdir -p "$TRACK_LOG_DIR"
  readonly LOG_FILE="$TRACK_LOG_DIR/${TIMESTAMP}.log"
  readonly JSONL_FILE="$TRACK_LOG_DIR/${TIMESTAMP}.jsonl"
  readonly LAST_FILE="$TRACK_LOG_DIR/${TIMESTAMP}-last.md"
  readonly REPORT_FILE="$TRACK_LOG_DIR/${TIMESTAMP}-report.json"
  readonly SENTINEL_FILE="$TRACK_LOG_DIR/${TIMESTAMP}-${TRACK_ID}.done"
else
  readonly LOG_FILE="$KIT_DIR/${PHASE}-${ID}-${TIMESTAMP}.log"
  readonly JSONL_FILE="$KIT_DIR/${PHASE}-${ID}-${TIMESTAMP}.jsonl"
  readonly LAST_FILE="$KIT_DIR/${PHASE}-${ID}-${TIMESTAMP}-last.md"
  readonly REPORT_FILE="$KIT_DIR/${PHASE}-${ID}-${TIMESTAMP}-report.json"
  readonly SENTINEL_FILE=""
fi
touch "$LOG_FILE" "$JSONL_FILE"
readonly SCHEMA_FILE="$REPO_ROOT/.claude/skills/project-execute/codex-report-schema.json"
info "log: $LOG_FILE"
info "jsonl: $JSONL_FILE"
[[ -f "$SCHEMA_FILE" ]] && info "schema: $SCHEMA_FILE" || warn "no schema at $SCHEMA_FILE; running without --output-schema"

# --- tmux session resolution -------------------------------------------------

# NOTE: this resolves "the most-recent attached tmux client", which is
# usually the user's current terminal but is not guaranteed when multiple
# clients are attached to different sessions. Use KIT_TMUX_SESSION to pin.

resolve_tmux_session() {
  [[ -n "${KIT_NO_TMUX:-}" ]] && return 1
  command -v tmux >/dev/null 2>&1 || return 1

  if [[ -n "${KIT_TMUX_SESSION:-}" ]]; then
    if tmux has-session -t "$KIT_TMUX_SESSION" 2>/dev/null; then
      printf '%s' "$KIT_TMUX_SESSION"; return 0
    fi
    warn "KIT_TMUX_SESSION=$KIT_TMUX_SESSION set but session does not exist; falling back to detection"
  fi

  local sess
  sess=$(tmux list-clients -F '#{client_activity} #{client_session}' 2>/dev/null \
           | sort -rn | head -1 | awk '{print $2}')
  if [[ -n "$sess" ]]; then
    printf '%s' "$sess"; return 0
  fi

  # Detached sessions are intentionally ignored — we only split where a
  # human is watching. Set KIT_TMUX_SESSION to override.
  return 1
}

TMUX_SESSION=""
if tmux_session=$(resolve_tmux_session); then
  TMUX_SESSION="$tmux_session"
  default_split="h"
  [[ -n "$TRACK_ID" ]] && default_split="v"
  split_flag="-${KIT_TMUX_SPLIT:-$default_split}"
  info "splitting tmux session '$TMUX_SESSION' (override with KIT_TMUX_SESSION=name)"
  # The viewer pipes tail through sed that quits on a dispatcher-only sentinel
  # appended after codex finishes. We use __KIT_DISPATCH_EXIT__= rather than a
  # bare EXIT= so a Codex-generated line that happens to start with "EXIT=" (a
  # script, env example, error-code listing) doesn't close the pane early.
  # When sed exits, tail gets SIGPIPE, the pane shell finishes, and tmux
  # closes the pane automatically (remain-on-exit off by default).
  pane_title="kit-orchestration: $PHASE/$ID"
  [[ -n "$TRACK_ID" ]] && pane_title="[track:$TRACK_ID] $pane_title"
  if ! tmux split-window -t "$TMUX_SESSION" "$split_flag" \
        "echo '── $pane_title ──'; tail -f '$LOG_FILE' | sed -n '/^__KIT_DISPATCH_EXIT__=/{p;q;};p'" >/dev/null 2>&1; then
    msg="tmux split-window failed for session '$TMUX_SESSION'; viewing log inline."
    warn "$msg"
    echo "## dispatch.sh: $msg" >> "$LOG_FILE"
    TMUX_SESSION=""
  fi
fi
if [[ -z "$TMUX_SESSION" ]]; then
  info "no attached tmux client to split into — streaming inline (KIT_NO_TMUX=1 to silence)"
  echo "==== Codex output begins ($PHASE/$ID) ===="
fi
readonly TMUX_SESSION

# --- run codex (prompt via stdin to avoid ARG_MAX) ---------------------------
#
# Codex emits JSONL events on stdout (--json); we tee the raw stream to
# $JSONL_FILE for orchestrator post-run analysis, and pretty-print it
# through a jq formatter into $LOG_FILE for the human-watching tmux pane.
# When jq is missing, the formatter is a passthrough (`cat`), so the pane
# shows raw JSONL — readable, just less polished.
#
# --output-schema (when the schema file exists) forces the final agent
# message to conform to .claude/skills/project-execute/codex-report-schema.json,
# eliminating training-data bleed-through at the structural level.
# --output-last-message writes that final (schema-conformant) message to
# $REPORT_FILE for the orchestrator.

format_jsonl() {
  if ! command -v jq >/dev/null 2>&1; then
    cat
    return
  fi
  jq --unbuffered -R -r '
    . as $raw |
    (try (fromjson) catch null) as $j |
    if $j == null then
      $raw
    elif $j.type == "thread.started" then
      "── thread \($j.thread_id // "?") ──"
    elif $j.type == "session.created" then
      "── session \($j.session_id // "?") ──"
    elif $j.type == "turn.started" then
      "[turn] start"
    elif $j.type == "turn.completed" then
      "[turn] done  in=\($j.usage.input_tokens // "?")  out=\($j.usage.output_tokens // "?")  cached=\($j.usage.cached_input_tokens // "?")"
    elif $j.type == "turn.failed" then
      "[turn] FAILED: \($j.error.message // "?")"
    elif $j.type == "item.started" and $j.item.type == "command_execution" then
      "$ \($j.item.command // "")"
    elif $j.type == "item.completed" and $j.item.type == "command_execution" then
      "  ↳ exit \($j.item.exit_code // "?")  status=\($j.item.status // "?")"
    elif $j.type == "item.completed" and $j.item.type == "file_change" then
      "📝 \($j.item.status // "?")  \($j.item.kind // "")  \($j.item.path // "")"
    elif $j.type == "item.completed" and $j.item.type == "agent_message" then
      "\n--- agent message ---\n\($j.item.text // "")\n---"
    elif $j.type == "item.completed" and $j.item.type == "reasoning" then
      "💭 \(($j.item.text // $j.item.summary // "") | gsub("\n"; " ") | .[0:200])"
    elif $j.type == "item.completed" and $j.item.type == "mcp_tool_call" then
      "[mcp] \($j.item.server // "?")::\($j.item.tool // "?")  status=\($j.item.status // "?")"
    elif $j.type == "item.completed" and $j.item.type == "web_search" then
      "🔎 \($j.item.query // "?")"
    elif $j.type == "item.completed" and $j.item.type == "todo_list" then
      "📋 todos updated"
    elif $j.type == "error" then
      "ERROR: \($j.error.message // $j.message // "?")"
    else
      empty
    end
  ' 2>/dev/null
}

codex_args=(
  -m "$MODEL"
  -c "model_reasoning_effort=$EFFORT"
  -s "$KIT_CODEX_SANDBOX"
  --skip-git-repo-check
  -C "$REPO_ROOT"
  --json
)
# When a schema is present, the final message is JSON conforming to the
# schema and goes to REPORT_FILE. Otherwise the final message is free
# text and goes to LAST_FILE. -o is set exactly once.
if [[ -f "$SCHEMA_FILE" ]]; then
  codex_args+=( --output-schema "$SCHEMA_FILE" -o "$REPORT_FILE" )
else
  codex_args+=( -o "$LAST_FILE" )
fi

set +e
kit_timeout "$KIT_CODEX_TIMEOUT" codex exec "${codex_args[@]}" - <"$PROMPT_FILE" \
  | tee -a "$JSONL_FILE" \
  | format_jsonl \
  | tee -a "$LOG_FILE"
codex_rc=${PIPESTATUS[0]}
set -e

echo "__KIT_DISPATCH_EXIT__=$codex_rc" >> "$LOG_FILE"
if [[ -n "$SENTINEL_FILE" ]]; then
  printf '%s\n' "$codex_rc" > "$SENTINEL_FILE"
fi

if [[ -z "$TMUX_SESSION" ]]; then
  echo "==== Codex output ends (exit $codex_rc) ===="
fi

if (( codex_rc == 124 )); then
  die "codex exec timed out after ${KIT_CODEX_TIMEOUT}s (set KIT_CODEX_TIMEOUT=<seconds> to extend)"
fi

if (( codex_rc != 0 )); then
  die "codex exec failed with exit $codex_rc; see $LOG_FILE (raw events: $JSONL_FILE)"
fi

if [[ -f "$REPORT_FILE" ]]; then
  info "dispatch complete: schema-validated report at $REPORT_FILE"
elif [[ -f "$LAST_FILE" ]]; then
  info "dispatch complete: final message at $LAST_FILE"
else
  warn "dispatch complete but no -o output file was written"
fi
exit 0
