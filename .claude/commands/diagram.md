---
model: sonnet
description: Genere des diagrammes ASCII expliquant l'architecture/decisions de la session courante
---

# /diagram

Analyse le contexte de la conversation et genere 1-3 diagrammes ASCII pertinents.

## Arguments

- `$ARGUMENTS` : Optionnel. Focus sur un aspect specifique (ex: "permission flow", "session states")

## Comportement

1. Analyser le contexte conversation (ce qui a ete implemente/discute dans la session)
2. Si `$ARGUMENTS` fourni : focus sur cet aspect specifique
3. Si contexte ambigu et pas d'arguments : poser UNE question via `AskUserQuestion`

```
question: "Quel aspect de l'implementation veux-tu diagrammer ?"
header: "Diagram"
options:
  - label: "Data flow"
    description: "Client → Router → Service → Repository → DB"
  - label: "Component tree"
    description: "Hierarchie composants React impactes"
  - label: "State machine"
    description: "Transitions d'etats (sessions, payments, etc.)"
  - label: "Sequence"
    description: "Interactions entre acteurs/systemes"
```

4. Generer 1-3 diagrammes ASCII selon le type de changement
5. Chaque diagramme avec une legende courte (1-2 lignes max)

## Types de diagrammes

Choisir le(s) type(s) le(s) plus pertinent(s) selon le contexte :

### Data Flow (changements backend, 3-tier)

```
Client ──→ Router ──→ Service ──→ Repository ──→ DB
             │           │
             │       Permission
             │        Check
           Zod
         Validation
```

### Component Tree (changements frontend)

```
src/
├── components/
│   ├── session-card.tsx     ← modified
│   └── session-list.tsx     ← modified
└── server/
    └── api/
        └── routers/session.ts ← modified
```

### State Machine (workflows, status transitions)

```
SCHEDULED ──→ STARTED ──→ COMPLETED
    │                        ↑
    └──→ CANCELLED           │
                    STARTED ─┘
```

### Sequence (interactions multi-acteurs)

```
User          Frontend        tRPC           Service
 │──── click ────→│              │               │
 │               │── mutation ──→│               │
 │               │              │── validate ──→│
 │               │              │←── result ────│
 │←── toast ─────│              │               │
```

### Decision Tree (choix d'architecture)

```
Permission check?
├─ YES: has role? ──→ allow
│       └─ NO ──→ 403
└─ NO: public route ──→ allow
```

### Dependency Graph (relations entre modules)

```
sessionRouter
├──→ sessionService
│    ├──→ sessionRepository
│    └──→ permissionService
└──→ validationSchemas
```

## Regles de generation STRICTES

### Caracteres autorises

```
─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼ ← → ↑ ↓ ──→ ◆ ○ ●
```

### Contraintes

- **Max 80 colonnes** de large (terminal standard)
- **1-3 diagrammes** max par invocation
- **ASCII pur** : pas de mermaid, pas d'unicode fancy, pas d'emoji dans les diagrammes
- **Legende courte** : 1-2 lignes max par diagramme, pas de paragraphe
- **Focus architecture** : expliquer le "pourquoi", pas le code

## Format de sortie

```markdown
## Architecture : {Feature Name}

### {Diagram Title}

{ASCII diagram}

> {1-2 line legend}

### {Optional: Second Diagram}

{ASCII diagram}

> {legend}
```

## Exemples d'invocation

```
/tech:diagram
→ Analyse la session, genere diagramme(s) du travail effectue

/tech:diagram permission flow
→ Genere le flow de permissions (3-tier + enforcePermission)

/tech:diagram session states
→ Genere la state machine des sessions

/tech:diagram component tree
→ Genere l'arborescence des composants impactes
```
