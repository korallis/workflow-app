---
name: project-deploy
description: "Deploy the current project to a preview or production environment. Runs pre-deploy checks, deploys (with Vercel integration if available), and verifies the deployment with browser automation. Use when someone says 'deploy', 'ship it', 'push to production', 'create a preview', or 'let me see it live'."
---

# Project Deploy

I'll perform a complete deployment workflow: pre-deploy checks, environment setup, deployment execution, post-deploy verification, and a summary report.

## Pre-Deployment Checks

### 1. Test Suite Validation

I'll run the full test suite to ensure code quality before deploy:
- Read MASTER_BLUEPRINT.md for the configured test command
- Execute tests with: `[test-command]`
- Report pass/fail count
- Block deployment if tests fail (unless explicitly overridden)

### 2. Source Control Status

I'll check for uncommitted changes:
- Run `git status`
- Warn if any untracked or modified files exist
- Request confirmation before deploying with dirty state

### 3. Environment Validation

I'll verify deployment readiness:
- Check for `.env` or `.env.local` files (flag if missing)
- Verify required environment variables are set for deployment target
- Warn about any secrets that might be exposed or missing
- Check deployment configuration (vercel.json, netlify.toml, etc.)

### 4. Code Quality Checks

I'll validate code quality:
- Run type checker (tsc, mypy, etc.) if configured
- Run linter (eslint, ruff, etc.) if configured
- Report any blocking issues

If pre-deploy checks fail, I'll stop and provide remediation steps.

## Deployment Execution

### For Vercel Projects

If MASTER_BLUEPRINT.md indicates Vercel deployment:

1. **Trigger deployment:**
   - Use `deploy_to_vercel` MCP tool to initiate deployment
   - Note deployment ID and URL

2. **Monitor deployment:**
   - Use `get_deployment` to check status
   - Poll for completion (typically 2-5 minutes)
   - Use `get_deployment_build_logs` to monitor build progress

3. **Handle build failures:**
   - If build fails, read error logs with `get_deployment_build_logs`
   - Analyze and present the error with:
     - Root cause (missing dependency, build config error, etc.)
     - File and line number
     - Suggested fix
   - Recommend remediation and offer to fix or retry

### For Other Deployment Targets

If non-Vercel deployment (specified in MASTER_BLUEPRINT.md):

1. Execute the configured deploy command
2. Capture and present output
3. Check exit code — block if non-zero
4. Present build logs if deployment fails

## Post-Deployment Verification

### 1. URL Verification

I'll confirm the deployment is accessible:
- Use `browser_navigate` to visit the deployed URL
- Verify HTTP 200 response (not 5xx, not redirects to error pages)
- Check page loads without errors

### 2. Smoke Testing

I'll test core functionality:
- `browser_snapshot` to verify page structure/accessibility tree
- `browser_take_screenshot` for visual baseline
- Navigate to key routes (home, about, main feature pages)
- Test interactive elements (forms, buttons, navigation)
- Verify responsive design (test mobile viewport if applicable)

### 3. Runtime Validation

I'll check for runtime errors:
- Use `browser_console_messages` with pattern `"error|exception"` to catch errors
- Verify critical features are accessible and functional
- Test critical user flows:
  - Sign up / login (if applicable)
  - Main feature workflows
  - Data submission (if applicable)

### 4. Performance Check

I'll verify basic performance:
- Check page load time (report if >3s for initial page)
- Verify images load correctly
- Check for failed resource requests with `browser_network_requests`

## Failure Handling

If any verification fails:

1. **Document the issue:**
   - Page/route where failure occurred
   - Error message or assertion that failed
   - Screenshot of the failure state

2. **Diagnosis:**
   - Check logs (`get_deployment_build_logs` or `browser_console_messages`)
   - Identify root cause

3. **Options:**
   - Rollback deployment (if previous version available)
   - Fix the issue locally, commit, and redeploy
   - Present issue details for manual investigation

## Post-Deploy Summary

I'll update project state:
- Note deployment URL and environment (preview/production)
- Record deployment timestamp
- Update LEARNINGS.md with deployment metrics
- Update module statuses if this was a release milestone
- Flag any issues found and their status

## Deployment Report

```
DEPLOYMENT REPORT — [date]
================================

TARGET:
  Environment:  [preview / production]
  URL:          [deployment-url]
  Deployment ID: [id]

PRE-DEPLOY CHECKS:
  Tests:           ✅ 42/42 passed
  Git status:      ✅ Clean (0 uncommitted changes)
  Type checking:   ✅ No errors
  Linting:         ✅ No errors
  Environment:     ✅ All required vars set

BUILD:
  Status:          ✅ Success (built in 2m 15s)
  Build logs:      ✅ No warnings

VERIFICATION:
  URL access:      ✅ 200 OK
  Page load:       ✅ 1.2s (home page)
  Navigation:      ✅ 5/5 routes working
  Forms:           ✅ Submission tested
  Console errors:  ❌ 1 error on /dashboard

ISSUES FOUND:
  1. TypeError on dashboard loading (non-critical)

NEXT STEPS:
  - Fix console error and redeploy, OR
  - Leave as-is (error doesn't block functionality), OR
  - Create follow-up task for dashboard fix

ROLLBACK:
  Previous version: [version-id]
  To rollback: [instructions]
```

## Key Commands

- `/project-test` — Run comprehensive test suite before deploying
- `/project-status` — Check overall project readiness
- `/project-module [name]` — Review specific module before deploying it
