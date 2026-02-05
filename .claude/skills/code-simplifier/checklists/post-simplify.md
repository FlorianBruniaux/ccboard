# Checklist: Après Simplification

## Validations Obligatoires

Après chaque modification, valider TOUS ces points.

---

## 1. Qualité du Code

### Lint et TypeScript

```bash
# OBLIGATOIRE - Exécuter après chaque modification
pnpm lint && pnpm tsc
```

- [ ] **Lint passe** - Aucune erreur ESLint
- [ ] **TypeScript passe** - Aucune erreur de compilation
- [ ] **Warnings acceptables** - Pas de nouveaux warnings critiques

### Si Échec

```bash
# Auto-fix les erreurs lint quand possible
pnpm lint:fix

# Vérifier les erreurs restantes
pnpm lint
pnpm tsc --noEmit
```

---

## 2. Tests

- [ ] **Tests existants passent** - Aucune régression
- [ ] **Comportement identique** - Même résultats pour mêmes inputs

```bash
# Exécuter les tests pertinents
pnpm test:repositories
pnpm test:services
pnpm test:lib

# Ou tous les tests
pnpm test
```

---

## 3. Fonctionnalité Préservée

### Vérification Manuelle

- [ ] **Valeurs de retour** - Mêmes outputs pour mêmes inputs
- [ ] **Effets de bord** - Notifications, logs, mutations préservés
- [ ] **Exceptions** - Mêmes erreurs dans mêmes conditions
- [ ] **Types exposés** - Signatures publiques inchangées

### Questions de Validation

1. Un test existant échoue-t-il ? → **Annuler**
2. Le comportement observable a-t-il changé ? → **Annuler**
3. Une signature publique a-t-elle changé ? → **Annuler**

---

## 4. Conventions Respectées

- [ ] **Arrow functions** - Aucun `function` keyword ajouté
- [ ] **import type** - Types importés correctement
- [ ] **Nommage** - Variables explicites, fichiers kebab-case
- [ ] **Architecture** - Séparation 3-tier maintenue

---

## 5. Documentation

- [ ] **Commentaires WHY** - Si logique non évidente, expliquer pourquoi
- [ ] **Pas de commentaires WHAT** - Le code doit s'auto-documenter
- [ ] **Changelog** - Si changement significatif, noter pour commit

---

## 6. Résumé pour l'Utilisateur

Préparer un résumé clair:

```markdown
## Simplifications Effectuées

### Fichier: `src/server/api/services/session.ts`

1. **Réduction d'imbrication** - `createSession()`: 4 → 2 niveaux
2. **Extraction fonction** - `validateSessionInput()` extraite
3. **Early returns** - Guards au début de `updateStatus()`

### Impact
- Lignes: 120 → 95 (-21%)
- Complexité cyclomatique: 12 → 7
- Comportement: IDENTIQUE

### Validation
- `pnpm lint`: ✅
- `pnpm tsc`: ✅
- Tests: ✅ (15/15)
```

---

## 7. Rollback si Problème

### Signes d'Alerte

- Tests qui échouaient pas avant
- Erreurs TypeScript nouvelles
- Comportement différent observé

### Actions

```bash
# Annuler les modifications non committées
git checkout -- src/path/to/file.ts

# Ou restaurer depuis dernier commit
git restore src/path/to/file.ts
```

---

## Validation Finale

| Check | Status |
|-------|--------|
| `pnpm lint` | ⬜ |
| `pnpm tsc` | ⬜ |
| Tests passent | ⬜ |
| Comportement identique | ⬜ |
| Conventions respectées | ⬜ |
| Résumé préparé | ⬜ |

**Tous les checks doivent être ✅ avant de considérer la simplification terminée.**