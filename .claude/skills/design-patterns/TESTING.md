# Guide de Test - Design Patterns Analyzer

**Projet**: Méthode Aristote
**Skill**: design-patterns
**Stack détectée**: Next.js 15.5 + React 19 + tRPC + Prisma + TypeScript

---

## Vue d'ensemble

Le skill `design-patterns` analyse votre codebase pour:
- **Détecter** les patterns GoF existants (23 patterns couverts)
- **Suggérer** des patterns pour corriger les code smells
- **Évaluer** la qualité des implémentations de patterns

**Particularité**: Adapte ses recommandations à votre stack (préfère les idiomes React/Next.js/tRPC/Prisma aux implémentations manuelles).

---

## 🧪 Mode 1: Detection

### Commande
```bash
claude "Utilise le skill design-patterns en mode Detection sur src/server/api/"
```

### Ce que ça fait
- Scanne les fichiers TypeScript/TSX
- Détecte les patterns GoF implémentés
- Identifie s'ils sont natifs à la stack ou custom
- Calcule un score de confiance (0.0-1.0)

### Output attendu (JSON)
```json
{
  "stack_detected": {
    "primary": "nextjs",
    "version": "15.5",
    "secondary": ["react", "trpc", "prisma", "typescript"],
    "detection_sources": ["package.json", "tsconfig.json", "*.tsx files"]
  },
  "patterns_found": {
    "factory-method": [
      {
        "file": "src/server/api/services/errors.ts",
        "lines": "45-89",
        "confidence": 0.85,
        "type": "custom",
        "name": "ErrorFactory"
      }
    ],
    "observer": [
      {
        "file": "src/server/api/routers/session.ts",
        "lines": "150-200",
        "confidence": 0.9,
        "type": "native",
        "implementation": "tRPC subscription + EventSource"
      }
    ],
    "repository": [
      {
        "file": "src/server/api/services/session.ts",
        "lines": "50-300",
        "confidence": 0.95,
        "type": "native",
        "implementation": "Prisma ORM"
      }
    ]
  },
  "summary": {
    "total": 4,
    "native_to_stack": 2,
    "custom_implementations": 2,
    "by_category": {"creational": 1, "structural": 1, "behavioral": 2}
  }
}
```

### Tests réalisés (session précédente)

✅ **Résultats confirmés**:
- **ErrorFactory** (Factory Method): `src/server/api/services/errors.ts` - Score 8.2/10
- **Observer via EventSource**: `src/server/api/routers/session.ts` - Score 9.1/10
- **Repository via Prisma**: Natif, excellente intégration
- **Strategy-like**: Détecté dans les routers tRPC

---

## 🔍 Mode 2: Suggestion

### Commande
```bash
claude "Utilise le skill design-patterns en mode Suggestion, analyse src/server/api/routers/ pour détecter les code smells"
```

### Ce que ça fait
- Détecte les code smells (switch sur type, long parameter list, global state, etc.)
- Suggère des patterns GoF pour les corriger
- Adapte les suggestions à votre stack (React/Next.js/tRPC/Prisma)

### Output attendu (Markdown)
```markdown
## Pattern Suggestions

**Stack détectée**: Next.js 15.5 + React 19 + tRPC + Prisma

---

### High Priority: Strategy → `src/server/api/routers/payment.ts:45-89`

**Smell**: Switch sur type de paiement (ligne 52)
**Impact**: High (complexité cyclomatique, difficile à étendre)

**Code actuel**:
```typescript
switch (paymentType) {
  case 'stripe': /* 20 lignes */ break;
  case 'paypal': /* 15 lignes */ break;
}
```

**Suggestion adaptée tRPC + Prisma**:
```typescript
// Créer des procédures tRPC séparées
const paymentStrategies = {
  stripe: stripePaymentProcedure,
  paypal: paypalPaymentProcedure,
};

export const paymentRouter = router({
  process: publicProcedure
    .input(z.object({ type: z.enum(['stripe', 'paypal']), amount: z.number() }))
    .mutation(({ input, ctx }) => {
      return paymentStrategies[input.type]({ amount: input.amount, ctx });
    }),
});
```

**Pourquoi pas un pattern manual**: tRPC encourage déjà la séparation des procédures (Strategy natif).

---

### Medium Priority: Builder → `src/components/SessionForm.tsx:120-180`

**Smell**: Long parameter list (>6 params) pour créer une session
**Impact**: Medium (lisibilité, maintenabilité)

**Suggestion adaptée React + TypeScript**:
```typescript
// Utiliser un hook custom avec progressive disclosure
const useSessionBuilder = () => {
  const [session, setSession] = useState<Partial<SessionData>>({});

  return {
    session,
    setTitle: (title: string) => setSession(prev => ({ ...prev, title })),
    setDuration: (duration: number) => setSession(prev => ({ ...prev, duration })),
    setType: (type: SessionType) => setSession(prev => ({ ...prev, type })),
    // ... autres setters
    build: () => validateAndCreateSession(session),
  };
};
```

**Pourquoi pas un pattern manual**: React hooks sont le Builder pattern idiomatique en React.
```
```

### Tests réalisés (session précédente)

✅ **Code smells identifiés**:
1. **Switch sur type de session**: Suggéré Strategy via tRPC procedures
2. **État partagé sessions**: Suggéré Context API + Provider au lieu de Singleton
3. **Long parameter list**: Suggéré Builder via React hooks

---

## 📊 Mode 3: Evaluation

### Commande
```bash
claude "Utilise le skill design-patterns en mode Evaluation, évalue la qualité du ErrorFactory dans src/server/api/services/errors.ts"
```

### Ce que ça fait
- Analyse un pattern détecté selon 5 critères (0-10 chaque)
- Calcule un score global pondéré
- Identifie les problèmes par priorité
- Suggère des améliorations concrètes

### Critères d'évaluation

| Critère | Poids | Description |
|---------|-------|-------------|
| **Correctness** | 30% | Respect de la structure canonique du pattern |
| **Testability** | 25% | Facilité à mocker/tester |
| **Single Responsibility** | 20% | Une seule responsabilité claire |
| **Open/Closed** | 15% | Extensible sans modification |
| **Documentation** | 10% | Clarté de l'intent et usage |

### Output attendu (JSON)
```json
{
  "pattern": "factory-method",
  "file": "src/server/api/services/errors.ts",
  "lines": "45-89",
  "scores": {
    "correctness": 9,
    "testability": 8,
    "single_responsibility": 9,
    "open_closed": 7,
    "documentation": 7
  },
  "overall_score": 8.2,
  "interpretation": "Good - Minor improvements, production-ready",
  "issues": [
    {
      "priority": "medium",
      "criterion": "open_closed",
      "description": "Adding new error types requires modifying ErrorFactory",
      "recommendation": "Use a registry pattern: errorFactory.register('NotFound', NotFoundError)"
    },
    {
      "priority": "low",
      "criterion": "documentation",
      "description": "Missing JSDoc explaining when to use each error type",
      "recommendation": "Add usage examples in JSDoc comments"
    }
  ],
  "recommendations": [
    {
      "title": "Improve extensibility with registry",
      "code_example": "..."
    }
  ]
}
```

### Tests réalisés (session précédente)

✅ **ErrorFactory évalué**: Score global 8.2/10
- Correctness: 9/10 (implémentation correcte)
- Testability: 8/10 (facilement mockable)
- Single Responsibility: 9/10 (focus sur création d'erreurs)
- Open/Closed: 7/10 (requiert modification pour nouveaux types)
- Documentation: 7/10 (manque exemples d'usage)

---

## 🎯 Commandes de Test Recommandées

### Test Complet (tous modes)
```bash
# 1. Détection globale
claude "Skill design-patterns mode Detection: analyse complète de src/"

# 2. Suggestions par zone
claude "Skill design-patterns mode Suggestion: analyse src/server/api/routers/ pour code smells"
claude "Skill design-patterns mode Suggestion: analyse src/components/ pour patterns React"

# 3. Évaluation des patterns détectés
claude "Skill design-patterns mode Evaluation: évalue ErrorFactory dans src/server/api/services/errors.ts"
claude "Skill design-patterns mode Evaluation: évalue les patterns Observer détectés"
```

### Tests Ciblés (par catégorie)

**Creational Patterns**:
```bash
claude "Skill design-patterns: détecte Singleton, Factory, Builder dans src/server/"
```

**Structural Patterns**:
```bash
claude "Skill design-patterns: détecte Decorator, Adapter, Facade dans src/lib/"
```

**Behavioral Patterns**:
```bash
claude "Skill design-patterns: détecte Observer, Strategy, Command dans src/server/api/"
```

---

## 📋 Checklist de Test

### Phase 1: Validation de base
- [ ] Le skill détecte correctement la stack (Next.js 15.5 + React 19 + tRPC + Prisma)
- [ ] Mode Detection retourne un JSON valide
- [ ] Les patterns détectés correspondent aux fichiers indiqués
- [ ] Les scores de confiance sont cohérents (0.0-1.0)

### Phase 2: Validation des suggestions
- [ ] Mode Suggestion identifie au moins 3 code smells
- [ ] Les suggestions utilisent les idiomes de la stack (React hooks, tRPC procedures, Prisma)
- [ ] Les exemples de code sont syntaxiquement corrects
- [ ] Les priorités (High/Medium/Low) sont justifiées

### Phase 3: Validation de l'évaluation
- [ ] Mode Evaluation calcule les 5 scores (0-10)
- [ ] Le score global pondéré est cohérent
- [ ] Les recommandations sont actionnables
- [ ] Les exemples de code amélioré sont fournis

### Phase 4: Stack awareness
- [ ] Le skill préfère React Context au lieu de Singleton manuel
- [ ] Le skill suggère tRPC procedures au lieu de Strategy manuel
- [ ] Le skill recommande Prisma Repository au lieu d'implémentation custom
- [ ] Le skill utilise React hooks au lieu de Builder classes

---

## 🔧 Debugging

### Le skill ne détecte pas la stack correctement
```bash
# Vérifier les sources de détection
cat package.json | grep -E "react|next|trpc|prisma"
cat tsconfig.json | grep "compilerOptions"
ls src/**/*.tsx | head -5
```

### Patterns non détectés
```bash
# Vérifier les signatures de détection
cat .claude/skills/design-patterns/signatures/detection-rules.yaml | grep -A 10 "singleton:"
```

### Suggestions non adaptées à la stack
```bash
# Vérifier les patterns natifs définis
cat .claude/skills/design-patterns/signatures/stack-patterns.yaml | grep -A 20 "react:"
```

---

## 📚 Références

### Documentation complète
- **Guide complet**: `/Users/florianbruniaux/Sites/perso/claude-code-ultimate-guide/guide/ultimate-guide.md` (section 5.4)
- **Fichiers de référence**: `.claude/skills/design-patterns/reference/*.md`
- **Règles de détection**: `.claude/skills/design-patterns/signatures/detection-rules.yaml`
- **Patterns par stack**: `.claude/skills/design-patterns/signatures/stack-patterns.yaml`

### Patterns couverts (23 GoF)

**Creational (5)**: Singleton, Factory Method, Abstract Factory, Builder, Prototype
**Structural (7)**: Adapter, Bridge, Composite, Decorator, Facade, Flyweight, Proxy
**Behavioral (11)**: Chain of Responsibility, Command, Iterator, Mediator, Memento, Observer, State, Strategy, Template Method, Visitor, Interpreter

### Stacks supportées (8)
React, Angular, NestJS, Vue 3, Express, RxJS, Redux/Zustand, Prisma/TypeORM

---

## ✅ Résultats Attendus (Méthode Aristote)

Basés sur l'analyse déjà effectuée:

| Pattern | Fichier | Score | Type | Notes |
|---------|---------|-------|------|-------|
| Factory Method | `src/server/api/services/errors.ts` | 8.2/10 | Custom | ErrorFactory bien implémenté |
| Observer | `src/server/api/routers/session.ts` | 9.1/10 | Native | EventSource + tRPC subscription |
| Repository | `src/server/api/services/session.ts` | 9.5/10 | Native | Prisma ORM |
| Strategy | `src/server/api/routers/*.ts` | 8.0/10 | Native | tRPC procedures |

**Code smells identifiés**: 3 (switch sur type, global state, long parameter list)
**Suggestions actionnables**: 3 avec exemples de code adaptés à la stack

---

## 💡 Tips

1. **Privilégier les patterns natifs**: Si tRPC/Prisma/React offrent une solution idiomatique, utilisez-la plutôt qu'une implémentation manuelle
2. **Mode Suggestion en premier**: Identifier les code smells avant d'implémenter de nouveaux patterns
3. **Évaluer régulièrement**: Lancer Evaluation après refactoring pour mesurer l'amélioration
4. **Filtrer par priorité**: Focus sur les suggestions High Priority en premier
5. **Documenter les choix**: Noter dans les commits pourquoi un pattern a été choisi (ou refusé)

---

**Dernière mise à jour**: 2026-01-21
**Version du skill**: 1.0.0
**Testé sur**: Méthode Aristote (commit 8e9f2e7a)
