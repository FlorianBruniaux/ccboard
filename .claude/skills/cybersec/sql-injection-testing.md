---
skill_name: sql-injection-testing
description: Test for SQL injection vulnerabilities in Prisma raw queries and edge cases
category: cybersec
priority: medium
tags: [security, sql-injection, prisma, database]
---

# SQL Injection Testing - Méthode Aristote

**PRIORITY #5 Security Risk**

## Context

**Prisma ORM = Generally Safe** from SQL injection because:
- Parameterized queries by default
- Type-safe query builder

**BUT vulnerable if using:**
- `db.$executeRaw()` - Execute raw SQL
- `db.$queryRaw()` - Query raw SQL
- `db.$executeRawUnsafe()` - Unsafe raw execution
- `db.$queryRawUnsafe()` - Unsafe raw query
- String interpolation in SQL

**Risk**: Even ONE unsafe query = entire DB compromised.

## Attack Vectors

### 1. Raw Query Injection

#### ❌ VULNERABLE Pattern
```typescript
// NEVER DO THIS
const search = async (query: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions WHERE title LIKE '%${query}%'
  `);
};

// Exploited by:
// query = "'; DROP TABLE sessions; --"
// Resulting SQL: SELECT * FROM sessions WHERE title LIKE '%'; DROP TABLE sessions; --%'
```

#### ✅ SAFE Pattern
```typescript
// Use parameterized queries
import { Prisma } from "@prisma/client";

const search = async (query: string) => {
  return db.$queryRaw<Session[]>`
    SELECT * FROM sessions WHERE title LIKE ${`%${query}%`}
  `;
  // Prisma escapes the parameter
};
```

### 2. Dynamic Table/Column Names

#### ❌ VULNERABLE Pattern
```typescript
// Dynamic column sorting
const getSessions = async (sortBy: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions ORDER BY ${sortBy} DESC
  `);
};

// Exploited by:
// sortBy = "id; DROP TABLE sessions; --"
```

#### ✅ SAFE Pattern
```typescript
// Whitelist allowed columns
const ALLOWED_SORT_COLUMNS = ["id", "title", "scheduledAt"] as const;
type SortColumn = typeof ALLOWED_SORT_COLUMNS[number];

const getSessions = async (sortBy: SortColumn) => {
  if (!ALLOWED_SORT_COLUMNS.includes(sortBy)) {
    throw new Error("Invalid sort column");
  }

  // Safe: sortBy is validated
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions ORDER BY ${sortBy} DESC
  `);
};
```

### 3. LIKE Clause Injection

```typescript
// Search with wildcards
const search = async (query: string) => {
  // Vulnerable if query contains SQL wildcards: %, _
  return db.session.findMany({
    where: {
      title: { contains: query }, // Safe with Prisma
    },
  });
};

// Exploited by: query = "%"
// Returns ALL sessions (information disclosure)
```

### 4. JSON Query Injection (PostgreSQL)

```typescript
// PostgreSQL JSONB queries
const findByMetadata = async (key: string, value: string) => {
  // Vulnerable
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions WHERE metadata->>'${key}' = '${value}'
  `);
};

// Exploited by:
// key = "status' OR '1'='1"
// Resulting SQL: ... WHERE metadata->>'status' OR '1'='1' = 'value'
```

## Testing Protocol

### Step 1: Find Raw Query Usage

```bash
# Search for potentially vulnerable patterns
grep -r "\$executeRaw" src/server/
grep -r "\$queryRaw" src/server/
grep -r "Unsafe" src/server/
grep -r "Prisma.sql" src/server/
```

**Expected Result:** Zero or minimal usage, all with parameterized queries.

### Step 2: Test Classic SQL Injection Payloads

#### Union-Based
```sql
' UNION SELECT null, username, password FROM users--
' UNION SELECT null, null, null, null--
```

#### Boolean-Based Blind
```sql
' OR '1'='1
' OR '1'='1'--
' OR 'a'='a
```

#### Time-Based Blind
```sql
'; SELECT pg_sleep(10)--
' AND (SELECT CASE WHEN (1=1) THEN pg_sleep(5) ELSE pg_sleep(0) END)--
```

#### Comment Injection
```sql
'--
'/*
'#
```

#### Stacked Queries
```sql
'; DROP TABLE sessions; --
'; DELETE FROM users WHERE '1'='1'; --
```

### Step 3: Test Each Endpoint

```typescript
// Example test for search endpoint
describe("SQL Injection - Session Search", () => {
  const sqlInjectionPayloads = [
    "' OR '1'='1",
    "'; DROP TABLE sessions; --",
    "' UNION SELECT * FROM users--",
    "admin'--",
    "' AND 1=1--",
    "' AND SLEEP(5)--",
  ];

  for (const payload of sqlInjectionPayloads) {
    it(`rejects SQL injection: ${payload}`, async () => {
      const caller = await createTestCaller();

      const result = await caller.session.search({
        query: payload,
      });

      // Should return empty or throw error, NOT execute SQL
      expect(result).toEqual([]);

      // Verify sessions table still exists
      const count = await db.session.count();
      expect(count).toBeGreaterThan(0);
    });
  }
});
```

### Step 4: Check Database Schema After Tests

```sql
-- Verify tables weren't dropped
SELECT table_name FROM information_schema.tables
WHERE table_schema = 'public';

-- Check for unexpected data modifications
SELECT COUNT(*) FROM sessions;
SELECT COUNT(*) FROM users;
```

## Vulnerable Code Patterns

### ❌ Pattern 1: String Interpolation
```typescript
const getSessionsByStatus = async (status: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions WHERE status = '${status}'
  `);
};
```

### ❌ Pattern 2: Dynamic Column Names
```typescript
const sortSessions = async (column: string, direction: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions ORDER BY ${column} ${direction}
  `);
};
```

### ❌ Pattern 3: Complex WHERE Clauses
```typescript
const customFilter = async (whereClause: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions WHERE ${whereClause}
  `);
};
```

### ✅ Safe Alternative: Use Prisma Query Builder
```typescript
const getSessionsByStatus = async (status: SessionStatus) => {
  return db.session.findMany({
    where: { status }, // Parameterized automatically
  });
};

const sortSessions = async (orderBy: keyof Session, direction: "asc" | "desc") => {
  return db.session.findMany({
    orderBy: { [orderBy]: direction },
  });
};
```

### ✅ Safe Alternative: Parameterized Raw Queries
```typescript
const complexQuery = async (userId: string, startDate: Date) => {
  return db.$queryRaw<Session[]>`
    SELECT s.* FROM sessions s
    WHERE s.tutor_id = ${userId}
    AND s.scheduled_at >= ${startDate}
  `;
  // Prisma escapes parameters
};
```

## PostgreSQL-Specific Risks

### JSONB Queries
```typescript
// ❌ VULNERABLE
const findByJson = (key: string, value: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions WHERE metadata->>'${key}' = '${value}'
  `);
};

// ✅ SAFE
const findByJson = (key: string, value: string) => {
  return db.$queryRaw`
    SELECT * FROM sessions WHERE metadata->>${key} = ${value}
  `;
};
```

### Array Queries
```typescript
// ❌ VULNERABLE
const findByIds = (ids: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions WHERE id = ANY(ARRAY[${ids}])
  `);
};

// ✅ SAFE
const findByIds = (ids: string[]) => {
  return db.session.findMany({
    where: { id: { in: ids } },
  });
};
```

### Full-Text Search
```typescript
// ❌ VULNERABLE
const fullTextSearch = (query: string) => {
  return db.$queryRawUnsafe(`
    SELECT * FROM sessions WHERE to_tsvector(title) @@ to_tsquery('${query}')
  `);
};

// ✅ SAFE
const fullTextSearch = (query: string) => {
  return db.$queryRaw`
    SELECT * FROM sessions WHERE to_tsvector(title) @@ to_tsquery(${query})
  `;
};
```

## Automated Detection

### Static Analysis Script
```typescript
// scripts/security/sql-injection-scanner.ts
import { readFileSync, readdirSync } from "fs";
import { join } from "path";

const vulnerablePatterns = [
  /\$executeRawUnsafe/g,
  /\$queryRawUnsafe/g,
  /\$\{.*\}.*sql/gi, // Template literal in SQL
  /WHERE.*\+.*\+/g, // String concatenation in WHERE
];

const scanFile = (filePath: string) => {
  const content = readFileSync(filePath, "utf8");
  const violations = [];

  for (const pattern of vulnerablePatterns) {
    const matches = content.match(pattern);
    if (matches) {
      violations.push({
        file: filePath,
        pattern: pattern.source,
        count: matches.length,
      });
    }
  }

  return violations;
};

const scanDirectory = (dir: string) => {
  // Recursively scan all .ts files
  // Report violations
};

console.log("🔍 Scanning for SQL injection vulnerabilities...");
const violations = scanDirectory("src/server");

if (violations.length > 0) {
  console.error("❌ Found potential SQL injection vulnerabilities:");
  console.table(violations);
  process.exit(1);
} else {
  console.log("✅ No SQL injection vulnerabilities detected");
}
```

### Pre-commit Hook
```bash
# .husky/pre-commit
#!/bin/sh

echo "🔍 Checking for SQL injection patterns..."

# Scan staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E "\.(ts|tsx)$")

if [ -n "$STAGED_FILES" ]; then
  for FILE in $STAGED_FILES; do
    # Check for unsafe patterns
    if grep -E "\$executeRawUnsafe|\$queryRawUnsafe" "$FILE"; then
      echo "❌ Found unsafe SQL pattern in $FILE"
      echo "Use \$executeRaw or \$queryRaw with parameterized queries instead"
      exit 1
    fi
  done
fi

echo "✅ No SQL injection patterns found"
```

## Remediation Checklist

- [ ] Audit all files for `$executeRawUnsafe` and `$queryRawUnsafe`
- [ ] Replace unsafe queries with parameterized versions
- [ ] Whitelist dynamic column/table names
- [ ] Add SQL injection tests to CI/CD
- [ ] Install pre-commit hook to block unsafe patterns
- [ ] Document safe query patterns in team guidelines
- [ ] Train team on Prisma security best practices

## Files to Review

**Critical:**
- `src/server/api/repositories/*.ts` - Direct DB access
- `src/server/api/services/*.ts` - May call raw queries
- `src/server/lib/db/*.ts` - Database utilities

**Create:**
- `scripts/security/sql-injection-scanner.ts`
- `tests/integration/sql-injection.test.ts`
- `.husky/pre-commit` (add SQL check)

## Success Criteria

- [ ] Zero `$executeRawUnsafe` or `$queryRawUnsafe` usage
- [ ] All raw queries use parameterized syntax
- [ ] Dynamic identifiers use whitelisting
- [ ] Automated scanning in CI/CD
- [ ] Pre-commit hook blocks unsafe patterns
- [ ] Integration tests for SQL injection attempts
- [ ] Documentation on safe query practices