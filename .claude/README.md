# ccboard Claude Code Setup

Configuration optimisée pour le développement Rust avec Claude Code.

## Structure

```
.claude/
├─ settings.local.json      # Permissions Rust étendues (git-ignored)
├─ agents/
│  └─ rust-ccboard.md       # Agent expert Rust pour ccboard (workspace, parsers, concurrency)
├─ skills/
│  └─ tdd-rust/
│     └─ SKILL.md          # Workflow TDD Rust (Red-Green-Refactor)
└─ hooks/
   └─ bash/
      └─ pre-commit-format.sh  # Auto-format + clippy avant commits
```

## Usage

### Agents

**Invoke Rust expert agent:**
```
@rust-ccboard implement extract_metadata function with JSONL streaming
```

L'agent active automatiquement :
- Patterns spécifiques ccboard (DashMap, parking_lot, graceful degradation)
- Error handling (anyhow/thiserror)
- Performance patterns (lazy loading, parallel scanning)
- Pre-commit checklist automatique

### Skills

**TDD workflow:**
```
/tdd add session metadata extraction
```

Force le cycle Red-Green-Refactor :
1. Écrit le test qui échoue
2. Implémente le code minimal
3. Refactor
4. Pre-commit checks

**Équivalent à utiliser le skill global:**
```
/rust-expert refactor parser with error handling
```

### Hooks

Le hook `pre-commit-format.sh` s'exécute automatiquement avant les commits si configuré dans `settings.json` (non-local).

**Pour activer (settings.json globale ou projet) :**
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash.*git commit",
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/bash/pre-commit-format.sh"
          }
        ]
      }
    ]
  }
}
```

## Permissions

`settings.local.json` autorise automatiquement :
- Commandes Cargo (build, test, clippy, fmt, watch)
- Exécutables ccboard (debug + release)
- Outils de recherche (rtk, rg, fd)
- Commandes lecture (ls, cat, head, tail, wc)

## Commandes recommandées

```bash
# Development avec agent
claude
> @rust-ccboard fix parser error handling

# TDD workflow
claude
> /tdd implement session metadata

# Quick test
cargo test -p ccboard-core

# Pre-commit manual
cargo fmt --all && cargo clippy --all-targets && cargo test --all
```

## Best Practices

✅ Toujours utiliser `@rust-ccboard` pour le code core (parsers, models, store)
✅ Utiliser `/tdd` pour nouvelles features avec tests
✅ Lancer `cargo fmt && cargo clippy` avant chaque commit
✅ Tests dans `#[cfg(test)] mod tests` (embedded) ou `tests/` (integration)

❌ Éviter `.unwrap()` en production (sauf tests avec `.expect()`)
❌ Ne pas parser JSONL entier au startup (lazy metadata)
❌ Ne pas oublier `.context()` avec `?` operator