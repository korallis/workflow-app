---
name: project-test
description: "Run a comprehensive test pass across the project — unit tests, type checking, linting, and visual verification with browser automation. Use when someone says 'run the tests', 'check everything works', 'test the project', or before deploying to verify quality."
---

# Project Test

I'll run a comprehensive test pass across all quality gates: unit tests, type checking, linting, and visual/integration verification.

## Test Infrastructure Discovery

I'll identify the test setup by reading:
- **MASTER_BLUEPRINT.md** — configured test command, type checker, and linter
- **package.json** (or equivalent) — available test scripts and dependencies
- **Module specs** — which modules have UI components vs API-only code
- **Test directories** — identify test file locations and coverage

This tells me:
- Test runner (jest, pytest, vitest, etc.)
- Type checker (tsc, mypy, etc.)
- Linter (eslint, ruff, etc.)
- Which modules have visual components to test

## Unit & Integration Tests

### 1. Run Test Suite

Execute the configured test command:
```
[test-command-from-MASTER_BLUEPRINT]
```

### 2. Parse Results

From test output, capture:
- Total tests run
- Pass count
- Fail count
- Skipped count
- Execution time

### 3. Report Failures

For each test failure, I'll extract:
- Test file and describe block
- Test name (what was being tested)
- Expected value
- Actual value
- Stack trace (first 5 lines)
- Suggested fix (based on error type)

Example output:
```
FAILED: src/modules/contacts/contact.spec.ts
  › Contact creation
    Expected: email validation
    Actual: accepted invalid email
    Fix: Check ContactForm.validateEmail() in contacts/form.tsx line 34
```

### 4. Calculate Coverage

If coverage reports are available:
- Report overall coverage percentage
- Flag modules/files with <80% coverage
- Suggest tests for uncovered lines

## Type Checking

### 1. Run Type Checker

Execute the configured type checker:
```
tsc --noEmit              (for TypeScript projects)
mypy src/                 (for Python projects)
```

### 2. Parse Type Errors

For each error, capture:
- File path
- Line number
- Column number
- Error message
- Context (the offending line)

Example:
```
TYPE ERROR: src/api/users.ts:24
  Property 'email' does not exist on type 'User'
  Line 24: const email = user.email;
           ^^^^^^^^^^
  Fix: User type should extend { email: string }
```

### 3. Report Summary

- Total type errors
- Errors by severity (error vs warning)
- Files with most errors
- Recommendation to fix before deploy

## Linting

### 1. Run Linter

Execute the configured linter:
```
eslint src/               (for JavaScript/TypeScript)
ruff check src/           (for Python)
```

### 2. Parse Violations

For each violation:
- File path
- Line number
- Rule name
- Message
- Auto-fix available? (yes/no)

### 3. Auto-Fix Where Possible

- Run linter with `--fix` flag to auto-correct violations
- Report what was fixed
- For remaining violations, suggest manual fixes

Example:
```
LINTING: src/components/Button.tsx
  ✅ Fixed 3 issues:
    - 2x unused imports (auto-removed)
    - 1x incorrect spacing (auto-fixed)
  ⚠️ 1 issue requires manual fix:
    - Line 15: Missing JSDoc for exported function
      Fix: Add comment: /** Button component with text */
```

## Visual Verification (UI Projects)

For projects with UI components, I'll use Playwright browser automation to verify functionality and appearance.

### 1. Start Development Server

If not running:
- Check if dev server is accessible (http://localhost:3000, etc.)
- If not, start it: `npm run dev` (or equivalent)
- Wait for server to be ready

### 2. Discover Routes

Identify all implemented routes by:
- Reading module specs
- Checking routing config (next.config.js, vite.config.ts, etc.)
- Looking at URL patterns in source code

Routes to test (example):
- `/` (home)
- `/modules/contacts` (contacts module)
- `/modules/deals` (deals module)
- etc.

### 3. Test Each Route

For each route, I'll:

**Navigation & Page Load:**
- `browser_navigate` to the route
- Verify page loads (HTTP 200, no redirect loops)
- Check page title/heading matches expected content

**Accessibility & Structure:**
- `browser_snapshot` to get accessibility tree
- Verify main content elements are present
- Check heading hierarchy (h1 → h2 → h3)
- Verify form labels, buttons, and interactive elements

**Visual Verification:**
- `browser_take_screenshot` for each page
- Verify layout is clean (no overlapping elements)
- Check responsive design (test mobile viewport: 375px width)
- Look for visual regressions vs baseline

**Interaction Testing:**
- Test form inputs: type text, verify input values
- Test buttons: click and verify actions
- Test navigation: click links and verify routing
- Test dropdowns/selects: expand and verify options

**Runtime Health:**
- `browser_console_messages` with pattern `"error|exception|Warning"` to catch:
  - JavaScript errors
  - Uncaught exceptions
  - Deprecation warnings
- Report each error with:
  - Error message
  - File/line where error occurred
  - Suggested fix

Example:
```
ROUTE VERIFICATION: /modules/contacts
  ✅ Page loads in 1.2s
  ✅ All expected elements present
  ✅ Forms submit correctly
  ✅ Navigation works
  ❌ Console error: TypeError: Cannot read property 'map' of undefined
     at ContactList.tsx:45
     Fix: Add null check: contacts?.map(...) or contacts && contacts.map(...)
```

### 4. Test Critical User Flows

For each major feature, test end-to-end flow:

**Example: Contact Creation Flow**
```
1. Navigate to /modules/contacts
2. Click "New Contact" button
3. Fill form: name, email, phone
4. Click "Save"
5. Verify contact appears in list
6. Verify confirmation toast shows
7. Verify no console errors
```

Report per-flow:
- Steps completed successfully
- Any failures or errors encountered
- Performance metrics (time to complete)

## Test Report Summary

```
TEST REPORT — [date]
================================

DISCOVERY:
  Test runner:    jest 29.5.0
  Type checker:   tsc 5.0.2
  Linter:         eslint 8.40.0
  Modules:        12 total (8 with UI)

UNIT TESTS:
  Total:          247 tests
  Passed:         ✅ 245 (99.2%)
  Failed:         ❌ 2
  Skipped:        ⏭️  0
  Duration:       2m 15s
  Coverage:       ✅ 84.3% (target: 80%)

  FAILURES:
    1. src/modules/deals/deal.spec.ts — Deal amount calculation
       Expected: $1,500.00
       Actual:   $1,50.00
       Fix: Check Decimal.ts line 42 — missing precision handling

    2. src/api/email.spec.ts — Email validation
       Expected: reject invalid domain
       Actual:   accepted "test@invalid"
       Fix: Update regex in validateEmail() or use email-validator package

TYPE CHECKING:
  Total errors:   ⚠️ 3
  Type errors:    src/components/Form.tsx:24
                  src/api/users.ts:18
                  src/modules/deals/calc.ts:5

  Recommendation: Fix before deploying

LINTING:
  Total violations: 12
  Auto-fixed:       ✅ 8
  Remaining:        ⚠️ 4

  Issues:
    - 2x unused imports (auto-fixed)
    - 2x missing JSDoc (manual fix needed)
    - Line 15 of Button.tsx: Add /** Button component */

VISUAL VERIFICATION (UI Routes):
  Routes tested:  ✅ 8/8
  Pages loaded:   ✅ 8/8 (avg 1.1s)
  Forms tested:   ✅ 6/6
  Console errors: ❌ 1 error on /modules/deals

  ERRORS FOUND:
    /modules/deals — TypeError: Cannot read property 'amount' of undefined
    at DealForm.tsx:67
    Fix: Add null check before accessing deal.amount

RESPONSIVE DESIGN:
  Desktop (1024px):  ✅ Verified
  Tablet (768px):    ⚠️ 1 overflow issue on /modules/contacts
  Mobile (375px):    ⚠️ 2 layout issues

OVERALL HEALTH:
  Status:         ⚠️ Ready with warnings
  Ready to deploy: No (fix 1 test + 1 type error + console error)
  Estimated time to fix: 30-45 minutes

RECOMMENDED ACTIONS:
  1. Fix Deal amount calculation test (unit test failure)
  2. Fix email validation type error
  3. Fix TypeError on /modules/deals page
  4. Fix mobile responsive issues (optional for v1)
  5. Add missing JSDoc comments (nice-to-have)

NEXT STEPS:
  /project-deploy          (after issues resolved)
  /project-module deals    (deep dive on deal issues)
```

## Key Commands

- `/project-status` — Check overall project status
- `/project-deploy` — Deploy after tests pass
- `/project-module [name]` — Deep dive on specific module with failures
- `/project-research [question]` — Research and document technical debt
