---
skill_name: broken-authentication
description: Test authentication and authorization vulnerabilities in Clerk + custom permissions system
category: cybersec
priority: critical
tags: [security, authentication, authorization, clerk, roles]
---

# Broken Authentication Testing - Méthode Aristote

**PRIORITY #2 Security Risk**

## Context

Dual authentication system:
- **Clerk**: OAuth, session management, user identity
- **Custom**: 7-role hierarchy + 3-layer permissions

**Complexity = Attack Surface:**
- Role hierarchy: SUPER_ADMIN → ADMIN → PEDAGOGIC_ASSISTANT → TUTOR_COACH → TUTOR → PARENT → STUDENT
- Edge cases: TUTOR_COACH (hybrid role), PARENT (multiple children), ADMIN (bypass permissions?)

## Attack Vectors

### 1. Role Escalation
```typescript
// Can TUTOR escalate to ADMIN?
// Test: Modify role in session/JWT
await fetch("/api/trpc/user.updateRole", {
  body: JSON.stringify({ userId: "self", role: "ADMIN" })
});
```

### 2. Session Hijacking
```typescript
// Can stolen Clerk session access protected resources?
// Test: Copy session token from User A to User B
const stolenToken = "clerk_session_xyz";
fetch("/api/trpc/session.list", {
  headers: { Cookie: `__session=${stolenToken}` }
});
```

### 3. Permission Bypass
```typescript
// Can PARENT bypass TUTOR-only endpoints?
// Test: Parent calling tutor-specific methods
await trpc.session.create.mutate({ /* tutor-only operation */ });
// Expected: 403 Forbidden
```

### 4. Hierarchy Bypass
```typescript
// Can TUTOR_COACH bypass ADMIN restrictions?
// Test: Mixed role privileges
const coach = { role: "TUTOR_COACH" };
await trpc.user.delete.mutate({ id: "some_user" });
// Expected: Forbidden unless explicitly allowed
```

## Testing Protocol

### Step 1: Role Matrix Audit

Map all 7 roles against 13 resources × 4 actions:

```bash
# Generate permission matrix
node scripts/audit-permissions.js

# Output example:
# STUDENT → SESSION:READ ✅
# STUDENT → SESSION:CREATE ❌
# TUTOR → SESSION:CREATE ✅
# PARENT → STUDENT:READ ✅ (own children only)
```

### Step 2: Authentication Flow Testing

#### Clerk Integration
```typescript
// Test: Clerk webhook signature validation
POST /api/webhooks/clerk
Headers: {
  "svix-id": "msg_xyz",
  "svix-timestamp": "1234567890",
  "svix-signature": "FORGED_SIGNATURE"
}
// Expected: 401 Unauthorized
```

#### Session Validation
```typescript
// Test: Expired session handling
const expiredSession = generateExpiredClerkSession();
await trpc.session.list.query({}, { session: expiredSession });
// Expected: 401 Unauthorized
```

### Step 3: Role-Based Access Control (RBAC)

#### Test Each Role
```typescript
const roles = ["STUDENT", "PARENT", "TUTOR", "TUTOR_COACH", "PEDAGOGIC_ASSISTANT", "ADMIN", "SUPER_ADMIN"];

for (const role of roles) {
  const user = createTestUser(role);

  // Test resource access
  const results = await testResourceAccess(user);

  // Verify against permission matrix
  expect(results).toMatchPermissionMatrix(role);
}
```

#### Edge Case: TUTOR_COACH
```typescript
// TUTOR_COACH = TUTOR + COACH privileges
const coach = createTestUser("TUTOR_COACH");

// Can create sessions (TUTOR privilege)?
await expect(trpc.session.create.mutate({ /*...*/ })).resolves.toBeDefined();

// Can view all tutors' sessions (COACH privilege)?
await expect(trpc.session.listAll.query()).resolves.toHaveLength(10);
```

#### Edge Case: PARENT with Multiple Children
```typescript
const parent = createTestUser("PARENT");
const child1 = createTestUser("STUDENT", { parentId: parent.id });
const child2 = createTestUser("STUDENT", { parentId: parent.id });
const otherChild = createTestUser("STUDENT");

// Can access child1 and child2
await expect(trpc.student.getById.query({ id: child1.id })).resolves.toBeDefined();
await expect(trpc.student.getById.query({ id: child2.id })).resolves.toBeDefined();

// Cannot access otherChild
await expect(trpc.student.getById.query({ id: otherChild.id })).rejects.toThrow("Forbidden");
```

### Step 4: JWT/Token Manipulation

```bash
# Decode Clerk session token
echo $CLERK_SESSION | base64 -d | jq

# Check for:
# - Weak signing algorithm (HS256 vs RS256)
# - Missing expiration (exp claim)
# - User role in token (can be tampered?)
```

## Vulnerability Patterns

### ❌ Vulnerable: Role in Client State
```typescript
// NEVER trust client-provided role
const UserProfile = () => {
  const [role, setRole] = useState("STUDENT"); // Client-controlled

  return (
    <AdminPanel visible={role === "ADMIN"} /> // Bypassable
  );
};
```

### ✅ Secure: Role from Server Session
```typescript
const UserProfile = () => {
  const { data: session } = trpc.auth.getSession.useQuery();

  return (
    <AdminPanel visible={session?.user.role === "ADMIN"} />
  );
};
```

### ❌ Vulnerable: Missing Permission Check
```typescript
export const sessionRouter = createTRPCRouter({
  delete: protectedProcedure
    .input(z.object({ id: z.string() }))
    .mutation(({ ctx, input }) => {
      // ANYONE authenticated can delete ANY session
      return ctx.db.session.delete({ where: { id: input.id } });
    }),
});
```

### ✅ Secure: Layered Permission Check
```typescript
export const sessionRouter = createTRPCRouter({
  delete: protectedProcedure
    .input(z.object({ id: z.string() }))
    .mutation(({ ctx, input }) => {
      return sessionService.delete(ctx, input.id);
    }),
});

export const sessionService = {
  delete: async (ctx: ProtectedContext, id: string) => {
    // Layer 1: Matrix permission
    await enforcePermission(ctx, "SESSION", "DELETE");

    // Layer 2: Ownership check
    const session = await sessionRepository.findOne(ctx.db, id);
    if (session.tutorId !== ctx.session.userId && ctx.session.user.role !== "ADMIN") {
      throw new ForbiddenError("Cannot delete other users' sessions");
    }

    return sessionRepository.deleteOne(ctx.db, id);
  },
};
```

## Automated Testing

### Integration Test Template
```typescript
describe("Authentication - Role Escalation", () => {
  it("prevents STUDENT from creating sessions", async () => {
    const student = await createTestUser("STUDENT");
    const caller = trpc.createCaller({ session: student.session, db });

    await expect(
      caller.session.create({ /* ... */ })
    ).rejects.toThrow("Forbidden");
  });

  it("prevents PARENT from accessing other children", async () => {
    const parent = await createTestUser("PARENT");
    const otherChild = await createTestUser("STUDENT");
    const caller = trpc.createCaller({ session: parent.session, db });

    await expect(
      caller.student.getById({ id: otherChild.id })
    ).rejects.toThrow("Forbidden");
  });
});
```

### Fuzzing Authentication Headers
```bash
# Test with malformed tokens
curl -H "Authorization: Bearer INVALID_TOKEN" https://app.com/api/trpc/session.list
curl -H "Authorization: Bearer " https://app.com/api/trpc/session.list
curl -H "Authorization: " https://app.com/api/trpc/session.list
curl https://app.com/api/trpc/session.list # No auth header
```

## Clerk-Specific Risks

### Webhook Validation
```typescript
// ✅ MUST validate Clerk webhook signatures
import { Webhook } from "svix";

export const POST = async (req: Request) => {
  const payload = await req.text();
  const headers = {
    "svix-id": req.headers.get("svix-id"),
    "svix-timestamp": req.headers.get("svix-timestamp"),
    "svix-signature": req.headers.get("svix-signature"),
  };

  const wh = new Webhook(process.env.CLERK_WEBHOOK_SECRET);

  try {
    wh.verify(payload, headers); // Throws if invalid
  } catch (err) {
    return new Response("Invalid signature", { status: 401 });
  }

  // Process webhook...
};
```

### Session Validation
```typescript
// ✅ MUST validate session on every request
import { auth } from "@clerk/nextjs/server";

export const createContext = async () => {
  const { userId, sessionId } = await auth();

  if (!userId || !sessionId) {
    throw new UnauthorizedError("No active session");
  }

  // Fetch user from DB with role
  const user = await db.user.findUnique({ where: { clerkId: userId } });

  return { userId, sessionId, user, db };
};
```

## Remediation Checklist

- [ ] Audit all 7 roles × 13 resources × 4 actions
- [ ] Test role escalation scenarios
- [ ] Validate Clerk webhook signatures
- [ ] Implement session timeout/refresh logic
- [ ] Test TUTOR_COACH hybrid role thoroughly
- [ ] Test PARENT with 0, 1, 5+ children
- [ ] Add rate limiting on auth endpoints
- [ ] Implement MFA for ADMIN/SUPER_ADMIN roles
- [ ] Add Sentry alerts for repeated auth failures

## Files to Review

**Critical:**
- `src/server/lib/permissions.ts` - Permission matrix
- `src/middleware.ts` - Clerk middleware config
- `src/app/api/webhooks/clerk/route.ts` - Webhook validation
- `src/server/api/trpc.ts` - Context creation + auth

**Testing:**
- `tests/integration/auth/*.test.ts`
- `tests/e2e/auth/*.spec.ts`

## Success Criteria

- [ ] Zero role escalation vulnerabilities
- [ ] All edge cases (TUTOR_COACH, PARENT) tested
- [ ] Clerk webhook validation enforced
- [ ] Session expiration handled correctly
- [ ] 100% permission matrix coverage in tests
- [ ] Security documentation for auth flows