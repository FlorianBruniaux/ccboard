# Exemples: Bons Refactorings

## Cas Réels de Simplification

Exemples concrets avant/après pour guider les décisions.

---

## 1. Service Layer - Réduction d'Imbrication

### Avant (Problématique)

```typescript
// src/server/api/services/session.ts
export const sessionService = {
  start: async (ctx: ProtectedContext, id: number) => {
    const session = await sessionRepository.findOne(ctx.db, id);

    if (session) {
      if (session.status === "SCHEDULED") {
        if (session.tutorId === ctx.session.userId || isAdmin(ctx)) {
          if (session.visio) {
            if (!session.visio.isExpired) {
              const updated = await sessionRepository.updateOne(ctx.db, id, {
                status: "STARTED",
                startedAt: new Date(),
              });

              await sessionHistoryRepository.createOne(ctx.db, {
                sessionId: id,
                fromStatus: "SCHEDULED",
                toStatus: "STARTED",
                changedBy: ctx.session.userId,
              });

              return updated;
            } else {
              throw new EducationalError("Visio expired");
            }
          } else {
            throw new ResourceNotFoundError("Visio", id.toString());
          }
        } else {
          throw new AuthorizationError("Cannot start session");
        }
      } else {
        throw new ConflictError("Session not in SCHEDULED status");
      }
    } else {
      throw new ResourceNotFoundError("Session", id.toString());
    }
  },
};
```

**Problèmes**: 7 niveaux d'imbrication, flux difficile à suivre, erreurs enterrées.

### Après (Simplifié)

```typescript
// src/server/api/services/session.ts
export const sessionService = {
  start: async (ctx: ProtectedContext, id: number) => {
    // 1. Récupération avec validation existence
    const session = await sessionRepository.findOne(ctx.db, id);
    if (!session) {
      throw new ResourceNotFoundError("Session", id.toString());
    }

    // 2. Validation statut
    if (session.status !== "SCHEDULED") {
      throw new ConflictError("Session not in SCHEDULED status", {
        currentStatus: session.status,
      });
    }

    // 3. Validation permissions
    await enforcePermission(ctx, "SESSION", "UPDATE", {
      resourceId: id.toString(),
      ownerId: session.tutorId,
    });

    // 4. Validation visio
    if (!session.visio) {
      throw new ResourceNotFoundError("Visio", id.toString());
    }
    if (session.visio.isExpired) {
      throw new EducationalError("Visio expired", { visioId: session.visio.id });
    }

    // 5. Mise à jour
    const updated = await sessionRepository.updateOne(ctx.db, id, {
      status: "STARTED",
      startedAt: new Date(),
    });

    // 6. Audit trail
    await sessionHistoryRepository.createOne(ctx.db, {
      sessionId: id,
      fromStatus: "SCHEDULED",
      toStatus: "STARTED",
      changedBy: ctx.session.userId,
    });

    return updated;
  },
};
```

**Améliorations**:
- Early returns avec erreurs explicites
- Max 2 niveaux d'imbrication
- Flux linéaire et lisible
- Permission via enforcePermission standard

---

## 2. Composant React - Extraction de Logique

### Avant (Problématique)

```typescript
// src/components/sessions/session-list.tsx
export const SessionList = () => {
  const [statusFilter, setStatusFilter] = useState<SessionStatus | "all">("all");
  const [dateRange, setDateRange] = useState<DateRange | null>(null);
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");

  const { data: sessions, isLoading, error } = api.session.getAll.useQuery();

  const filteredSessions = useMemo(() => {
    if (!sessions) return [];

    let result = [...sessions];

    if (statusFilter !== "all") {
      result = result.filter((s) => s.status === statusFilter);
    }

    if (dateRange) {
      result = result.filter((s) => {
        const date = new Date(s.scheduledAt);
        return date >= dateRange.start && date <= dateRange.end;
      });
    }

    result.sort((a, b) => {
      const comparison = new Date(a.scheduledAt).getTime() - new Date(b.scheduledAt).getTime();
      return sortOrder === "asc" ? comparison : -comparison;
    });

    return result;
  }, [sessions, statusFilter, dateRange, sortOrder]);

  const stats = useMemo(() => ({
    total: sessions?.length ?? 0,
    scheduled: sessions?.filter((s) => s.status === "SCHEDULED").length ?? 0,
    completed: sessions?.filter((s) => s.status === "COMPLETED").length ?? 0,
    cancelled: sessions?.filter((s) => s.status === "CANCELLED").length ?? 0,
  }), [sessions]);

  const handleStatusChange = useCallback((status: SessionStatus | "all") => {
    setStatusFilter(status);
  }, []);

  // ... 50 lignes de plus pour le rendu
};
```

### Après (Simplifié)

```typescript
// src/components/sessions/hooks/use-session-list.ts
type SessionFilters = {
  status: SessionStatus | "all";
  dateRange: DateRange | null;
  sortOrder: "asc" | "desc";
};

export const useSessionList = () => {
  const [filters, setFilters] = useState<SessionFilters>({
    status: "all",
    dateRange: null,
    sortOrder: "desc",
  });

  const { data: sessions, isLoading, error } = api.session.getAll.useQuery();

  const filteredSessions = useMemo(() => {
    if (!sessions) return [];
    return filterAndSortSessions(sessions, filters);
  }, [sessions, filters]);

  const stats = useMemo(() => calculateSessionStats(sessions), [sessions]);

  const updateFilter = useCallback(<K extends keyof SessionFilters>(
    key: K,
    value: SessionFilters[K]
  ) => {
    setFilters((prev) => ({ ...prev, [key]: value }));
  }, []);

  return {
    sessions: filteredSessions,
    stats,
    filters,
    updateFilter,
    isLoading,
    error,
  };
};

// src/components/sessions/utils/session-filters.ts
export const filterAndSortSessions = (
  sessions: Session[],
  filters: SessionFilters
): Session[] => {
  let result = [...sessions];

  if (filters.status !== "all") {
    result = result.filter((s) => s.status === filters.status);
  }

  if (filters.dateRange) {
    result = result.filter((s) => isWithinDateRange(s.scheduledAt, filters.dateRange));
  }

  return sortByDate(result, filters.sortOrder);
};

export const calculateSessionStats = (sessions: Session[] | undefined) => ({
  total: sessions?.length ?? 0,
  scheduled: sessions?.filter((s) => s.status === "SCHEDULED").length ?? 0,
  completed: sessions?.filter((s) => s.status === "COMPLETED").length ?? 0,
  cancelled: sessions?.filter((s) => s.status === "CANCELLED").length ?? 0,
});

// src/components/sessions/session-list.tsx
export const SessionList = () => {
  const { sessions, stats, filters, updateFilter, isLoading, error } = useSessionList();

  if (isLoading) return <SessionListSkeleton />;
  if (error) return <ErrorAlert error={error} />;

  return (
    <div>
      <SessionStats stats={stats} />
      <SessionFilters filters={filters} onUpdate={updateFilter} />
      <SessionGrid sessions={sessions} />
    </div>
  );
};
```

**Améliorations**:
- Logique extraite dans un hook dédié
- Utilitaires réutilisables
- Composant principal focalisé sur le rendu
- Testabilité améliorée

---

## 3. Repository - Consolidation de Patterns

### Avant (Problématique)

```typescript
// Duplication dans chaque repository
export const sessionRepository = {
  findOne: async (db: PrismaClient, id: number) => {
    const result = await db.session.findUnique({
      where: { id },
      include: {
        tutor: true,
        student: true,
        activities: { include: { activity: true } },
      },
    });
    return result;
  },

  findMany: async (db: PrismaClient, tutorId: string) => {
    const results = await db.session.findMany({
      where: { tutorId },
      include: {
        tutor: true,
        student: true,
        activities: { include: { activity: true } },
      },
      orderBy: { scheduledAt: "desc" },
    });
    return results;
  },
};
```

### Après (Simplifié)

```typescript
// src/server/api/repositories/shared/session-includes.ts
export const SESSION_INCLUDES = {
  tutor: true,
  student: true,
  activities: { include: { activity: true } },
} as const;

// src/server/api/repositories/session.ts
export const sessionRepository = {
  findOne: (db: PrismaClient, id: number) =>
    db.session.findUnique({
      where: { id },
      include: SESSION_INCLUDES,
    }),

  findMany: (db: PrismaClient, filters: SessionFilters) =>
    db.session.findMany({
      where: buildSessionWhereClause(filters),
      include: SESSION_INCLUDES,
      orderBy: { scheduledAt: "desc" },
    }),

  findManyForTutor: (db: PrismaClient, tutorId: string) =>
    sessionRepository.findMany(db, { tutorId }),
};
```

**Améliorations**:
- Include constant réutilisable
- Méthodes plus concises
- Composition via méthodes internes

---

## 4. Gestion d'Erreurs - Centralisation

### Avant (Problématique)

```typescript
// Répété dans chaque service
const session = await sessionRepository.findOne(ctx.db, id);
if (!session) {
  throw new ResourceNotFoundError("Session", id.toString());
}

const user = await userRepository.findOne(ctx.db, userId);
if (!user) {
  throw new ResourceNotFoundError("User", userId);
}

const activity = await activityRepository.findOne(ctx.db, activityId);
if (!activity) {
  throw new ResourceNotFoundError("Activity", activityId.toString());
}
```

### Après (Simplifié)

```typescript
// src/server/lib/helpers/find-or-throw.ts
export const findOneOrThrow = async <T>(
  findFn: () => Promise<T | null>,
  entityName: string,
  id: string | number
): Promise<T> => {
  const entity = await findFn();
  if (!entity) {
    throw new ResourceNotFoundError(entityName, id.toString());
  }
  return entity;
};

// Usage dans les services
const session = await findOneOrThrow(
  () => sessionRepository.findOne(ctx.db, id),
  "Session",
  id
);

const user = await findOneOrThrow(
  () => userRepository.findOne(ctx.db, userId),
  "User",
  userId
);
```

**Améliorations**:
- Pattern centralisé et réutilisable
- Moins de duplication
- Comportement cohérent