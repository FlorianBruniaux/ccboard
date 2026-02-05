# Pattern: Architecture 3-Tier

## Règle Fondamentale

```
Client → tRPC → Router (validation) → Service (business) → Repository (data) → Prisma
```

Cette séparation est **STRICTE**. Ne jamais mélanger les responsabilités.

---

## Router Layer

### Localisation
`src/server/api/routers/*.ts`

### Responsabilités AUTORISÉES

```typescript
export const sessionRouter = createTRPCRouter({
  // Définition de procédure tRPC
  create: protectedProcedure
    // Validation Zod
    .input(CreateSessionSchema)
    // Mutation avec délégation au service
    .mutation(({ ctx, input }) => sessionService.create(ctx, input)),

  getById: protectedProcedure
    .input(z.object({ id: z.number() }))
    .query(({ ctx, input }) => sessionService.getById(ctx, input.id)),
});
```

### Responsabilités INTERDITES

```typescript
// INTERDIT - Business logic dans router
create: protectedProcedure
  .input(CreateSessionSchema)
  .mutation(async ({ ctx, input }) => {
    // PAS de permission check ici
    if (!canCreate(ctx.session.user)) {
      throw new Error("Unauthorized");
    }

    // PAS d'accès direct à ctx.db
    const session = await ctx.db.session.create({ data: input });

    // PAS de logique métier
    await sendNotification(session);

    return session;
  }),
```

---

## Service Layer

### Localisation
`src/server/api/services/*.ts`

### Responsabilités AUTORISÉES

```typescript
export const sessionService = {
  create: async (ctx: ProtectedContext, input: CreateSessionInput) => {
    // Permission enforcement
    await enforcePermission(ctx, "SESSION", "CREATE");

    // Business logic
    const validatedInput = validateBusinessRules(input);

    // Orchestration multi-repository
    const session = await sessionRepository.createOne(ctx.db, validatedInput);

    // Effets de bord métier
    await notificationService.sendSessionCreated(session);

    return session;
  },

  updateStatus: async (
    ctx: ProtectedContext,
    id: number,
    newStatus: SessionStatus
  ) => {
    // Récupération via repository
    const session = await sessionRepository.findOne(ctx.db, id);

    if (!session) {
      throw new ResourceNotFoundError("Session", id.toString());
    }

    // Validation transition de statut (business rule)
    validateStatusTransition(session.status, newStatus);

    // Mise à jour via repository
    const updated = await sessionRepository.updateOne(ctx.db, id, {
      status: newStatus,
    });

    // Audit trail
    await sessionHistoryRepository.createOne(ctx.db, {
      sessionId: id,
      fromStatus: session.status,
      toStatus: newStatus,
      changedBy: ctx.session.userId,
    });

    return updated;
  },
};
```

### Responsabilités INTERDITES

```typescript
// INTERDIT - Appels Prisma directs
create: async (ctx: ProtectedContext, input: CreateSessionInput) => {
  // PAS d'appel direct à ctx.db
  return ctx.db.session.create({ data: input });
};
```

---

## Repository Layer

### Localisation
`src/server/api/repositories/*.ts`

### Responsabilités AUTORISÉES

```typescript
export const sessionRepository = {
  // CRUD standard
  findOne: (db: PrismaClient, id: number) =>
    db.session.findUnique({
      where: { id },
      include: DATA_TO_INCLUDE_SESSION,
    }),

  findMany: (db: PrismaClient, filters: SessionFilters) =>
    db.session.findMany({
      where: buildWhereClause(filters),
      include: DATA_TO_INCLUDE_SESSION,
      orderBy: { scheduledAt: "desc" },
    }),

  createOne: (db: PrismaClient, data: CreateSessionInput) =>
    db.session.create({
      data,
      include: DATA_TO_INCLUDE_SESSION,
    }),

  updateOne: (db: PrismaClient, id: number, data: UpdateSessionInput) =>
    db.session.update({
      where: { id },
      data,
      include: DATA_TO_INCLUDE_SESSION,
    }),

  deleteOne: (db: PrismaClient, id: number) =>
    db.session.delete({ where: { id } }),

  // Pagination
  getAllPaginated: (db: PrismaClient, params: PaginationParams) =>
    db.session.findMany({
      skip: params.offset,
      take: params.limit,
      include: DATA_TO_INCLUDE_SESSION,
    }),
};
```

### Responsabilités INTERDITES

```typescript
// INTERDIT - Business logic dans repository
findActiveForTutor: async (db: PrismaClient, tutorId: string) => {
  const sessions = await db.session.findMany({
    where: { tutorId, status: "SCHEDULED" },
  });

  // PAS de logique métier
  return sessions.filter((s) => isWithinBusinessHours(s));
};

// INTERDIT - Permission checks
findOne: async (db: PrismaClient, id: number, userId: string) => {
  const session = await db.session.findUnique({ where: { id } });

  // PAS de vérification d'accès
  if (session.tutorId !== userId) {
    throw new Error("Unauthorized");
  }

  return session;
};
```

---

## Patterns de Simplification

### Extraction vers Service

```typescript
// AVANT - logique dans router
create: protectedProcedure
  .input(CreateSessionSchema)
  .mutation(async ({ ctx, input }) => {
    if (input.type === "SUPERVISED" && !input.tutorId) {
      throw new Error("Tutor required");
    }
    const session = await ctx.db.session.create({ data: input });
    await ctx.db.sessionHistory.create({ /* ... */ });
    return session;
  }),

// APRÈS - délégué au service
create: protectedProcedure
  .input(CreateSessionSchema)
  .mutation(({ ctx, input }) => sessionService.create(ctx, input)),
```

### Extraction vers Repository

```typescript
// AVANT - Prisma direct dans service
const sessions = await ctx.db.session.findMany({
  where: { tutorId, status: "SCHEDULED" },
  include: {
    student: true,
    activities: { include: { activity: true } },
  },
  orderBy: { scheduledAt: "asc" },
});

// APRÈS - via repository
const sessions = await sessionRepository.findManyForTutor(ctx.db, tutorId);
```

---

## Checklist de Conformité 3-Tier

### Router
- [ ] Que des définitions de procédures tRPC
- [ ] Que de la validation Zod
- [ ] Délégation systématique au service
- [ ] Aucun `ctx.db` direct

### Service
- [ ] Permission checks avec `enforcePermission()`
- [ ] Business logic et validation métier
- [ ] Appels via repositories uniquement
- [ ] Gestion des erreurs avec ProductionError

### Repository
- [ ] CRUD pur (findOne, findMany, createOne, updateOne, deleteOne)
- [ ] Aucune logique métier
- [ ] Aucune permission check
- [ ] Include constants réutilisables