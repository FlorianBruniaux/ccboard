# Cybersecurity Skills - Méthode Aristote

Ensemble de skills de sécurité pour auditer et protéger la plateforme Méthode Aristote.

## 📚 Skills Disponibles

 | Skill | Priorité | Status | Description |
|-------|----------|--------|-------------|
| **idor-testing** | 🔴 Critique | ✅ **Complet** | Test des vulnérabilités IDOR dans le système de permissions 3-layer |
| **broken-authentication** | 🔴 Critique | ✅ **Complet** | Test d'escalation de rôles et bypass d'authentification (Clerk + custom) |
| **api-fuzzing-bug-bounty** | 🟡 Haute | ✅ **Complet** | Fuzzing automatique des endpoints tRPC avec payloads malveillants |
| **xss-html-injection** | 🟡 Haute | 📝 Spec only | Test XSS dans le chat, notes tuteurs, feedback parents |
| **sql-injection-testing** | 🟢 Moyenne | 📝 Spec only | Audit des requêtes Prisma raw pour injection SQL |

## 🚀 Utilisation des Skills

### Depuis Claude Code CLI

```bash
# Lancer un audit IDOR
claude "Lance le skill idor-testing sur les routers session et student"

# Tester l'authentification
claude "Exécute broken-authentication pour vérifier les rôles TUTOR_COACH et PARENT"

# Fuzzer l'API
claude "Run api-fuzzing-bug-bounty sur sessionRouter"
```

### Scripts NPM

```bash
# Fuzzing API complet
pnpm security:fuzz

# Fuzzing ciblé
pnpm security:fuzz:session
pnpm security:fuzz:activity

# Tests de sécurité
pnpm test:security              # Tous les tests
pnpm test:security:auth         # Tests authentification
pnpm test:security:idor         # Tests IDOR
pnpm test:security:fuzz         # Tests fuzzing

# Scanners de sécurité
pnpm security:idor:scan         # Scanner IDOR
pnpm security:idor:scan:verbose # Scanner IDOR (mode détaillé)
pnpm security:scan:all          # Tous les scanners (IDOR + Fuzzing)
```

## 🛡️ Automatisation

### Hook Git Pre-Push

Le hook `.husky/pre-push` exécute automatiquement :

1. **API Fuzzing** sur les routers modifiés
2. **IDOR Scanner** sur les services/routers modifiés
3. **SQL Injection Scan** sur les fichiers TypeScript modifiés
4. **XSS Detection** pour `dangerouslySetInnerHTML` sans sanitization
5. **Permission Check Validation** dans les services
6. **TypeScript Type Check**

**Exemple de sortie :**

```bash
$ git push origin feature/new-endpoint

🔒 Running pre-push security checks...

🔍 Checking for changed API routers...
⚠️  Changed routers detected:
    src/server/api/routers/session.ts

🧪 Running API fuzzing on changed routers: sessionRouter
📋 Testing 12 procedures...
✅ FUZZING PASSED - No vulnerabilities detected!

🔍 Scanning for IDOR vulnerabilities...
⚠️  Changed API files detected:
    src/server/api/services/session.ts

🛡️  Running IDOR scanner...
✅ IDOR SCAN PASSED - No vulnerabilities detected!

🔍 Checking for SQL injection patterns...
✅ No SQL injection patterns found

🔍 Running TypeScript type check...
✅ TypeScript check passed

✅ All pre-push security checks passed!
🚀 Safe to push
```

### CI/CD Integration GitHub Actions

Le projet dispose déjà d'un workflow **Claude Code Review** (`.github/workflows/claude-code-review.yml`) qui effectue des reviews automatiques sur les Pull Requests.

#### Workflow Existant : Claude Code Review

**Fichier** : `.github/workflows/claude-code-review.yml`

Ce workflow utilise l'action `anthropics/claude-code-action@v1` pour :
- Analyser automatiquement les diffs de chaque PR
- Vérifier les conventions de code (3-tier architecture, naming, imports)
- Charger les guides spécifiques selon les fichiers modifiés
- Poster des commentaires de review directement sur la PR

**Intégration Sécurité** : Le workflow charge déjà les skills cybersec quand pertinent :

```yaml
# Extrait du prompt du workflow
| Si le diff contient... | Alors lire ce guide/agent |
| services/ ou routers/ | .claude/skills/cybersec/idor-testing.md |
| auth/ ou permissions/ | .claude/skills/cybersec/broken-authentication.md |
| API endpoints | .claude/skills/cybersec/api-fuzzing-bug-bounty.md |
```

#### Workflow Complémentaire : Security Checks

Pour exécuter les tests et scanners de sécurité, ajouter un workflow dédié :

```yaml
# .github/workflows/security.yml
name: Security Checks

on:
  pull_request:
    branches: [develop, main]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: actions/setup-node@v4

      - name: Install dependencies
        run: pnpm install

      - name: IDOR Scanner
        run: pnpm security:idor:scan

      - name: API Fuzzing
        run: pnpm security:fuzz

      - name: Authentication Tests
        run: pnpm test:security:auth

      - name: IDOR Tests
        run: pnpm test:security:idor

      - name: SQL Injection Scan
        run: |
          if grep -r "\$executeRawUnsafe\|\$queryRawUnsafe" src/; then
            echo "❌ Unsafe SQL methods detected"
            exit 1
          fi
```

#### Avantages de la Double Approche

| Workflow | Type | Rôle |
|----------|------|------|
| **claude-code-review.yml** | 🤖 Review IA | Analyse qualitative, conseils, détection patterns |
| **security.yml** | 🧪 Tests auto | Validation quantitative, scanners, tests d'intégration |

**Ensemble**, ils offrent une couverture complète :
- Review IA contextuelle sur chaque PR
- Tests automatisés bloquants si vulnérabilités
- Hook pre-push comme première ligne de défense

## 📊 Résultats Attendus

### ✅ Sécurité Optimale

- **✅ 0 vulnérabilités IDOR** : Scanner détecte permissions/ownership manquants
- **✅ 0 escalation de rôles** : 7 rôles testés, hiérarchie respectée
- **✅ 0 payloads malveillants acceptés** : Fuzzer rejette tous les inputs invalides
- **⚠️ 0 XSS** : Détection basique (à améliorer avec tests E2E)
- **⚠️ 0 SQL injection** : Détection pattern-based (à améliorer avec audit complet)

### ⚠️ Avertissements Tolérables

- `dangerouslySetInnerHTML` avec `DOMPurify` → OK (warning seulement)
- Services sans `enforcePermission` si logique métier ne nécessite pas → Warning

### 🚨 Bloquants

- Fuzzer détecte payload accepté → **PUSH BLOQUÉ**
- SQL injection pattern trouvé → **PUSH BLOQUÉ**
- Test d'authentification échoue → **PUSH BLOQUÉ**

## 🔧 Configuration

### Désactiver le Hook (Urgence)

```bash
# Temporaire (un seul push)
git push --no-verify

# Permanent (déconseillé)
rm .husky/pre-push
```

### Personnaliser le Fuzzing

Éditer `scripts/security/api-fuzzer.ts` :

```typescript
// Ajouter des payloads spécifiques
const FUZZ_PAYLOADS = {
  strings: [
    // ... payloads existants
    "YOUR_CUSTOM_PAYLOAD",
  ],
};

// Exclure certains routers
const EXCLUDED_ROUTERS = ["healthRouter", "publicRouter"];
```

### Ajuster les Seuils

```typescript
// Accepter jusqu'à 5 warnings XSS (pas recommandé)
const MAX_XSS_WARNINGS = 5;

// Timeout pour fuzzing
const FUZZ_TIMEOUT_MS = 30000; // 30 secondes
```

## 📖 Documentation Détaillée

Chaque skill contient :

- **Context** : Pourquoi ce risque existe
- **Attack Vectors** : Comment l'exploiter
- **Testing Protocol** : Comment tester
- **Vulnerable Code Patterns** : Exemples à éviter
- **Secure Code Patterns** : Exemples à suivre
- **Remediation Checklist** : Actions correctives
- **Files to Review** : Où chercher les vulnérabilités

## 🎓 Formation Équipe

### Nouveaux Développeurs

1. Lire les 5 skills (30 min)
2. Exécuter `pnpm security:fuzz --verbose` (observer la sortie)
3. Casser intentionnellement un test de sécurité
4. Corriger la vulnérabilité

### Code Review Checklist

- [ ] Le code touche-t-il les permissions ? → Lire `idor-testing.md`
- [ ] Nouveau rôle/hiérarchie ? → Lire `broken-authentication.md`
- [ ] Nouvel endpoint tRPC ? → Exécuter `pnpm security:fuzz:NEW_ROUTER`
- [ ] User-generated content affiché ? → Lire `xss-html-injection.md`
- [ ] Requête Prisma raw ? → Lire `sql-injection-testing.md`

## 🆘 Que Faire en Cas de Vulnérabilité Détectée

### 1. Ne Pas Paniquer

Les outils de sécurité sont là pour PRÉVENIR, pas pour punir.

### 2. Analyser le Rapport

```bash
# Exécuter en mode verbose pour détails
pnpm tsx scripts/security/api-fuzzer.ts --verbose

# Identifier le payload problématique
# Comprendre pourquoi il a été accepté
```

### 3. Corriger

```typescript
// ❌ Avant
const getSession = async (id: string) => {
  return db.session.findUnique({ where: { id } });
};

// ✅ Après
const getSession = async (ctx: ProtectedContext, id: string) => {
  await enforcePermission(ctx, "SESSION", "READ");
  const session = await sessionRepository.findOne(ctx.db, id);

  if (session.tutorId !== ctx.session.userId) {
    throw new ForbiddenError("Cannot access other tutors' sessions");
  }

  return session;
};
```

### 4. Valider

```bash
# Re-exécuter le fuzzer
pnpm security:fuzz

# Exécuter les tests de sécurité
pnpm test:security
```

### 5. Documenter

Ajouter un test de non-régression :

```typescript
it("prevents TUTOR from accessing other tutors' sessions", async () => {
  // Test case...
});
```

## 🔗 Ressources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [tRPC Security Best Practices](https://trpc.io/docs/server/authorization)
- [Prisma Security](https://www.prisma.io/docs/concepts/components/prisma-client/raw-database-access#sql-injection-prevention)
- [Clerk Security](https://clerk.com/docs/security/overview)

## 📝 Changelog

- **2025-01-10** : Création des 5 skills cybersec + hook pre-push + fuzzer