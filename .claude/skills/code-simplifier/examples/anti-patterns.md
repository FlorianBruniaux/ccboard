# Exemples: Anti-Patterns

## Ce qu'il ne faut PAS faire

Erreurs courantes de simplification à éviter.

---

## 1. Changement de Comportement Caché

### INTERDIT

```typescript
// AVANT - retourne null si non trouvé
const findUser = async (id: string) => {
  return db.user.findUnique({ where: { id } });
};

// "SIMPLIFICATION" - VIOLATION!
// Change le comportement: lance une erreur au lieu de retourner null
const findUser = async (id: string) => {
  const user = await db.user.findUnique({ where: { id } });
  if (!user) throw new ResourceNotFoundError("User", id);
  return user;
};
```

**Problème**: Le code appelant s'attend à `null`, pas à une exception.

### CORRECT

```typescript
// Si on veut changer le comportement, créer une NOUVELLE fonction
const findUserOrThrow = async (id: string) => {
  const user = await db.user.findUnique({ where: { id } });
  if (!user) throw new ResourceNotFoundError("User", id);
  return user;
};

// L'originale reste inchangée
const findUser = async (id: string) => {
  return db.user.findUnique({ where: { id } });
};
```

---

## 2. Sur-Simplification avec Perte de Clarté

### INTERDIT

```typescript
// AVANT - clair mais verbeux
const getSessionStatus = (session: Session): StatusInfo => {
  if (session.status === "SCHEDULED") {
    return { label: "Programmée", color: "blue", canStart: true };
  }
  if (session.status === "STARTED") {
    return { label: "En cours", color: "green", canStart: false };
  }
  if (session.status === "COMPLETED") {
    return { label: "Terminée", color: "gray", canStart: false };
  }
  return { label: "Inconnue", color: "red", canStart: false };
};

// "SIMPLIFICATION" - TROP COMPRESSÉ
const getSessionStatus = (s: Session) =>
  s.status === "SCHEDULED" ? { l: "Programmée", c: "blue", cs: true }
  : s.status === "STARTED" ? { l: "En cours", c: "green", cs: false }
  : s.status === "COMPLETED" ? { l: "Terminée", c: "gray", cs: false }
  : { l: "Inconnue", c: "red", cs: false };
```

**Problème**: Illisible, noms cryptiques, ternaires imbriqués.

### CORRECT

```typescript
// Garder la version claire, ou utiliser un mapping
const STATUS_INFO: Record<SessionStatus, StatusInfo> = {
  SCHEDULED: { label: "Programmée", color: "blue", canStart: true },
  STARTED: { label: "En cours", color: "green", canStart: false },
  COMPLETED: { label: "Terminée", color: "gray", canStart: false },
  CANCELLED: { label: "Annulée", color: "red", canStart: false },
};

const getSessionStatus = (session: Session): StatusInfo =>
  STATUS_INFO[session.status] ?? { label: "Inconnue", color: "red", canStart: false };
```

---

## 3. Suppression de Try/Catch Nécessaire

### INTERDIT

```typescript
// AVANT - gestion d'erreur appropriée
const syncWithDailyCo = async (roomId: string) => {
  try {
    const roomData = await dailyCoClient.getRoom(roomId);
    return roomData;
  } catch (error) {
    throw new ExternalServiceError("Daily.co", "Sync failed", { cause: error });
  }
};

// "SIMPLIFICATION" - VIOLATION!
// Supprime la gestion d'erreur métier
const syncWithDailyCo = async (roomId: string) => {
  return dailyCoClient.getRoom(roomId);
};
```

**Problème**: Les erreurs Daily.co ne sont plus encapsulées dans notre hiérarchie.

---

## 4. Conversion en function keyword

### INTERDIT

```typescript
// AVANT - convention projet
const createSession = async (input: SessionInput) => {
  // ...
};

// "SIMPLIFICATION" - VIOLATION CONVENTION!
async function createSession(input: SessionInput) {
  // ...
}
```

**Problème**: Viole la convention arrow functions du projet.

---

## 5. Modification de Signature Publique

### INTERDIT

```typescript
// AVANT - API publique
export const sessionService = {
  create: async (ctx: ProtectedContext, input: CreateSessionInput) => {
    // ...
    return session;
  },
};

// "SIMPLIFICATION" - BREAKING CHANGE!
export const sessionService = {
  // Changement de signature: input devient optionnel
  create: async (ctx: ProtectedContext, input?: CreateSessionInput) => {
    if (!input) return null;
    // ...
    return session;
  },
};
```

**Problème**: Tous les appelants s'attendent à l'ancienne signature.

---

## 6. Suppression d'Abstractions Utiles

### INTERDIT

```typescript
// AVANT - architecture 3-tier respectée
// Router
create: protectedProcedure
  .input(schema)
  .mutation(({ ctx, input }) => sessionService.create(ctx, input)),

// Service
create: async (ctx, input) => {
  await enforcePermission(ctx, "SESSION", "CREATE");
  return sessionRepository.createOne(ctx.db, input);
},

// "SIMPLIFICATION" - VIOLATION ARCHITECTURE!
create: protectedProcedure
  .input(schema)
  .mutation(async ({ ctx, input }) => {
    // Tout dans le router
    await enforcePermission(ctx, "SESSION", "CREATE");
    return ctx.db.session.create({ data: input });
  }),
```

**Problème**: Détruit la séparation 3-tier, mélange responsabilités.

---

## 7. One-Liner Illisible

### INTERDIT

```typescript
// AVANT - étapes claires
const processData = (data: RawData) => {
  const validated = validateData(data);
  const transformed = transformData(validated);
  const enriched = enrichData(transformed);
  return enriched;
};

// "SIMPLIFICATION" - ILLISIBLE
const processData = (data: RawData) =>
  enrichData(transformData(validateData(data)));
```

**Problème**: Difficile à débugger, à tester, à comprendre.

---

## 8. Suppression de Commentaires WHY

### INTERDIT

```typescript
// AVANT
// 15 minutes de tolérance pour démarrer la session
// (règle métier: punctualité tuteur)
const PUNCTUALITY_TOLERANCE_MS = 15 * 60 * 1000;

// "SIMPLIFICATION"
const PUNCTUALITY_TOLERANCE_MS = 900000;
```

**Problème**: Le magic number perd son contexte métier.

---

## 9. Abstraction Prématurée

### INTERDIT

```typescript
// AVANT - code direct simple
const formatUserName = (user: User) =>
  `${user.firstName} ${user.lastName}`;

// "SIMPLIFICATION" - SUR-INGÉNIERIE
interface NameFormatter<T> {
  format(entity: T): string;
}

class UserNameFormatter implements NameFormatter<User> {
  format(user: User): string {
    return `${user.firstName} ${user.lastName}`;
  }
}

const userFormatter = new UserNameFormatter();
const name = userFormatter.format(user);
```

**Problème**: Abstraction inutile pour un seul cas d'usage.

---

## 10. Ignorer les Tests Existants

### INTERDIT

```typescript
// Test existant
test("findUser returns null for unknown id", async () => {
  const result = await findUser("unknown-id");
  expect(result).toBeNull();
});

// "SIMPLIFICATION" qui casse le test
const findUser = async (id: string) => {
  const user = await db.user.findUnique({ where: { id } });
  if (!user) throw new Error("Not found"); // Casse le test!
  return user;
};
```

**Problème**: Les tests sont la spécification du comportement. Les ignorer = changer le comportement.

---

## Checklist Anti-Patterns

Avant de valider une simplification, vérifier:

- [ ] Aucun test existant ne devient rouge
- [ ] Aucune signature publique modifiée
- [ ] Conventions projet respectées (arrow functions, etc.)
- [ ] Try/catch préservés pour appels externes/transactions
- [ ] Architecture 3-tier maintenue
- [ ] Commentaires WHY préservés
- [ ] Code reste lisible (pas de one-liners complexes)
- [ ] Pas d'abstraction prématurée ajoutée