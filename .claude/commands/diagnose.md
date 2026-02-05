---
model: haiku
description: Diagnostic environnement - Vérifie DB, migrations, deps, types. AUTO-SUGGEST sur erreurs Prisma/modules.
---

# /diagnose

Vérifie l'état de l'environnement de développement et suggère des corrections.

## Quand utiliser

- **Automatiquement suggéré** quand Claude détecte ces patterns d'erreur :
  - `Unknown argument` (Prisma) → migration manquante
  - `Cannot find module '@prisma/client'` → client non généré
  - `P1001: Can't reach database` → DATABASE_URL
  - `Module not found` → node_modules manquant
  - `CLERK_SECRET_KEY is not set` → .env incomplet

- **Manuellement** après un `git pull` ou en début de session

## Exécution

### 1. Vérifications parallèles

Lancer ces commandes en parallèle :

```bash
# Git status
git status --short && git branch --show-current
```

```bash
# Node modules check
if [ ! -d "node_modules" ]; then
  echo "❌ MISSING: node_modules"
elif [ "package.json" -nt "node_modules/.modules.yaml" ] 2>/dev/null; then
  echo "⚠️ OUTDATED: pnpm install needed"
else
  echo "✅ OK: deps"
fi
```

```bash
# Prisma client check
if [ ! -f "node_modules/.prisma/client/index.js" ]; then
  echo "❌ MISSING: prisma generate needed"
elif [ "src/server/db/prisma/schema.prisma" -nt "node_modules/.prisma/client/index.js" ]; then
  echo "⚠️ OUTDATED: prisma generate needed"
else
  echo "✅ OK: prisma client"
fi
```

```bash
# Local migrations list
ls -1 src/server/db/prisma/migrations/ 2>/dev/null | tail -3
```

### 2. Check migrations DB (MCP Postgres)

```sql
SELECT migration_name, finished_at::date as applied_at
FROM _prisma_migrations
ORDER BY finished_at DESC
LIMIT 5;
```

Comparer avec les migrations locales :
- Si une migration locale n'est pas en DB → `pnpm prisma migrate deploy` nécessaire
- Si toutes présentes → ✅ synced

### 3. TypeScript (optionnel, si erreurs suspectes)

```bash
pnpm tsc --noEmit 2>&1 | grep -E "^src/" | head -10
```

## Format de sortie

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 Diagnostic Environnement
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 Dépendances:    ✅ OK
🗄️ Migrations DB:  ⚠️ 1 pending (20260107165942_add_training_competencies)
⚡ Prisma Client:   ⚠️ Outdated
📝 TypeScript:     ✅ OK (pre-existing errors ignored)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Actions suggérées

Utiliser `AskUserQuestion` si problèmes détectés :

```
question: "Problèmes détectés. Quelles corrections appliquer ?"
header: "Fixes"
multiSelect: true
options:
  - label: "pnpm install"
    description: "Installer/mettre à jour les dépendances"
  - label: "pnpm prisma migrate deploy"
    description: "Appliquer les migrations en attente"
  - label: "pnpm prisma generate"
    description: "Régénérer le client Prisma"
  - label: "Tout corriger (recommandé)"
    description: "pnpm install && pnpm prisma migrate deploy && pnpm prisma generate"
```

## Exécution des fixes

Si l'utilisateur choisit "Tout corriger" :

```bash
pnpm install && pnpm prisma migrate deploy && pnpm prisma generate
```

Sinon, exécuter les commandes sélectionnées séquentiellement.

## Détection automatique

**IMPORTANT** : Claude doit suggérer `/tech:diagnose` automatiquement quand il voit ces erreurs :

| Erreur | Pattern | Cause probable |
|--------|---------|----------------|
| Prisma field unknown | `Unknown argument 'xxx'` | Migration non appliquée |
| Module not found | `Cannot find module` | node_modules outdated |
| Prisma client missing | `@prisma/client` not found | Client non généré |
| DB connection | `P1001`, `P1002` | DATABASE_URL incorrect |
| Type errors mass | 50+ TS errors soudains | Schema desync |

Exemple de suggestion automatique :
```
Cette erreur "Unknown argument 'competencies'" indique un schéma Prisma
désynchronisé. Je suggère de lancer `/tech:diagnose` pour vérifier
l'état de l'environnement.
```