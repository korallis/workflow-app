# Master Architecture Blueprint

> **Status:** {{STATUS}} (Draft / Approved / In Progress)
> **Last Updated:** {{DATE}}
> **Project:** {{PROJECT_NAME}}
> **Team:** {{TEAM_MEMBERS}}

## 1. Project Overview

**Problem Statement:** {{PROBLEM_STATEMENT}}

**Solution Overview:** {{SOLUTION_OVERVIEW}}

**Key Success Criteria:**
- {{CRITERION_1}}
- {{CRITERION_2}}
- {{CRITERION_3}}

**Scope Boundaries:** {{SCOPE_BOUNDARIES}}

---

## 2. Tech Stack

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| Frontend Framework | {{FRONTEND_FRAMEWORK}} | {{VERSION}} | {{RATIONALE}} |
| Frontend Build & Tooling | {{FRONTEND_BUILD}} | {{VERSION}} | {{RATIONALE}} |
| Backend Runtime | {{BACKEND_RUNTIME}} | {{VERSION}} | {{RATIONALE}} |
| Backend Framework | {{BACKEND_FRAMEWORK}} | {{VERSION}} | {{RATIONALE}} |
| Database (Primary) | {{PRIMARY_DATABASE}} | {{VERSION}} | {{RATIONALE}} |
| Database (Cache/Sessions) | {{CACHE_DATABASE}} | {{VERSION}} | {{RATIONALE}} |
| Authentication | {{AUTH_SOLUTION}} | {{VERSION}} | {{RATIONALE}} |
| API Style | {{API_STYLE}} | — | {{RATIONALE}} |
| Deployment Platform | {{DEPLOYMENT_PLATFORM}} | — | {{RATIONALE}} |
| CI/CD | {{CI_CD_TOOL}} | — | {{RATIONALE}} |
| Monitoring & Logging | {{MONITORING_SOLUTION}} | — | {{RATIONALE}} |
| Testing Framework | {{TEST_FRAMEWORK}} | {{VERSION}} | {{RATIONALE}} |

**Version Strategy:** {{VERSION_STRATEGY}}

**Technology Constraints:** {{CONSTRAINTS}}

---

## 3. Data Model

### Core Entities

```typescript
// {{ENTITY_1}}
interface {{ENTITY_1}} {
  id: string;
  {{FIELD_1}}: {{TYPE}};
  {{FIELD_2}}: {{TYPE}};
  createdAt: Date;
  updatedAt: Date;
}

// {{ENTITY_2}}
interface {{ENTITY_2}} {
  id: string;
  {{FIELD_1}}: {{TYPE}};
  {{FIELD_2}}: {{TYPE}};
  createdAt: Date;
  updatedAt: Date;
}
```

### Relationships

{{ENTITY_RELATIONSHIP_DIAGRAM}}

**Example:** {{EXAMPLE_1}}

### Key Design Decisions

- **Primary Key Strategy:** {{PRIMARY_KEY_STRATEGY}}
- **Soft Deletes:** {{SOFT_DELETE_POLICY}}
- **Audit Trail:** {{AUDIT_POLICY}}
- **Multi-tenancy:** {{TENANCY_MODEL}} (single-tenant / multi-tenant / hybrid)
- **Scalability Considerations:** {{SCALABILITY_NOTES}}

---

## 4. API Design Patterns

### API Style
**Style:** {{API_STYLE}} (REST / GraphQL / tRPC / gRPC / Hybrid)

### Base URL & Versioning
```
Production: https://api.{{DOMAIN}}/v1
Staging: https://staging-api.{{DOMAIN}}/v1
```

### Authentication & Authorization
- **Method:** {{AUTH_METHOD}} (JWT / OAuth / Session / API Key / mTLS)
- **Token Lifetime:** {{TOKEN_LIFETIME}}
- **Scopes/Permissions Model:** {{PERMISSIONS_MODEL}}
- **Rate Limiting:** {{RATE_LIMIT_STRATEGY}}

### Naming Conventions
- **Endpoint Naming:** {{ENDPOINT_NAMING}} (e.g., `/api/v1/resources`, `/api/v1/resources/{id}/sub-resources`)
- **Field Naming:** {{FIELD_NAMING}} (camelCase / snake_case)
- **Error Field Names:** {{ERROR_NAMING}}

### Error Response Format
```json
{
  "error": {
    "code": "{{ERROR_CODE}}",
    "message": "{{ERROR_MESSAGE}}",
    "details": { "{{DETAIL_KEY}}": "{{DETAIL_VALUE}}" }
  }
}
```

**Standard Error Codes:** {{ERROR_CODES}}

### Response Format
```json
{
  "data": { "{{RESOURCE}}" },
  "meta": { "requestId": "uuid", "timestamp": "ISO8601" }
}
```

### Pagination (if applicable)
- **Style:** {{PAGINATION_STYLE}} (offset / cursor / keyset)
- **Default Limit:** {{DEFAULT_PAGE_SIZE}}
- **Max Limit:** {{MAX_PAGE_SIZE}}

---

## 5. Shared UI Patterns

### Design System
- **Color Palette:** {{COLOR_PALETTE}}
- **Typography:** {{TYPOGRAPHY_SPECS}}
- **Component Library:** {{COMPONENT_LIBRARY}} (custom / Material UI / shadcn/ui / other)
- **Icon Library:** {{ICON_LIBRARY}}

### Layout Patterns
- **Page Structure:** {{PAGE_STRUCTURE_PATTERN}}
- **Navigation:** {{NAV_PATTERN}} (sidebar / top nav / tabbed / etc.)
- **Responsive Breakpoints:** {{RESPONSIVE_BREAKPOINTS}}
- **Mobile-first:** {{MOBILE_FIRST_APPROACH}}

### Form Patterns
- **Validation Display:** {{VALIDATION_PATTERN}} (inline / summary / field-level)
- **Error Messaging:** {{ERROR_MESSAGE_PATTERN}}
- **Field Labeling:** {{FIELD_LABEL_PATTERN}}
- **Submission Behavior:** {{SUBMISSION_BEHAVIOR}}

### Loading & Skeleton States
- **Loading Indicator:** {{LOADING_INDICATOR_STYLE}}
- **Skeleton Components:** {{SKELETON_USAGE}}
- **Progressive Enhancement:** {{PROGRESSIVE_ENHANCEMENT}}

### Navigation & Routing
- **Route Structure:** {{ROUTE_STRUCTURE}}
- **Deep Linking:** {{DEEP_LINKING_SUPPORT}}
- **Breadcrumbs:** {{BREADCRUMB_USAGE}}
- **404/Error Pages:** {{ERROR_PAGE_PATTERN}}

### Accessibility (a11y)
- **WCAG Level:** {{WCAG_LEVEL}} (A / AA / AAA)
- **Keyboard Navigation:** {{KEYBOARD_NAV_REQUIRED}}
- **Screen Reader Testing:** {{SCREEN_READER_TOOLS}}

---

## 6. Modules

| Module | Priority | Description | Depends On | Estimated Effort |
|--------|----------|-------------|-----------|------------------|
| {{MODULE_1}} | P0/P1/P2 | {{DESCRIPTION}} | {{DEPENDS}} | {{EFFORT}} |
| {{MODULE_2}} | P0/P1/P2 | {{DESCRIPTION}} | {{DEPENDS}} | {{EFFORT}} |
| {{MODULE_3}} | P0/P1/P2 | {{DESCRIPTION}} | {{DEPENDS}} | {{EFFORT}} |
| {{MODULE_N}} | P0/P1/P2 | {{DESCRIPTION}} | {{DEPENDS}} | {{EFFORT}} |

**Module Ownership:** {{MODULE_OWNERSHIP_DETAILS}}

---

## 7. Infrastructure & Deployment

### Hosting & Infrastructure
- **Deployment Platform:** {{DEPLOYMENT_PLATFORM}}
- **Infrastructure as Code:** {{IAC_TOOL}} (Terraform / CloudFormation / ARM / CDK / other)
- **Container Strategy:** {{CONTAINER_STRATEGY}} (Docker / containerless / hybrid)
- **Database Hosting:** {{DATABASE_HOSTING}}
- **CDN & Static Assets:** {{CDN_SOLUTION}}

### Environment Strategy
- **Environments:** {{ENVIRONMENTS}} (dev / staging / production / preview)
- **Environment Parity:** {{ENVIRONMENT_PARITY_APPROACH}}
- **Secrets Management:** {{SECRETS_MANAGEMENT}}

### CI/CD Pipeline
```
Trigger → Build → Test → Deploy Staging → Integration Tests → Deploy Production
```

- **Tool:** {{CI_CD_TOOL}}
- **Branch Strategy:** {{BRANCH_STRATEGY}} (main / develop / feature branches)
- **Deployment Approvals:** {{DEPLOYMENT_APPROVALS}}
- **Rollback Strategy:** {{ROLLBACK_STRATEGY}}

### Monitoring, Logging & Observability
- **Logging:** {{LOGGING_SOLUTION}}
- **Metrics & APM:** {{METRICS_SOLUTION}}
- **Error Tracking:** {{ERROR_TRACKING_SOLUTION}}
- **Uptime Monitoring:** {{UPTIME_MONITORING}}
- **Alert Thresholds:** {{ALERT_THRESHOLDS}}

### Backup & Disaster Recovery
- **Backup Frequency:** {{BACKUP_FREQUENCY}}
- **Recovery Time Objective (RTO):** {{RTO}}
- **Recovery Point Objective (RPO):** {{RPO}}
- **Disaster Recovery Plan:** {{DR_PLAN}}

---

## 8. Security & Compliance

### Authentication & Authorization
- **User Authentication:** {{AUTH_MECHANISM}}
- **MFA Support:** {{MFA_REQUIRED}}
- **Session Management:** {{SESSION_MANAGEMENT}}
- **Password Policy:** {{PASSWORD_POLICY}}

### Data Security
- **Data at Rest:** {{DATA_AT_REST_ENCRYPTION}}
- **Data in Transit:** {{DATA_IN_TRANSIT_ENCRYPTION}} (TLS 1.2+)
- **Sensitive Data Handling:** {{SENSITIVE_DATA_HANDLING}}
- **PII Protection:** {{PII_PROTECTION}}

### Network & Infrastructure Security
- **Network Isolation:** {{NETWORK_ISOLATION}}
- **VPC/Firewall:** {{FIREWALL_CONFIG}}
- **DDoS Protection:** {{DDOS_PROTECTION}}
- **API Rate Limiting:** {{RATE_LIMITING}}

### Compliance Requirements
- **Regulations:** {{REGULATIONS}} (GDPR / HIPAA / SOC2 / PCI-DSS / other)
- **Data Residency:** {{DATA_RESIDENCY}}
- **Audit Logging:** {{AUDIT_LOGGING}}
- **Compliance Certifications:** {{CERTIFICATIONS}}

### Dependency & Vulnerability Management
- **Dependency Scanning:** {{DEPENDENCY_SCANNING_TOOL}}
- **Vulnerability Response Time:** {{VULN_RESPONSE_SLA}}
- **Security Updates Cadence:** {{SECURITY_UPDATE_CADENCE}}

---

## 9. Open Architectural Questions

- **Question 1:** {{QUESTION_1}}
  - Impact: {{IMPACT}}
  - Resolution: {{STATUS}}

- **Question 2:** {{QUESTION_2}}
  - Impact: {{IMPACT}}
  - Resolution: {{STATUS}}

- **Question N:** {{QUESTION_N}}
  - Impact: {{IMPACT}}
  - Resolution: {{STATUS}}

---

## Approval & Signoff

- **Created By:** {{CREATOR}}
- **Approved By:** {{APPROVER}}
- **Approval Date:** {{APPROVAL_DATE}}

**Changes Since Last Approval:**
{{CHANGE_LOG}}

---

## References & Resources

- [Technology Documentation Links]
- [Architecture Decision Records (ADRs)]
- [Related Spike Documentation]
- [Competitor/Reference Implementations]
