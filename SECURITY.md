# Security Policy

## Supported Versions

ccboard is currently in active development. Security updates are provided for the latest release only.

| Version | Supported          |
| ------- | ------------------ |
| 0.5.x   | :white_check_mark: |
| < 0.5   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security vulnerability in ccboard, please report it responsibly:

### Contact

- **Email**: florian.bruniaux@gmail.com
- **Subject**: `[SECURITY] ccboard - Brief Description`

### Response Timeline

- **Acknowledgment**: Within 48 hours of receiving your report
- **Initial Assessment**: Within 7 days
- **Fix & Disclosure**: Coordinated timeline based on severity

### What to Include

To help us triage and fix the issue quickly, please include:

1. **Description**: Clear explanation of the vulnerability
2. **Impact**: Potential security impact and affected versions
3. **Steps to Reproduce**: Detailed steps or proof-of-concept
4. **Environment**: OS, Rust version, ccboard version
5. **Suggested Fix**: Optional, but appreciated

### Responsible Disclosure

We follow responsible disclosure principles:

- We will acknowledge receipt of your report within 48 hours
- We will provide regular updates on our progress
- We will credit you in the security advisory (unless you prefer to remain anonymous)
- We will coordinate the public disclosure timeline with you

### Security Update Process

1. **Fix Development**: We develop and test the fix
2. **Advisory Draft**: We prepare a security advisory
3. **Coordinated Release**: We release the fix and publish the advisory
4. **Public Disclosure**: We announce the vulnerability and fix

## Security Best Practices

When using ccboard:

- **Keep Updated**: Always use the latest release for security fixes
- **File Permissions**: Ensure `~/.claude` has appropriate permissions (readable only by you)
- **Sensitive Data**: ccboard reads local files but never transmits data externally
- **Code Review**: ccboard is open-source - audit the code if handling sensitive projects

## Scope

### In Scope

- Security vulnerabilities in ccboard code
- Data leakage through file system operations
- Unsafe code that could lead to memory issues
- Dependencies with known vulnerabilities

### Out of Scope

- Issues with Claude Code itself (report to Anthropic)
- User misconfiguration
- Social engineering attacks
- Physical access vulnerabilities

## Security Considerations

ccboard is designed with security in mind:

- **Read-Only Operations**: ccboard only reads from `~/.claude`, never writes (MVP)
- **Local Processing**: All data processing is local, no external connections
- **Minimal Dependencies**: Small dependency footprint to reduce attack surface
- **Memory Safety**: Rust's memory safety guarantees prevent common vulnerabilities

---

**Last Updated**: 2026-02-10
**Contact**: florian.bruniaux@gmail.com
