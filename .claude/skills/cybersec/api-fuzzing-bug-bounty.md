---
skill_name: api-fuzzing-bug-bounty
description: Automated fuzzing of tRPC endpoints to discover input validation vulnerabilities
category: cybersec
priority: high
tags: [security, fuzzing, api, trpc, zod, validation]
---

# API Fuzzing & Bug Bounty Testing - Méthode Aristote

**PRIORITY #3 Security Risk**

## Context

**tRPC = Type-safe ≠ Security-safe**

Méthode Aristote has 50+ tRPC procedures across multiple routers:
- `sessionRouter` - 10+ procedures (session data = sensitive)
- `activityRouter` - 8+ procedures (student performance data)
- `studentRouter` - 6+ procedures (PII: names, grades, parent info)
- `workplanRouter` - 5+ procedures (learning analytics)

**Risk**: Zod validates types, NOT business logic. Fuzzing can expose:
- SQL injection in raw queries
- Path traversal in file operations
- Integer overflow in calculations
- Logic bugs (negative prices, future dates, etc.)

## Attack Surface Map

### High-Value Targets
```typescript
// Critical endpoints with sensitive data
sessionRouter.create          // Can create sessions for other users?
sessionRouter.update          // Can modify others' sessions?
activityRouter.submitAnswer   // Can submit for other students?
studentRouter.updateProgress  // Can manipulate grades?
workplanRouter.generateAI     // Can trigger expensive AI calls?
```

### Input Vectors
1. **Numeric Inputs**: IDs, counts, durations, prices
2. **String Inputs**: Names, descriptions, URLs, file paths
3. **Date Inputs**: Session dates, deadlines
4. **Enum Inputs**: Roles, statuses, activity types
5. **Array Inputs**: Multiple selections, batch operations
6. **Object Inputs**: Nested data structures

## Fuzzing Test Cases

### 1. Boundary Value Analysis

#### Integer Overflow
```typescript
// Test session duration
await trpc.session.create.mutate({
  duration: Number.MAX_SAFE_INTEGER, // 9007199254740991 minutes
  duration: -999999, // Negative duration
  duration: 0, // Zero duration
  duration: 1.5, // Float when expecting integer
});
```

#### String Length
```typescript
// Test description field
await trpc.activity.create.mutate({
  description: "a".repeat(1000000), // 1MB string
  description: "", // Empty string
  description: "\0\0\0", // Null bytes
  description: "<?xml>", // XML injection attempt
});
```

#### Array Bounds
```typescript
// Test student IDs array
await trpc.workplan.generateBatch.mutate({
  studentIds: Array(10000).fill("student_123"), // 10k IDs
  studentIds: [], // Empty array
  studentIds: [null, undefined, "", "DROP TABLE"], // Invalid items
});
```

### 2. Type Confusion

```typescript
// Send wrong types despite Zod validation
await fetch("/api/trpc/session.create", {
  method: "POST",
  body: JSON.stringify({
    // Zod expects string, send object
    id: { $ne: null }, // MongoDB-style injection

    // Zod expects number, send string
    duration: "999999999999999999999",

    // Zod expects date, send crafted string
    scheduledAt: "'; DROP TABLE sessions; --",

    // Zod expects enum, send array
    status: ["SCHEDULED", "COMPLETED"], // Confusion attack
  }),
});
```

### 3. SQL Injection (Prisma Raw Queries)

```typescript
// If you use $executeRaw or $queryRaw anywhere:
await trpc.session.search.query({
  query: "'; DELETE FROM sessions WHERE '1'='1", // Classic SQL injection
  query: "1' UNION SELECT password FROM users--", // Union-based
  query: "1' AND SLEEP(10)--", // Time-based blind SQLi
});
```

### 4. Path Traversal

```typescript
// If file operations exist (document upload, export, etc.)
await trpc.document.download.query({
  path: "../../../../etc/passwd", // Unix path traversal
  path: "..\\..\\..\\windows\\system32\\config\\sam", // Windows
  path: "/proc/self/environ", // Environment variable leak
});
```

### 5. Logic Bombs

```typescript
// Business logic vulnerabilities
await trpc.session.create.mutate({
  tutorId: "user_attacker",
  studentId: "user_victim",
  scheduledAt: new Date("1970-01-01"), // Past date
  scheduledAt: new Date("2099-12-31"), // Far future
  price: -100, // Negative price (credit instead of charge?)
  duration: 0.000001, // Microsecond session
});
```

### 6. Race Conditions

```typescript
// Concurrent requests to exploit TOCTOU bugs
const promises = Array(100).fill(null).map(() =>
  trpc.session.cancel.mutate({ id: "session_123" })
);

await Promise.all(promises);
// Did all 100 succeed? Were refunds issued 100x?
```

## Automated Fuzzing Script

### Basic Fuzzer
```typescript
// scripts/security/api-fuzzer.ts
import { appRouter } from "~/server/api/root";

const fuzzPayloads = {
  strings: [
    "", // Empty
    "a".repeat(1000000), // Large
    "\0\0\0", // Null bytes
    "'OR'1'='1", // SQL injection
    "<script>alert(1)</script>", // XSS
    "../../../etc/passwd", // Path traversal
    "\u0000", // Null character
    "🔥💀🚀", // Unicode edge cases
  ],
  numbers: [
    0, -1, Number.MAX_SAFE_INTEGER, Number.MIN_SAFE_INTEGER,
    1.1, -999999, 0.000001, Infinity, NaN,
  ],
  arrays: [
    [], [null], [undefined], Array(10000).fill("x"),
  ],
  objects: [
    {}, { $ne: null }, { __proto__: { admin: true } },
  ],
};

const fuzzProcedure = async (procedureName: string, schema: ZodSchema) => {
  console.log(`Fuzzing ${procedureName}...`);

  for (const payload of generatePayloads(schema, fuzzPayloads)) {
    try {
      // @ts-expect-error - Intentionally sending bad data
      await trpc[procedureName].mutate(payload);
      console.warn(`⚠️ ${procedureName} accepted invalid payload:`, payload);
    } catch (err) {
      // Expected - validation should reject
      if (!err.message.includes("validation")) {
        console.error(`🚨 Unexpected error in ${procedureName}:`, err);
      }
    }
  }
};

// Run fuzzer
for (const [routerName, router] of Object.entries(appRouter._def.procedures)) {
  await fuzzProcedure(routerName, router.input);
}
```

### Git Hook Integration

Create `.husky/pre-push` to fuzz changed endpoints:

```bash
#!/bin/sh
# .husky/pre-push

echo "🔍 Running API fuzzing on changed routers..."

# Get changed router files
CHANGED_ROUTERS=$(git diff --name-only main...HEAD | grep "src/server/api/routers/")

if [ -n "$CHANGED_ROUTERS" ]; then
  echo "Changed routers detected:"
  echo "$CHANGED_ROUTERS"

  # Run targeted fuzzing
  pnpm tsx scripts/security/api-fuzzer.ts --routers="$CHANGED_ROUTERS"

  if [ $? -ne 0 ]; then
    echo "❌ API fuzzing failed. Push blocked."
    exit 1
  fi

  echo "✅ API fuzzing passed"
fi
```

## Advanced Fuzzing Techniques

### Mutation-Based Fuzzing
```typescript
// Start with valid payload, mutate incrementally
const validPayload = {
  tutorId: "user_123",
  studentId: "student_456",
  scheduledAt: new Date(),
  duration: 60,
};

const mutations = [
  { tutorId: null },
  { tutorId: "" },
  { tutorId: "admin" },
  { studentId: validPayload.tutorId }, // Same as tutor
  { duration: -60 },
  { scheduledAt: "not a date" },
];

for (const mutation of mutations) {
  await fuzz({ ...validPayload, ...mutation });
}
```

### Grammar-Based Fuzzing
```typescript
// Generate inputs following Zod schema grammar
const generateFromSchema = (schema: ZodSchema) => {
  if (schema instanceof z.ZodString) {
    return [
      faker.lorem.word(),
      faker.lorem.paragraph(100), // Long
      "", // Empty
      "🚀", // Unicode
    ];
  }

  if (schema instanceof z.ZodNumber) {
    return [0, -1, 1e10, Number.MAX_SAFE_INTEGER, 0.1, -0.1];
  }

  // ... handle all Zod types
};
```

## Bug Bounty Focus Areas

### 1. Session Manipulation
- Create sessions with past dates
- Assign sessions to other tutors
- Cancel completed sessions
- Duplicate session IDs

### 2. Grade Tampering
- Submit answers for other students
- Modify activity scores
- Bypass time limits on timed activities
- Replay submissions

### 3. Financial Logic
- Negative prices
- Zero-cost sessions
- Refund exploitation
- Currency confusion (if multi-currency)

### 4. Rate Limiting Bypass
- AI generation endpoints (expensive)
- Email sending
- Report generation
- Webhook endpoints

## Detection & Monitoring

### Logging Suspicious Patterns
```typescript
// middleware.ts
export const fuzzyInputDetector = async (req: Request) => {
  const body = await req.json();

  const suspiciousPatterns = [
    /(['";]|--|\/\*|\*\/)/gi, // SQL injection
    /(\.\.\/|\.\.\\)/g, // Path traversal
    /<script|javascript:/gi, // XSS
    /\x00/g, // Null bytes
  ];

  const bodyStr = JSON.stringify(body);
  for (const pattern of suspiciousPatterns) {
    if (pattern.test(bodyStr)) {
      await logSecurityEvent({
        type: "FUZZING_ATTEMPT",
        pattern: pattern.source,
        payload: body,
        ip: req.headers.get("x-forwarded-for"),
      });
    }
  }
};
```

### Sentry Integration
```typescript
import * as Sentry from "@sentry/nextjs";

if (suspiciousInput) {
  Sentry.captureException(new Error("Fuzzing attempt detected"), {
    tags: { security: "fuzzing" },
    extra: { payload: input, user: ctx.session.userId },
  });
}
```

## Remediation Checklist

- [ ] Add input length limits to all string fields
- [ ] Validate numeric ranges (price > 0, duration > 0, etc.)
- [ ] Check date sanity (no past/far-future dates)
- [ ] Escape special characters in search queries
- [ ] Implement rate limiting on expensive operations
- [ ] Add fuzzing to CI/CD pipeline
- [ ] Monitor Sentry for fuzzing patterns
- [ ] Document valid input ranges in Zod schemas

## Files to Create/Review

**Create:**
- `scripts/security/api-fuzzer.ts` - Main fuzzing script
- `scripts/security/fuzz-payloads.json` - Payload library
- `.husky/pre-push` - Git hook integration
- `tests/security/api-fuzzing.test.ts` - Automated fuzzing tests

**Review:**
- All `src/server/api/routers/*.ts` - Input validation
- `src/server/api/services/*.ts` - Business logic checks
- `src/server/api/repositories/*.ts` - Raw query usage

## Success Criteria

- [ ] Automated fuzzing script covers all routers
- [ ] Git hook blocks pushes with vulnerable endpoints
- [ ] Zero SQL injection vulnerabilities (if using raw queries)
- [ ] All numeric inputs have min/max validation
- [ ] All string inputs have length limits
- [ ] Date inputs validated against business rules
- [ ] Rate limiting on AI/expensive operations
- [ ] Security monitoring dashboard in Sentry