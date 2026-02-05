# Principe: Éviter la Sur-Ingénierie

## Règle Fondamentale

> Construire pour le besoin actuel, pas pour des hypothétiques futurs.

YAGNI (You Aren't Gonna Need It) + KISS (Keep It Simple, Stupid).

---

## Définition de la Sur-Ingénierie

### Signes d'Alerte

- Abstractions pour un seul cas d'usage
- Patterns de design sans justification claire
- Configurabilité excessive (options jamais utilisées)
- Généralisations prématurées
- "Au cas où on aurait besoin plus tard"

### Impact Négatif

- Complexité accrue = plus de bugs
- Maintenance difficile
- Onboarding ralenti
- Dette technique cachée

---

## Applications Concrètes

### Pas d'Abstraction Prématurée

```typescript
// SUR-INGÉNIERIE - factory pattern pour un seul type
const createNotificationFactory = () => ({
  create: (type: string, data: unknown) => {
    switch (type) {
      case "email":
        return new EmailNotification(data);
      // Un seul case utilisé en pratique
      default:
        throw new Error("Unknown type");
    }
  },
});

// SIMPLE - fonction directe
const sendEmailNotification = async (data: EmailData) => {
  return emailService.send(data);
};
```

### Pas de Configurabilité Inutile

```typescript
// SUR-INGÉNIERIE - options jamais utilisées
type SessionConfig = {
  maxDuration?: number;
  allowExtension?: boolean;
  extensionIncrement?: number;
  maxExtensions?: number;
  notifyOnExtension?: boolean;
  notificationTemplate?: string;
  // 10 autres options...
};

const createSession = (data: SessionInput, config: SessionConfig = {}) => {
  // Gestion complexe de toutes les options
};

// SIMPLE - comportement par défaut
const createSession = (data: SessionInput) => {
  return db.session.create({
    data: {
      ...data,
      maxDuration: 60, // Valeur métier fixe
    },
  });
};
```

### Pas de Wrapper Sans Valeur

```typescript
// SUR-INGÉNIERIE - wrapper inutile
class DatabaseWrapper {
  private db: PrismaClient;

  constructor() {
    this.db = new PrismaClient();
  }

  async findUser(id: string) {
    return this.db.user.findUnique({ where: { id } });
  }

  async createUser(data: UserInput) {
    return this.db.user.create({ data });
  }
  // Juste des proxies vers Prisma...
}

// SIMPLE - utilisation directe via Repository pattern existant
// Le repository EST déjà l'abstraction appropriée
const userRepository = {
  findOne: (id: string) => db.user.findUnique({ where: { id } }),
  createOne: (data: UserInput) => db.user.create({ data }),
};
```

---

## Règle des Trois Occurrences

> Abstraire seulement après 3 occurrences du même pattern.

### Première Occurrence

```typescript
// Juste implémenter directement
const formatUserName = (user: User) =>
  `${user.firstName} ${user.lastName}`;
```

### Deuxième Occurrence

```typescript
// Encore direct, noter la duplication
const formatTutorName = (tutor: Tutor) =>
  `${tutor.firstName} ${tutor.lastName}`;
```

### Troisième Occurrence → Abstraire

```typescript
// Maintenant on abstrait
type Person = { firstName: string; lastName: string };

const formatFullName = (person: Person) =>
  `${person.firstName} ${person.lastName}`;
```

---

## Architecture 3-Tier: Juste Assez

### Router (Validation uniquement)

```typescript
// BIEN - juste validation et délégation
export const sessionRouter = createTRPCRouter({
  create: protectedProcedure
    .input(CreateSessionSchema)
    .mutation(({ ctx, input }) => sessionService.create(ctx, input)),
});
```

### Service (Business logic)

```typescript
// BIEN - business logic sans sur-abstraction
export const sessionService = {
  create: async (ctx: ProtectedContext, input: CreateSessionInput) => {
    await enforcePermission(ctx, "SESSION", "CREATE");
    return sessionRepository.createOne(ctx.db, input);
  },
};
```

### Repository (CRUD)

```typescript
// BIEN - CRUD simple
export const sessionRepository = {
  createOne: (db: PrismaClient, data: CreateSessionInput) =>
    db.session.create({ data }),
};
```

---

## Anti-Patterns de Sur-Ingénierie

### Le "Enterprise Pattern"

```typescript
// SUR-INGÉNIERIE EXTRÊME
interface IUserService { /* ... */ }
interface IUserRepository { /* ... */ }
interface IUserValidator { /* ... */ }
interface IUserMapper { /* ... */ }

class UserServiceImpl implements IUserService {
  constructor(
    private readonly repository: IUserRepository,
    private readonly validator: IUserValidator,
    private readonly mapper: IUserMapper,
  ) {}
  // Injection de dépendances pour un seul consumer...
}

// RÉALITÉ MÉTHODE ARISTOTE - simple et fonctionnel
const userService = {
  getById: async (ctx: Context, id: string) => {
    await enforcePermission(ctx, "USER", "READ");
    return userRepository.findOne(id);
  },
};
```

### Le "Just In Case"

```typescript
// SUR-INGÉNIERIE - prévoir tous les cas
const processPayment = async (
  amount: number,
  currency: string = "EUR",
  provider: PaymentProvider = "stripe",
  retryCount: number = 3,
  retryDelay: number = 1000,
  webhookUrl?: string,
  metadata?: Record<string, unknown>,
  // ... encore 5 paramètres "au cas où"
) => { /* ... */ };

// SIMPLE - besoin actuel uniquement
const processPayment = async (amount: number) => {
  return stripe.charges.create({
    amount,
    currency: "eur", // Fixe pour notre marché
  });
};
```

---

## Checklist Avant Abstraction

- [ ] Ce pattern apparaît-il >= 3 fois ?
- [ ] L'abstraction réduit-elle vraiment la complexité ?
- [ ] Un autre développeur comprendra-t-il l'abstraction facilement ?
- [ ] Le besoin est-il concret et actuel (pas "potentiel") ?
- [ ] Le coût de l'abstraction est-il justifié par le gain ?

Si une seule réponse est "non" → **ne pas abstraire**.
