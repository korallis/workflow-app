# CLAUDE Module Guide: {{MODULE_NAME}}

> **Purpose:** This guide helps Claude understand module conventions, architecture patterns, and boundaries specific to {{MODULE_NAME}}.
> **Last Updated:** {{DATE}}
> **Maintainer:** {{MAINTAINER}}

---

## 1. Patterns to Follow

### Architectural Patterns

**{{PATTERN_1_NAME}}**
- **Description:** {{PATTERN_1_DESCRIPTION}}
- **When to use:** {{WHEN_TO_USE}}
- **Example in this module:** {{EXAMPLE_LOCATION}}
  ```typescript
  {{CODE_EXAMPLE}}
  ```

**{{PATTERN_2_NAME}}**
- **Description:** {{PATTERN_2_DESCRIPTION}}
- **When to use:** {{WHEN_TO_USE}}
- **Example in this module:** {{EXAMPLE_LOCATION}}
  ```typescript
  {{CODE_EXAMPLE}}
  ```

### Design Patterns Applied

| Pattern | Usage | Rationale |
|---------|-------|-----------|
| {{PATTERN}} | {{WHERE_USED}} | {{WHY_CHOSEN}} |
| {{PATTERN}} | {{WHERE_USED}} | {{WHY_CHOSEN}} |

### Common Idioms in This Module

- **Error Handling:** {{ERROR_HANDLING_APPROACH}} (e.g., try-catch, Result type, error wrapper)
- **Async Operations:** {{ASYNC_PATTERN}} (e.g., Promises, async/await, observables)
- **Logging:** {{LOGGING_PATTERN}} (e.g., structured logging with context)
- **State Management:** {{STATE_MANAGEMENT}} (e.g., immutable updates, reactive)

---

## 2. Conventions in This Module

### File Organization

```
src/
├── {{module-name}}/
│   ├── index.ts                 # Main export file
│   ├── types.ts                 # Type definitions & interfaces
│   ├── constants.ts             # Module constants & enums
│   ├── services/                # Business logic
│   │   ├── {{domain}}.service.ts
│   │   └── {{other}}.service.ts
│   ├── repositories/            # Data access layer
│   │   ├── {{entity}}.repository.ts
│   │   └── {{other}}.repository.ts
│   ├── controllers/             # Request handlers (if applicable)
│   │   └── {{endpoint}}.controller.ts
│   ├── utils/                   # Helper functions
│   │   └── {{utility}}.ts
│   ├── hooks/                   # React hooks (if frontend)
│   │   └── use{{Hook}}.ts
│   ├── components/              # UI components (if frontend)
│   │   └── {{Component}}.tsx
│   ├── __tests__/               # Test files
│   │   ├── {{feature}}.test.ts
│   │   └── {{service}}.spec.ts
│   └── README.md                # Module overview
```

### Naming Conventions

| Entity | Convention | Example |
|--------|-----------|---------|
| Files | kebab-case | `user.service.ts`, `get-user.ts` |
| Classes | PascalCase | `UserService`, `OrderProcessor` |
| Functions | camelCase | `getUserById()`, `processPayment()` |
| Constants | UPPER_SNAKE_CASE | `MAX_RETRY_COUNT`, `DEFAULT_TIMEOUT` |
| Types/Interfaces | PascalCase | `User`, `CreateUserInput`, `ApiResponse<T>` |
| Enums | PascalCase | `OrderStatus`, `UserRole` |
| Private methods | _camelCase (prefix) or camelCase | `_validateInput()`, `validateInput()` |
| Directories | kebab-case | `services/`, `api-handlers/` |

### Code Style

**TypeScript Strictness:**
- Strict mode enabled: `"strict": true`
- No implicit any: `"noImplicitAny": true`
- Always type function parameters and returns
- Use const/let, never var
- Prefer interfaces for objects, types for unions/aliases

**Example:**
```typescript
// ✅ Good
interface UserInput {
  name: string;
  email: string;
}

function createUser(input: UserInput): Promise<User> {
  // Implementation
}

// ❌ Avoid
function createUser(input: any) {
  // Implementation
}
```

**Imports & Exports:**
- Use ES6 modules: `import` / `export`
- Named exports preferred over default exports (except for React components and main entry points)
- Organize imports: built-ins, external packages, internal modules, relative imports
  ```typescript
  import { join } from 'path';
  import { v4 as uuid } from 'uuid';
  import { UserService } from '@services/user.service';
  import { validateEmail } from './utils';
  ```

### Error Handling

**Error Wrapper Pattern:**
```typescript
interface Result<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: Record<string, any>;
  };
}

// Usage
async function getUser(id: string): Promise<Result<User>> {
  try {
    const user = await repository.findById(id);
    if (!user) {
      return {
        success: false,
        error: { code: 'NOT_FOUND', message: 'User not found' }
      };
    }
    return { success: true, data: user };
  } catch (err) {
    return {
      success: false,
      error: { code: 'INTERNAL_ERROR', message: 'Failed to fetch user' }
    };
  }
}
```

**Exception Handling (if used):**
- Create custom exception classes:
  ```typescript
  class NotFoundError extends Error {
    constructor(resource: string) {
      super(`${resource} not found`);
      this.name = 'NotFoundError';
    }
  }
  ```
- Use specific exception types, not generic Error
- Always log exceptions with context

### Logging

**Standard Logging Format:**
```typescript
import { logger } from '@utils/logger';

// Info
logger.info('User created', { userId: user.id, email: user.email });

// Warning
logger.warn('Retry attempt', { attempt: 2, maxRetries: 3 });

// Error
logger.error('Database query failed', { error, query: sql });

// Debug (development only)
logger.debug('Processing step', { step: 'validation', data: input });
```

**What to Log:**
- ✅ Operations with side effects (create, update, delete)
- ✅ External API calls (request/response summary)
- ✅ Authentication/authorization decisions
- ✅ Error conditions with context
- ❌ Passwords, tokens, secrets
- ❌ Large data objects (log keys only)

---

## 3. Module Boundaries

### What This Module Owns

**Data:**
- {{ENTITY_1}} (full ownership; defines schema, validation, lifecycle)
- {{ENTITY_2}} (full ownership)

**Services:**
- {{SERVICE_1}}: Manages {{RESPONSIBILITY}}
- {{SERVICE_2}}: Manages {{RESPONSIBILITY}}

**APIs / Exports:**
- Endpoint: `GET /api/v1/{{resource}}`
- Endpoint: `POST /api/v1/{{resource}}`
- Function: `export {{ {{FUNCTION}} }}`

### What This Module Reads From

| Module | Data/Service | Contract |
|--------|-------------|----------|
| {{MODULE_A}} | {{READS_WHAT}} | {{INTERFACE}} |
| {{MODULE_B}} | {{READS_WHAT}} | {{INTERFACE}} |
| {{EXTERNAL_SERVICE}} | {{API_ENDPOINT}} | {{RESPONSE_TYPE}} |

**Example:**
```typescript
// Reading from auth module
import { getCurrentUser } from '@modules/auth';

// Reading from database service
import { Database } from '@services/database';

// Making external API calls
const response = await fetch('https://external-service.com/api/data');
```

### What This Module Must NEVER Do

- ❌ {{VIOLATION_1}}: {{EXPLANATION}}
- ❌ {{VIOLATION_2}}: {{EXPLANATION}}
- ❌ {{VIOLATION_3}}: {{EXPLANATION}}
- ❌ **Access database tables from other modules directly** — Use their exported services instead
- ❌ **Modify other modules' data** — Request changes through their public APIs
- ❌ **Depend on implementation details** — Only depend on exported interfaces
- ❌ **Create circular imports** — Refactor shared logic into a separate utility module
- ❌ **Expose internal types** — Only export types meant for public use

**Example of Violation:**
```typescript
// ❌ NEVER do this:
import { db } from '@services/database';
const user = await db.query('SELECT * FROM users WHERE id = ?', [userId]);

// ✅ DO THIS instead:
import { UserRepository } from '@modules/user';
const user = await UserRepository.findById(userId);
```

---

## 4. Known Gotchas

### Performance Pitfalls

**{{GOTCHA_1_NAME}}**
- **Problem:** {{DESCRIPTION}}
- **Symptom:** {{HOW_TO_SPOT}}
- **Solution:** {{HOW_TO_FIX}}
- **Example:**
  ```typescript
  // ❌ Slow: N+1 query problem
  const users = await User.find();
  for (const user of users) {
    user.profile = await Profile.findOne({ userId: user.id });
  }

  // ✅ Fast: Load profiles in single query
  const users = await User.find().populate('profile');
  ```

**{{GOTCHA_2_NAME}}: Race Conditions**
- **Problem:** Concurrent updates may overwrite data
- **Solution:** Use atomic operations, version fields, or transactions
  ```typescript
  // ✅ Safe concurrent update
  const updated = await User.updateOne(
    { id, version: 1 },
    { $set: { name }, $inc: { version: 1 } }
  );
  if (updated.modifiedCount === 0) {
    throw new Error('Update conflict; retry');
  }
  ```

### Threading & Concurrency

- {{CONCURRENCY_ISSUE}}: {{EXPLANATION}}
- **Safe Patterns:** {{PATTERNS}}
- **Unsafe Patterns:** {{PATTERNS_TO_AVOID}}

### Common Mistakes

1. **{{MISTAKE_1}}**
   - ❌ Wrong: `const result = service.getData();` (blocking)
   - ✅ Right: `const result = await service.getData();` (async)

2. **{{MISTAKE_2}}**
   - ❌ Don't: Modify function parameters directly
   - ✅ Do: Create copies/clones and mutate those

3. **{{MISTAKE_3}}**
   - ❌ Avoid: Hardcoding configuration values
   - ✅ Use: Environment variables or config service

### Memory & Resource Leaks

- **{{LEAK_1}}:** {{DESCRIPTION}}
  - Mitigation: {{SOLUTION}}
- **{{LEAK_2}}:** {{DESCRIPTION}}
  - Mitigation: {{SOLUTION}}

### Debugging Tips

- Add structured logging at module entry/exit points
- Use {{DEBUGGING_TOOL}} for step-through debugging
- Check {{COMMON_LOG}} for errors specific to this module
- Profile with {{PROFILER}} if performance degrades
- Enable trace logging: `DEBUG={{MODULE_NAME}}:* npm start`

---

## 5. Test Patterns

### Unit Test Structure

```typescript
// {{feature}}.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { {{ServiceClass}} } from './{{service}}.service';

describe('{{ServiceClass}}', () => {
  let service: {{ServiceClass}};

  beforeEach(() => {
    service = new {{ServiceClass}}();
  });

  afterEach(() => {
    // Cleanup
  });

  describe('{{methodName}}', () => {
    it('should {{expected behavior}} when {{condition}}', async () => {
      // Arrange
      const input = { /* test data */ };

      // Act
      const result = await service.{{method}}(input);

      // Assert
      expect(result).toEqual({ /* expected result */ });
    });

    it('should throw {{error}} when {{condition}}', async () => {
      // Arrange
      const input = { /* invalid data */ };

      // Act & Assert
      await expect(service.{{method}}(input)).rejects.toThrow('{{error message}}');
    });
  });
});
```

### Mocking & Fixtures

**Mock {{DependentService}}:**
```typescript
import { vi } from 'vitest';

const mockUserRepository = {
  findById: vi.fn(),
  create: vi.fn(),
};

// In test:
mockUserRepository.findById.mockResolvedValue({ id: '1', name: 'Test User' });
```

**Test Fixtures:**
```typescript
export const fixtures = {
  validUser: { name: 'John', email: 'john@example.com' },
  invalidUser: { name: '', email: 'invalid' },
};
```

### Integration Tests

**Test {{Workflow}} end-to-end:**
```typescript
describe('{{Workflow}} Integration', () => {
  it('should complete full workflow', async () => {
    // Setup: Create user
    const user = await createUser(fixtures.validUser);

    // Action: Perform operation
    const result = await service.processUser(user.id);

    // Verify: Check state changes
    expect(result.status).toBe('completed');
    const updated = await getUser(user.id);
    expect(updated.status).toBe('completed');
  });
});
```

### Test Coverage Goals

- **Line Coverage:** > {{COVERAGE_TARGET}}%
- **Branch Coverage:** > {{BRANCH_COVERAGE}}%
- **Function Coverage:** 100% for critical paths

**Run coverage:**
```bash
npm test -- --coverage
```

### Test Data Management

- Use factories for complex test objects:
  ```typescript
  const user = userFactory.build({ name: 'Test' });
  ```
- Keep fixtures in separate `__fixtures__/` directory
- Clean up test data after each test (transaction rollback)
- Use separate test database for integration tests

---

## Implementation Checklist

When implementing features in this module, follow this checklist:

- [ ] Read the full `SPEC.md` and understand requirements
- [ ] Review `CLAUDE.md` (this file) for patterns and conventions
- [ ] Follow naming conventions from Section 2
- [ ] Implement with appropriate error handling (Section 2)
- [ ] Add structured logging (Section 2)
- [ ] Respect module boundaries (Section 3)
- [ ] Watch out for gotchas (Section 4)
- [ ] Write unit and integration tests (Section 5)
- [ ] Run linter and formatter: `npm run lint -- --fix`
- [ ] Achieve test coverage threshold: `npm test -- --coverage`
- [ ] Update type definitions if data model changes
- [ ] Document any new patterns or gotchas discovered
- [ ] Submit for code review with test coverage report

---

## Further Reading

- **Module Specification:** [SPEC.md](./SPEC.md)
- **Master Blueprint:** [../MASTER_BLUEPRINT.md](../MASTER_BLUEPRINT.md)
- **Related Modules:** {{RELATED_MODULES}}
- **Architecture Decision Records:** {{ADRS}}
- **External Documentation:** {{EXTERNAL_DOCS}}

---

## Questions & Discussions

If you encounter issues or questions while implementing this module:

1. Check this guide (Sections 2-5)
2. Check `SPEC.md` for requirements
3. Review recent commits for patterns used
4. Ask {{MAINTAINER}} or post in {{DISCUSSION_CHANNEL}}
5. Update this guide if you discover new patterns or gotchas
