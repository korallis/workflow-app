# Module Specification: {{MODULE_NAME}}

> **Status:** {{STATUS}} (Draft / Ready for Review / In Progress / Complete)
> **Last Updated:** {{DATE}}
> **Owner:** {{MODULE_OWNER}}
> **Version:** {{VERSION}}

---

## 1. Purpose

**Why does this module exist?**

{{MODULE_PURPOSE}}

**Business Value:** {{BUSINESS_VALUE}}

**Success Criteria:**
- {{SUCCESS_CRITERION_1}}
- {{SUCCESS_CRITERION_2}}
- {{SUCCESS_CRITERION_3}}

---

## 2. User Stories

| ID | As a | I want to | So that |
|----|----|----------|---------|
| {{US_1}} | {{USER_TYPE}} | {{ACTION}} | {{BENEFIT}} |
| {{US_2}} | {{USER_TYPE}} | {{ACTION}} | {{BENEFIT}} |
| {{US_3}} | {{USER_TYPE}} | {{ACTION}} | {{BENEFIT}} |
| {{US_N}} | {{USER_TYPE}} | {{ACTION}} | {{BENEFIT}} |

**Workflow Examples:**
- **{{WORKFLOW_1}}:** {{WORKFLOW_DESCRIPTION}}
- **{{WORKFLOW_2}}:** {{WORKFLOW_DESCRIPTION}}

---

## 3. Data Model

### Core Entities & Types

```typescript
// {{ENTITY_1}} - {{ENTITY_DESCRIPTION}}
interface {{ENTITY_1}} {
  id: string;
  {{FIELD_1}}: {{TYPE}};
  {{FIELD_2}}: {{TYPE}};
  {{FIELD_3}}?: {{TYPE}}; // optional
  createdAt: Date;
  updatedAt: Date;
}

// {{ENTITY_2}} - {{ENTITY_DESCRIPTION}}
interface {{ENTITY_2}} {
  id: string;
  {{FIELD_1}}: {{TYPE}};
  {{FIELD_2}}: {{TYPE}};
  {{FIELD_3}}: {{TYPE}};
  createdAt: Date;
  updatedAt: Date;
}

// {{ENUM_TYPE}} - {{ENUM_DESCRIPTION}}
enum {{ENUM_TYPE}} {
  {{VALUE_1}} = "{{VALUE_1}}",
  {{VALUE_2}} = "{{VALUE_2}}",
}

// {{DTO_TYPE}} - {{DTO_DESCRIPTION}}
interface {{DTO_TYPE}} {
  {{FIELD_1}}: {{TYPE}};
  {{FIELD_2}}: {{TYPE}};
}
```

### Relationships & Database Schema

```sql
-- {{TABLE_1}}
CREATE TABLE {{TABLE_1}} (
  id UUID PRIMARY KEY,
  {{COLUMN_1}} {{TYPE}} NOT NULL,
  {{COLUMN_2}} {{TYPE}},
  {{FOREIGN_KEY}} UUID REFERENCES {{PARENT_TABLE}}(id),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- {{TABLE_2}}
CREATE TABLE {{TABLE_2}} (
  id UUID PRIMARY KEY,
  {{COLUMN_1}} {{TYPE}} NOT NULL,
  {{FOREIGN_KEY}} UUID REFERENCES {{PARENT_TABLE}}(id),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Key Design Decisions

- **Soft Deletes:** {{SOFT_DELETE_YES_NO}} — {{RATIONALE}}
- **Timestamps:** {{TIMESTAMP_STRATEGY}}
- **Validation:** {{VALIDATION_LEVEL}} (client-only / server / both)
- **Data Constraints:** {{CONSTRAINTS_DESCRIPTION}}

---

## 4. API / Server Actions

### REST Endpoints (if applicable)

```http
GET /api/v1/{{resource-plural}}
  → List {{RESOURCE_NAME}} with pagination
  → Query Params: limit, offset, filter, sort
  → Response: {{ "data": [{{RESOURCE}}], "meta": { "total": number } }}
  → Status: 200 OK, 400 Bad Request

POST /api/v1/{{resource-plural}}
  → Create new {{RESOURCE_NAME}}
  → Body: {{ "field1": "value", "field2": "value" }}
  → Response: {{ "data": {{RESOURCE}} }}
  → Status: 201 Created, 400 Bad Request, 401 Unauthorized

GET /api/v1/{{resource-plural}}/{id}
  → Get single {{RESOURCE_NAME}}
  → Response: {{ "data": {{RESOURCE}} }}
  → Status: 200 OK, 404 Not Found

PATCH /api/v1/{{resource-plural}}/{id}
  → Update {{RESOURCE_NAME}}
  → Body: {{ "field1": "new_value" }}
  → Response: {{ "data": {{RESOURCE}} }}
  → Status: 200 OK, 400 Bad Request, 404 Not Found

DELETE /api/v1/{{resource-plural}}/{id}
  → Delete {{RESOURCE_NAME}}
  → Response: {{ "data": null, "meta": { "deleted": true } }}
  → Status: 200 OK, 404 Not Found
```

### Server Actions / RPC Endpoints (if applicable)

```typescript
// Create a new {{RESOURCE_NAME}}
export async function create{{RESOURCE_NAME}}(input: {{INPUT_TYPE}}): Promise<{{RESOURCE_NAME}}> {
  // Implementation
}

// Update existing {{RESOURCE_NAME}}
export async function update{{RESOURCE_NAME}}(
  id: string,
  input: {{UPDATE_INPUT_TYPE}}
): Promise<{{RESOURCE_NAME}}> {
  // Implementation
}

// Fetch {{RESOURCE_NAME}} by ID
export async function get{{RESOURCE_NAME}}(id: string): Promise<{{RESOURCE_NAME}} | null> {
  // Implementation
}

// List {{RESOURCE_PLURAL}}
export async function list{{RESOURCE_PLURAL}}(
  filter: {{FILTER_TYPE}},
  pagination: {{ limit: number; offset: number }}
): Promise<{{ items: {{RESOURCE_NAME}}[]; total: number }}> {
  // Implementation
}
```

### Authentication & Authorization

- **Required Scopes:** {{REQUIRED_SCOPES}}
- **Role-based Access:** {{ROLE_BASED_ACCESS}}
- **Resource-level Permissions:** {{RESOURCE_PERMISSIONS}}

---

## 5. UI Screens

### Screen 1: {{SCREEN_NAME}}

**Path:** `{{ROUTE}}`

**Purpose:** {{PURPOSE}}

**Key Elements:**
- {{ELEMENT_1}}: {{DESCRIPTION}}
- {{ELEMENT_2}}: {{DESCRIPTION}}
- {{ELEMENT_3}}: {{DESCRIPTION}}

**Wireframe/Mockup:**
```
┌─────────────────────────────────┐
│ Header / Navigation              │
├─────────────────────────────────┤
│                                 │
│ Main Content Area               │
│                                 │
├─────────────────────────────────┤
│ Footer / Actions                 │
└─────────────────────────────────┘
```

**User Interactions:**
- Click {{ELEMENT}} → {{ACTION}}
- Submit form → {{ACTION}}
- Navigate to {{ROUTE}} → {{ACTION}}

**States:**
- **Loading:** Display spinner; disable interactions
- **Empty:** Show placeholder message "{{MESSAGE}}"
- **Error:** Display error banner with retry option
- **Success:** Show confirmation toast; redirect to {{ROUTE}}

### Screen 2: {{SCREEN_NAME}}

**Path:** `{{ROUTE}}`

**Purpose:** {{PURPOSE}}

{{SCREEN_DETAILS}}

---

## 6. Business Logic & Rules

### Core Workflows

**Workflow: {{WORKFLOW_NAME}}**
1. User {{ACTION_1}}
2. System validates {{VALIDATION}}
3. System {{ACTION_2}}
4. System notifies user via {{NOTIFICATION_METHOD}}

**Workflow: {{WORKFLOW_NAME}}**
1. User {{ACTION_1}}
2. System checks {{CONDITION}}
3. If true: {{PATH_A}}
4. If false: {{PATH_B}}

### Validation Rules

| Field | Rule | Error Message |
|-------|------|--------------|
| {{FIELD_1}} | {{RULE}} | "{{ERROR_MSG}}" |
| {{FIELD_2}} | {{RULE}} | "{{ERROR_MSG}}" |
| {{FIELD_N}} | {{RULE}} | "{{ERROR_MSG}}" |

### Business Constraints

- **{{CONSTRAINT_1}}:** {{DESCRIPTION}}
- **{{CONSTRAINT_2}}:** {{DESCRIPTION}}
- **{{CONSTRAINT_N}}:** {{DESCRIPTION}}

### Side Effects & Triggers

- When {{EVENT}} occurs:
  - {{SIDE_EFFECT_1}}
  - {{SIDE_EFFECT_2}}
  - Notify {{SYSTEM}} via {{METHOD}}

---

## 7. Integration Points

| Component | Type | Contract | Notes |
|-----------|------|----------|-------|
| {{COMPONENT_1}} | {{READS/WRITES}} | {{DATA_TYPE}} | {{NOTES}} |
| {{COMPONENT_2}} | {{READS/WRITES}} | {{DATA_TYPE}} | {{NOTES}} |
| {{EXTERNAL_SERVICE}} | API Call | {{ENDPOINT}} | Async / Sync, retry policy |
| {{EVENT_SYSTEM}} | Event Stream | {{EVENT_TYPES}} | Publish / Subscribe |

### Dependencies

- **Imports From:** {{MODULE_A}}, {{MODULE_B}}
- **Exports To:** {{MODULE_C}}, {{MODULE_D}}
- **External Services:** {{SERVICE_1}} (v{{VERSION}}), {{SERVICE_2}}

### Contracts & Interfaces

```typescript
// Interface consumed from {{OTHER_MODULE}}
interface {{IMPORTED_INTERFACE}} {
  {{FIELD}}: {{TYPE}};
}

// Interface exported from this module
export interface {{EXPORTED_INTERFACE}} {
  {{FIELD}}: {{TYPE}};
}
```

---

## 8. Acceptance Criteria

- [ ] All user stories implemented and tested
- [ ] API endpoints functional with proper error handling
- [ ] Database migrations applied and rollback tested
- [ ] UI screens match approved mockups
- [ ] Form validation working client and server-side
- [ ] All integration points functional
- [ ] Unit test coverage > {{COVERAGE_TARGET}}%
- [ ] Integration tests passing
- [ ] Performance benchmarks met ({{METRIC}} < {{THRESHOLD}})
- [ ] Accessibility audit passing (WCAG {{LEVEL}})
- [ ] Documentation complete and reviewed
- [ ] Security review completed
- [ ] Deployed to staging and verified
- [ ] Ready for production release

---

## 9. Out of Scope

**Explicitly NOT included in this module:**
- {{OUT_OF_SCOPE_1}}
- {{OUT_OF_SCOPE_2}}
- {{OUT_OF_SCOPE_3}}

**Rationale:** {{RATIONALE_FOR_SCOPE_DECISIONS}}

**Future Enhancements:**
- {{FUTURE_1}}: Consider for v{{FUTURE_VERSION}}
- {{FUTURE_2}}: Depends on {{BLOCKING_FACTOR}}

---

## 10. Open Questions

| Question | Impact | Status | Resolution |
|----------|--------|--------|------------|
| {{QUESTION_1}} | {{HIGH/MEDIUM/LOW}} | {{PENDING/RESOLVED}} | {{RESOLUTION}} |
| {{QUESTION_2}} | {{HIGH/MEDIUM/LOW}} | {{PENDING/RESOLVED}} | {{RESOLUTION}} |
| {{QUESTION_N}} | {{HIGH/MEDIUM/LOW}} | {{PENDING/RESOLVED}} | {{RESOLUTION}} |

---

## Implementation Notes

**Estimated Effort:** {{ESTIMATE}} (story points / hours)

**Complexity Factors:**
- {{COMPLEXITY_1}}
- {{COMPLEXITY_2}}

**Risk Assessment:**
- {{RISK_1}}: {{MITIGATION}}
- {{RISK_2}}: {{MITIGATION}}

**Team Assignments:**
- Lead: {{LEAD_NAME}}
- Reviewers: {{REVIEWER_NAMES}}

---

## Sign-Off

- **Spec Created By:** {{CREATOR}}
- **Approved By:** {{APPROVER}}
- **Approval Date:** {{DATE}}

**Related Documents:**
- [Parent Blueprint](../MASTER_BLUEPRINT.md)
- [Module CLAUDE Guide](./CLAUDE.md)
- [Architecture Decision Records]
- [Related Spike Documents]

## Optional Parallel Track Companion

Create `parallel.yaml` beside this module spec when the module may run through
`/project-tracks`. The planner requires `version: 1` and refuses brownfield
modules without an explicit declaration.

```yaml
version: 1
touches:
  - src/{{module_slug}}/**
shared:
  - src/types/{{shared_type}}.ts
ports:
  - 3001
migrations: false
```
