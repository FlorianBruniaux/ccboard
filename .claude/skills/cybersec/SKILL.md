---
name: cybersec
description: >
  Security audit toolkit for the ccboard project (Rust/Leptos/Axum stack).
  Use when auditing authentication flows, API endpoints, WASM security,
  or reviewing Rust code for unsafe usage and memory safety issues.
version: 1.0.0
effort: high
allowed-tools: [Read, Grep, Glob, Bash]
tags: [security, audit, rust, axum, leptos, wasm]
---

# Cybersec Skills — ccboard

Security audit toolkit for the ccboard Rust/Leptos stack.

## Reference files in this directory

These files are reference documentation for security test patterns.
Update them to reflect ccboard's actual Axum routes, auth flows, and data model.

## Key security areas for ccboard

- **Authentication**: session tokens, CSRF protection in Leptos forms
- **Authorization**: route-level guards in Axum middleware
- **WASM**: client-side code exposure, sensitive data in WASM binary
- **SQL**: raw query usage, input sanitization
- **Dependency audit**: `cargo audit` for known CVEs in dependencies

## Usage

```bash
cargo audit                         # CVE scan
grep -r "unsafe" src/               # unsafe block audit
grep -r "unwrap()" src/ | wc -l    # panic surface audit
```
