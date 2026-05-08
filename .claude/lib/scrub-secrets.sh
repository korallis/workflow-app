#!/usr/bin/env bash
# .claude/lib/scrub-secrets.sh — read-path secret redactor
#
# Reads stdin (or a file path arg), redacts common credential shapes,
# writes to stdout. Used by Claude when reading dispatch logs back into
# context, so secrets that Codex may have echoed don't get re-injected.
#
# Patterns covered (deliberately conservative — false positives ok,
# false negatives bad):
#   - Anthropic / OpenAI keys: sk-ant-..., sk-proj-..., sk-...
#   - GitHub PATs: ghp_..., gho_..., ghs_..., ghu_..., github_pat_...
#   - Slack tokens: xoxb-, xoxa-, xoxp-, xoxs-, xapp-
#   - Stripe secret keys: sk_live_..., rk_live_...
#   - JWTs: eyJ<base64>.<base64>.<base64>
#   - Bearer tokens: Bearer <opaque>
#   - Authorization: <opaque> headers (without Bearer scheme)
#   - Basic auth in URLs: https://user:password@...
#   - AWS access keys: AKIA[0-9A-Z]{16}
#
# Patterns deliberately NOT covered:
#   - Generic 32+ hex strings (would redact git SHAs, file hashes, etc.)
#   - Any heuristic for high-entropy short strings (false positive risk too high)
#
# Usage:
#   cat log | scrub-secrets.sh              # stdin → stdout
#   scrub-secrets.sh path/to/log            # file → stdout

set -euo pipefail

input() {
  if (( $# == 0 )); then cat
  else cat "$1"
  fi
}

input "$@" | sed -E '
  s/sk-ant-[A-Za-z0-9_-]{20,}/sk-ant-REDACTED/g
  s/sk-proj-[A-Za-z0-9_-]{20,}/sk-proj-REDACTED/g
  s/sk-[A-Za-z0-9_-]{32,}/sk-REDACTED/g
  s/sk_live_[A-Za-z0-9]{20,}/sk_live_REDACTED/g
  s/rk_live_[A-Za-z0-9]{20,}/rk_live_REDACTED/g
  s/ghp_[A-Za-z0-9]{30,}/ghp_REDACTED/g
  s/gho_[A-Za-z0-9]{30,}/gho_REDACTED/g
  s/ghs_[A-Za-z0-9]{30,}/ghs_REDACTED/g
  s/ghu_[A-Za-z0-9]{30,}/ghu_REDACTED/g
  s/github_pat_[A-Za-z0-9_]{40,}/github_pat_REDACTED/g
  s/xox[abprs]-[A-Za-z0-9-]{10,}/xox-REDACTED/g
  s/xapp-[0-9]+-[A-Za-z0-9-]{10,}/xapp-REDACTED/g
  s/eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/eyJ.REDACTED.REDACTED/g
  s/(Bearer[[:space:]]+)[A-Za-z0-9._~+/=-]{20,}/\1REDACTED/g
  s/([Aa]uthorization:[[:space:]]+)([A-Za-z][A-Za-z0-9_-]*[[:space:]]+)?[A-Za-z0-9._~+/=-]{20,}/\1\2REDACTED/g
  s|(https?://)[^:/[:space:]]+:[^@[:space:]]+@|\1REDACTED:REDACTED@|g
  s/\bAKIA[0-9A-Z]{16}\b/AKIAREDACTED/g
'
