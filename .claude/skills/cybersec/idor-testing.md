---
skill_name: idor-testing
description: Test for Insecure Direct Object Reference vulnerabilities in Méthode Aristote
category: cybersec
priority: critical
tags: [security, idor, permissions, authorization]
---

# IDOR Testing - Méthode Aristote

**PRIORITY #1 Security Risk**

## Context

Méthode Aristote has a complex 3-layer permission system:
- **Matrix Layer**: Role-based permissions (13 resources × 4 actions)
- **Hierarchy Layer**: Organizational relationships (tutors → students, parents → children)
- **Ownership Layer**: Resource ownership checks

**Critical Risk**: Tutors/parents accessing sessions/students from other users.

## Attack Surface

### High-Risk Endpoints
- `sessionRouter.getById` - Access any session by ID?
- `sessionRouter.list` - Filter bypass to see other users' sessions?
- `studentRouter.getById` - Access other tutors' students?
- `activityRouter.getById` - Access other sessions' activities?
- `workplanRouter.getById` - Access other students' workplans?

### Test Scenarios

#### 1. Cross-Tenant Access (Tutor → Tutor)
```typescript
// User A (TUTOR) ID: user_123
// User B (TUTOR) ID: user_456
// Session owned by B: session_789

// Test: Can A access session_789?
await trpc.session.getById.query({ id: "session_789" });
// Expected: Forbidden (403)
// Actual: ???
```

#### 2. Role Escalation (Parent → Student Data)
```typescript
// Parent ID: user_parent
// Child ID: student_child
// Other student: student_other

// Test: Can parent access other student's data?
await trpc.student.getById.query({ id: "student_other" });
// Expected: Forbidden (403)
// Actual: ???
```

#### 3. Sequential ID Enumeration
```typescript
// If IDs are predictable (cuid/uuid = safer, but test anyway)
for (let i = 1; i <= 1000; i++) {
  await trpc.session.getById.query({ id: `session_${i}` });
}
// Expected: Only owned sessions return data
// Actual: ???
```

#### 4. Filter Bypass
```typescript
// Test: Can filtering bypass ownership checks?
await trpc.session.list.query({
  where: { tutorId: "user_other_tutor" }
});
// Expected: Empty or forbidden
// Actual: ???
```

## Testing Protocol

### Step 1: Map Permission Matrix
```bash
# Read current permission implementation
grep -r "enforcePermission" src/server/api/services/
```

**Verify:**
- [ ] All service methods call `enforcePermission`
- [ ] Repository layer does NOT enforce permissions (single responsibility)
- [ ] Router layer does NOT bypass service layer

### Step 2: Test Each Resource
For each of 13 resources (SESSION, STUDENT, ACTIVITY, etc.):

```typescript
// Create test users with different roles
const tutor1 = createTestUser("TUTOR");
const tutor2 = createTestUser("TUTOR");
const parent1 = createTestUser("PARENT");
const student1 = createTestUser("STUDENT");

// Create resource owned by tutor1
const session = await createSession({ tutorId: tutor1.id });

// Test unauthorized access
await expectForbidden(() =>
  trpc.session.getById.query({ id: session.id }, { user: tutor2 })
);
```

### Step 3: Automated IDOR Scanner
```bash
# Run automated IDOR tests
pnpm test:security:idor
```

## Exploitation Examples

### Vulnerable Code Pattern
```typescript
// ❌ VULNERABLE - No permission check
export const sessionRouter = createTRPCRouter({
  getById: protectedProcedure
    .input(z.object({ id: z.string() }))
    .query(({ ctx, input }) => {
      // IDOR: Any authenticated user can access any session
      return ctx.db.session.findUnique({ where: { id: input.id } });
    }),
});
```

### Secure Code Pattern
```typescript
// ✅ SECURE - Permission enforced in service layer
export const sessionRouter = createTRPCRouter({
  getById: protectedProcedure
    .input(z.object({ id: z.string() }))
    .query(({ ctx, input }) => {
      return sessionService.getById(ctx, input.id);
    }),
});

export const sessionService = {
  getById: async (ctx: ProtectedContext, id: string) => {
    await enforcePermission(ctx, "SESSION", "READ");
    const session = await sessionRepository.findOne(ctx.db, id);

    // Ownership check
    if (session.tutorId !== ctx.session.userId) {
      throw new ForbiddenError("Cannot access other tutors' sessions");
    }

    return session;
  },
};
```

## Detection Methods

### Code Review Checklist
- [ ] Every `protectedProcedure` calls a service method
- [ ] Services call `enforcePermission` before data access
- [ ] Ownership checks verify `userId` matches resource owner
- [ ] List endpoints filter by ownership by default
- [ ] No direct `ctx.db` calls in routers

### Runtime Testing
```bash
# Test with different user contexts
IDOR_TEST_USER_A=user_123 IDOR_TEST_USER_B=user_456 pnpm test:e2e:idor
```

### Burp Suite Proxy
1. Configure Burp to intercept requests
2. Capture legitimate session access
3. Change session ID in request
4. Forward → should get 403 Forbidden

## Remediation Checklist

- [ ] Audit all 13 resources × 4 actions = 52 operations
- [ ] Add integration tests for cross-user access
- [ ] Implement automated IDOR testing in CI/CD
- [ ] Document ownership rules in `doc/guides/tech/security-ownership.md`
- [ ] Add Sentry alerts for repeated 403 errors (potential attack)

## Files to Review

**Critical:**
- `src/server/api/routers/*.ts` - All routers
- `src/server/api/services/*.ts` - Permission enforcement
- `src/server/api/repositories/*.ts` - Data access (should be dumb)
- `src/server/lib/permissions.ts` - Core permission logic

**Testing:**
- `tests/integration/permissions/*.test.ts`
- `tests/e2e/idor/*.spec.ts` (create if missing)

## Success Criteria

- [ ] Zero IDOR vulnerabilities found in automated scan
- [ ] 100% coverage of permission checks in services
- [ ] Integration tests for all cross-user scenarios
- [ ] Security documentation updated
- [ ] Team trained on IDOR risks
