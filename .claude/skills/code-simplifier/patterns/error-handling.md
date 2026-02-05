# Pattern: Gestion d'Erreurs

## Architecture ProductionError

Le projet utilise une hiérarchie d'erreurs structurées avec intégration Sentry automatique.

---

## Hiérarchie des Erreurs (10 types)

| Type | HTTP | Usage |
|------|------|-------|
| `ResourceNotFoundError` | 404 | Entité non trouvée en BDD |
| `ValidationError` | 400 | Validation input échouée |
| `AuthorizationError` | 403 | Permission refusée |
| `ConflictError` | 409 | Conflit d'état, doublon |
| `DataIntegrityError` | 500 | Violation contrainte BDD |
| `EducationalError` | 409 | Problèmes session/visio |
| `ExternalServiceError` | 500 | Échec API externe |
| `ContextError` | 500 | Problème contexte |
| `ResourceLimitError` | 429 | Rate limiting |
| `MaintenanceError` | 503 | Mode maintenance |

---

## Patterns d'Utilisation

### Service Layer - Throw Approprié

```typescript
import {
  ResourceNotFoundError,
  AuthorizationError,
  ConflictError,
} from "~/server/lib/errors";

export const sessionService = {
  getById: async (ctx: ProtectedContext, id: number) => {
    const session = await sessionRepository.findOne(ctx.db, id);

    // ResourceNotFoundError pour entité absente
    if (!session) {
      throw new ResourceNotFoundError("Session", id.toString());
    }

    // AuthorizationError pour permission refusée
    const scope = getPermissionScope(ctx, "SESSION", session.studentId);
    await enforcePermission(ctx, "READ", "SESSION", scope);

    return session;
  },

  updateStatus: async (ctx: ProtectedContext, id: number, newStatus: SessionStatus) => {
    const session = await sessionRepository.findOne(ctx.db, id);

    if (!session) {
      throw new ResourceNotFoundError("Session", id.toString());
    }

    // ConflictError pour transition invalide
    if (!isValidTransition(session.status, newStatus)) {
      throw new ConflictError(
        `Transition ${session.status} → ${newStatus} non autorisée`,
        { currentStatus: session.status, requestedStatus: newStatus }
      );
    }

    return sessionRepository.updateOne(ctx.db, id, { status: newStatus });
  },
};
```

### Try/Catch Approprié

```typescript
// BIEN - try/catch pour appels externes
const syncWithDailyCo = async (roomId: string) => {
  try {
    const roomData = await dailyCoClient.getRoom(roomId);
    return roomData;
  } catch (error) {
    throw new ExternalServiceError(
      "Daily.co",
      "Failed to sync room",
      { roomId, cause: error }
    );
  }
};

// BIEN - try/catch pour transactions
const createSessionWithHistory = async (ctx: ProtectedContext, input: CreateSessionInput) => {
  try {
    return await ctx.db.$transaction(async (tx) => {
      const session = await tx.session.create({ data: input });
      await tx.sessionHistory.create({
        data: {
          sessionId: session.id,
          toStatus: "SCHEDULED",
          changedBy: ctx.session.userId,
        },
      });
      return session;
    });
  } catch (error) {
    if (error instanceof ProductionError) throw error;
    throw new DataIntegrityError("Failed to create session", { cause: error });
  }
};

// ÉVITER - try/catch inutile
const getUser = async (id: string) => {
  try {
    return await db.user.findUnique({ where: { id } });
  } catch (error) {
    // Re-throw sans valeur ajoutée
    throw error;
  }
};
```

### Re-throw Pattern

```typescript
// Pattern correct pour préserver les erreurs métier
const processSession = async (ctx: ProtectedContext, id: number) => {
  try {
    const session = await sessionService.getById(ctx, id);
    // ... traitement
    return result;
  } catch (error) {
    // Préserver les ProductionError
    if (error instanceof ProductionError) throw error;

    // Wrapper les erreurs inattendues
    throw new DataIntegrityError("Session processing failed", {
      sessionId: id,
      cause: error,
    });
  }
};
```

---

## Frontend - Toast avec Erreurs

```typescript
import { appToast } from "~/lib/appToast";

const mutation = api.session.create.useMutation({
  onSuccess: () => {
    appToast.success("Session créée avec succès");
  },
  onError: (error) => {
    // appToast détecte automatiquement le type d'erreur
    // et affiche un toast enrichi avec modal de debug
    appToast.error(error);
  },
});
```

---

## Patterns de Simplification

### Éviter le Code Défensif Excessif

```typescript
// AVANT - sur-défensif
const getSession = async (ctx: ProtectedContext, id: number) => {
  if (!id) {
    throw new ValidationError("ID requis");
  }
  if (typeof id !== "number") {
    throw new ValidationError("ID doit être un nombre");
  }
  if (id <= 0) {
    throw new ValidationError("ID doit être positif");
  }
  // Zod valide déjà tout ça au niveau router...
};

// APRÈS - validation Zod au router, confiance au service
const getSession = async (ctx: ProtectedContext, id: number) => {
  // id est déjà validé par Zod au niveau du router
  const session = await sessionRepository.findOne(ctx.db, id);
  if (!session) {
    throw new ResourceNotFoundError("Session", id.toString());
  }
  return session;
};
```

### Centraliser les Patterns d'Erreur

```typescript
// BIEN - helper réutilisable
const findOneOrThrow = async <T>(
  findFn: () => Promise<T | null>,
  entityName: string,
  id: string
): Promise<T> => {
  const entity = await findFn();
  if (!entity) {
    throw new ResourceNotFoundError(entityName, id);
  }
  return entity;
};

// Usage
const session = await findOneOrThrow(
  () => sessionRepository.findOne(ctx.db, id),
  "Session",
  id.toString()
);
```

---

## Anti-Patterns à Éviter

### Silent Catch

```typescript
// MAL - erreur avalée
try {
  await riskyOperation();
} catch {
  // Erreur ignorée silencieusement
}

// BIEN - log ou re-throw
try {
  await riskyOperation();
} catch (error) {
  logger.error("Risky operation failed", { error });
  throw new DataIntegrityError("Operation failed", { cause: error });
}
```

### Messages Génériques

```typescript
// MAL - message non informatif
throw new Error("Something went wrong");

// BIEN - message actionnable
throw new ResourceNotFoundError("Session", id.toString());
// Génère: "Session with id '123' not found"
```

---

## Checklist Gestion d'Erreurs

- [ ] Utiliser les types ProductionError appropriés
- [ ] Try/catch uniquement pour transactions et appels externes
- [ ] Re-throw les ProductionError existantes
- [ ] Wrapper les erreurs inattendues avec contexte
- [ ] Éviter les catch silencieux
- [ ] Messages d'erreur explicites et actionnables