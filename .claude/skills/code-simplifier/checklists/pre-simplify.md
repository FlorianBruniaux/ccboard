# Checklist: Avant Simplification

## Vérifications Obligatoires

Avant toute modification de code, valider chaque point.

---

## 1. Scope

- [ ] **Scope défini** - Fichiers/fonctions ciblées clairement identifiées
- [ ] **Pas de scope creep** - Limiter aux zones demandées par l'utilisateur
- [ ] **Impact évalué** - Comprendre les dépendances du code ciblé

```bash
# Trouver les références au code ciblé
grep -r "nomFonction" src/
```

---

## 2. Compréhension

- [ ] **Code lu intégralement** - Pas de modification sans lecture complète
- [ ] **Comportement compris** - Inputs, outputs, effets de bord identifiés
- [ ] **Edge cases identifiés** - Cas limites et gestion d'erreurs notés

```typescript
// Documenter le comportement actuel
/*
 * Input: SessionInput avec tutorId optionnel
 * Output: Session créée avec visio
 * Effets: Notification au tuteur, log Sentry
 * Erreurs: ResourceNotFoundError si tuteur invalide
 */
```

---

## 3. Tests Existants

- [ ] **Tests présents** - Vérifier existence de tests pour le code ciblé
- [ ] **Tests passent** - Exécuter les tests AVANT modification
- [ ] **Couverture suffisante** - Si couverture faible, alerter l'utilisateur

```bash
# Vérifier les tests existants
ls tests/unit/**/session*.test.ts
pnpm test:repositories
pnpm test:services
```

---

## 4. Conventions Projet

- [ ] **Arrow functions** - Pas de `function` keyword dans le scope
- [ ] **import type** - Types importés séparément
- [ ] **Zod schemas** - Pas d'enums TypeScript
- [ ] **Architecture 3-tier** - Séparation router/service/repository respectée

---

## 5. Risques Identifiés

- [ ] **Fonctions exportées** - Ne pas modifier les signatures publiques
- [ ] **Types partagés** - Attention aux types utilisés ailleurs
- [ ] **Effets de bord** - Préserver l'ordre et l'existence des effets
- [ ] **Transactions** - Ne pas casser les blocs transactionnels

---

## 6. Plan de Simplification

- [ ] **Changements listés** - Chaque modification planifiée
- [ ] **Ordre défini** - Séquence logique d'application
- [ ] **Rollback possible** - Capacité à annuler si problème

---

## Questions à Poser

Avant de commencer, clarifier avec l'utilisateur:

1. **Scope exact** - "Quels fichiers/fonctions dois-je simplifier ?"
2. **Priorités** - "Clarté ou performance prioritaire ?"
3. **Contraintes** - "Y a-t-il des parties à ne pas toucher ?"
4. **Tests** - "Les tests existants couvrent-ils le comportement ?"

---

## Red Flags - STOP

Ne PAS procéder si:

- [ ] Aucun test existant et comportement complexe
- [ ] Code critique en production sans rollback possible
- [ ] Scope flou ou trop large
- [ ] Dépendances non identifiées
- [ ] Deadline serrée sans marge d'erreur

**Action**: Alerter l'utilisateur et demander clarification.