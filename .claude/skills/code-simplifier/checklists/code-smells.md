# Checklist: Code Smells

## Indicateurs de Code à Simplifier

Ces patterns signalent du code qui bénéficierait d'une simplification.

---

## 1. Imbrication Profonde

### Détection

```typescript
// SMELL - 4+ niveaux d'imbrication
if (user) {
  if (user.role === "TUTOR") {
    if (user.isActive) {
      if (user.sessions.length > 0) {
        // Logique enterrée
      }
    }
  }
}
```

### Solution

```typescript
// CLEAN - early returns
if (!user) return null;
if (user.role !== "TUTOR") return null;
if (!user.isActive) return null;
if (user.sessions.length === 0) return null;

// Logique principale au même niveau
```

**Règle**: Max 3 niveaux d'imbrication.

---

## 2. Fonctions Trop Longues

### Détection

- Fonction > 50 lignes
- Multiple responsabilités mélangées
- Difficile à nommer précisément

```typescript
// SMELL - 80 lignes, 5 responsabilités
const handleSessionSubmit = async (data) => {
  // Validation (20 lignes)
  // Transformation (15 lignes)
  // API call (10 lignes)
  // Post-processing (20 lignes)
  // Notification (15 lignes)
};
```

### Solution

```typescript
// CLEAN - fonctions focalisées
const validateSessionData = (data: SessionInput): ValidationResult => { /* ... */ };
const transformToPayload = (data: SessionInput): ApiPayload => { /* ... */ };
const createSessionApi = async (payload: ApiPayload) => { /* ... */ };
const notifyParticipants = async (session: Session) => { /* ... */ };

const handleSessionSubmit = async (data: SessionInput) => {
  const validation = validateSessionData(data);
  if (!validation.success) return { error: validation.errors };

  const payload = transformToPayload(data);
  const session = await createSessionApi(payload);
  await notifyParticipants(session);

  return { success: true, session };
};
```

**Règle**: Une fonction = une responsabilité, < 50 lignes.

---

## 3. Ternaires Imbriqués

### Détection

```typescript
// SMELL
const status = isLoading
  ? "loading"
  : hasError
    ? error.code === 404
      ? "not-found"
      : "error"
    : data
      ? "success"
      : "empty";
```

### Solution

```typescript
// CLEAN - fonction explicite
const getStatus = (): Status => {
  if (isLoading) return "loading";

  if (hasError) {
    return error.code === 404 ? "not-found" : "error";
  }

  return data ? "success" : "empty";
};
```

**Règle**: Max 1 niveau de ternaire, sinon if/else ou switch.

---

## 4. Code Dupliqué (DRY Violation)

### Détection

```typescript
// SMELL - même logique répétée
const formatTutorName = (tutor: Tutor) =>
  `${tutor.firstName} ${tutor.lastName}`.trim();

const formatStudentName = (student: Student) =>
  `${student.firstName} ${student.lastName}`.trim();

const formatParentName = (parent: Parent) =>
  `${parent.firstName} ${parent.lastName}`.trim();
```

### Solution

```typescript
// CLEAN - abstraction après 3ème occurrence
type Person = { firstName: string; lastName: string };

const formatFullName = (person: Person) =>
  `${person.firstName} ${person.lastName}`.trim();
```

**Règle**: Abstraire à la 3ème occurrence identique.

---

## 5. Nommage Non Explicite

### Détection

```typescript
// SMELL
const d = new Date();
const u = users.find((x) => x.id === id);
const temp = data.map((item) => item.value);
const result = process(input);
```

### Solution

```typescript
// CLEAN
const currentDate = new Date();
const targetUser = users.find((user) => user.id === userId);
const itemValues = data.map((item) => item.value);
const processedSession = processSession(sessionInput);
```

**Règle**: Le nom doit décrire le contenu/purpose sans ambiguïté.

---

## 6. Violations Conventions Projet

### Détection

```typescript
// SMELL - function keyword
function getUser(id: string) { /* ... */ }

// SMELL - enum TypeScript
enum Status { ACTIVE, INACTIVE }

// SMELL - import type mélangé
import { User, createUser } from "@prisma/client";
```

### Solution

```typescript
// CLEAN
const getUser = (id: string) => { /* ... */ };

// Zod schema
const StatusSchema = z.enum(["ACTIVE", "INACTIVE"]);

// import type séparé
import type { User } from "@prisma/client";
import { createUser } from "~/services/user";
```

---

## 7. Architecture 3-Tier Violée

### Détection

```typescript
// SMELL - Prisma dans router
create: protectedProcedure
  .input(schema)
  .mutation(async ({ ctx, input }) => {
    return ctx.db.session.create({ data: input }); // Direct DB access
  });

// SMELL - Business logic dans repository
findActiveUsers: async (db) => {
  const users = await db.user.findMany();
  return users.filter((u) => isWithinBusinessHours(u)); // Business logic
};
```

### Solution

Voir `patterns/3-tier-cleanup.md` pour les corrections.

---

## 8. Abstractions Inutiles

### Détection

```typescript
// SMELL - wrapper sans valeur
class UserWrapper {
  constructor(private user: User) {}
  getName() { return this.user.name; }
  getEmail() { return this.user.email; }
  // Juste des proxies...
}

// SMELL - factory pour un seul type
const createNotification = (type: string) => {
  switch (type) {
    case "email": return new EmailNotification();
    // Un seul case utilisé...
  }
};
```

### Solution

Supprimer l'abstraction, utiliser directement l'objet/fonction.

---

## Tableau Récapitulatif

| Smell | Seuil | Action |
|-------|-------|--------|
| Imbrication | > 3 niveaux | Early returns |
| Fonction longue | > 50 lignes | Extraction |
| Ternaires | > 1 niveau | if/else ou switch |
| Duplication | >= 3 occurrences | Abstraction |
| Nommage | Variables 1-2 lettres | Renommer explicitement |
| Convention | function/enum/import | Corriger selon projet |
| Architecture | DB dans router | Extraire vers service |
| Abstraction | Wrapper 1:1 | Supprimer |