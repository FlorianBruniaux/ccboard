# Pattern: TypeScript Méthode Aristote

## Conventions Obligatoires

Ces conventions sont **NON-NÉGOCIABLES** pour tout code du projet.

---

## 1. Fonctions: Arrow Functions Uniquement

```typescript
// TOUJOURS
const myFunction = () => {};
const myAsyncFunction = async () => {};
const myFunctionWithParams = (param: string): string => param.toUpperCase();

// JAMAIS
function myFunction() {}
async function myAsyncFunction() {}
```

### Composants React

```typescript
// TOUJOURS
const MyComponent = () => {
  return <div>Content</div>;
};

const MyComponentWithProps = ({ title }: { title: string }) => {
  return <h1>{title}</h1>;
};

// JAMAIS
function MyComponent() {
  return <div>Content</div>;
}
```

---

## 2. Imports: Organisation et Types Séparés

### Structure d'Imports

```typescript
// --- External ---
import { useState, useCallback } from "react";
import { z } from "zod";

// --- UI Components ---
import { Button } from "~/components/ui/button";
import { Card } from "~/components/ui/card";
import { Typography } from "~/components/ui/typography";

// --- Internal ---
import { api } from "~/trpc/react";
import { appToast } from "~/lib/appToast";

// --- Types ---
import type { User, Session } from "@prisma/client";
import type { SessionWithRelations } from "~/types";
```

### import type Obligatoire

```typescript
// TOUJOURS pour les types
import type { User } from "@prisma/client";
import type { FC, ReactNode } from "react";

// JAMAIS types mélangés avec runtime
import { User } from "@prisma/client"; // Si User est un type
```

---

## 3. Zod Schemas > Enums TypeScript

### Définition

```typescript
// TOUJOURS - Zod schemas
import { UserRoleSchema, SessionStatusSchema } from "~/server/db/prisma/zod";

const role = UserRoleSchema.Values.ADMIN;
const status = SessionStatusSchema.Values.SCHEDULED;

// Validation
const validRole = UserRoleSchema.parse(input);

// JAMAIS - enums TypeScript
enum UserRole {
  ADMIN = "ADMIN",
  TUTOR = "TUTOR",
}
```

### Utilisation

```typescript
// Type inference depuis Zod
type UserRole = z.infer<typeof UserRoleSchema>;

// Comparaison
if (user.role === UserRoleSchema.Values.ADMIN) {
  // ...
}

// Switch exhaustif
const getRoleLabel = (role: UserRole): string => {
  switch (role) {
    case UserRoleSchema.Values.ADMIN:
      return "Administrateur";
    case UserRoleSchema.Values.TUTOR:
      return "Tuteur";
    case UserRoleSchema.Values.STUDENT:
      return "Étudiant";
    // ... autres cas
  }
};
```

---

## 4. Types > Interfaces

```typescript
// PRÉFÉRÉ
type UserWithSessions = User & {
  sessions: Session[];
};

type Props = {
  title: string;
  onSubmit: () => void;
};

// ÉVITER (sauf extension nécessaire)
interface UserWithSessions extends User {
  sessions: Session[];
}
```

---

## 5. Nommage de Fichiers: kebab-case

```
// TOUJOURS
session-card.tsx
use-active-filters.ts
user-service.ts
session-repository.ts

// JAMAIS
SessionCard.tsx
useActiveFilters.ts
userService.ts
```

---

## 6. Typage Explicite

### Retours de Fonction

```typescript
// BIEN - type de retour explicite pour fonctions exportées
export const getUserById = async (id: string): Promise<User | null> => {
  return db.user.findUnique({ where: { id } });
};

// OK - inférence pour fonctions internes simples
const formatName = (user: User) => `${user.firstName} ${user.lastName}`;
```

### Éviter any

```typescript
// JAMAIS
const processData = (data: any) => { /* ... */ };

// TOUJOURS - typage explicite ou unknown
const processData = (data: unknown) => {
  if (isValidData(data)) {
    // data est typé après le guard
  }
};
```

---

## 7. Optional Chaining et Nullish Coalescing

```typescript
// BIEN
const userName = user?.profile?.displayName ?? "Anonyme";
const count = data?.items?.length ?? 0;

// ÉVITER
const userName = user && user.profile && user.profile.displayName
  ? user.profile.displayName
  : "Anonyme";
```

---

## 8. Destructuring Approprié

```typescript
// BIEN - destructuring clair
const { firstName, lastName, email } = user;
const { data, isLoading, error } = api.user.getById.useQuery({ id });

// ÉVITER - destructuring trop profond
const { data: { user: { profile: { settings } } } } = response;

// PRÉFÉRER
const settings = response.data?.user?.profile?.settings;
```

---

## 9. Async/Await > .then()

```typescript
// TOUJOURS
const fetchUser = async (id: string) => {
  const user = await db.user.findUnique({ where: { id } });
  return user;
};

// ÉVITER
const fetchUser = (id: string) => {
  return db.user.findUnique({ where: { id } }).then((user) => user);
};
```

---

## 10. Const Assertions

```typescript
// Pour les objets de configuration
const STATUS_COLORS = {
  SCHEDULED: "blue",
  STARTED: "green",
  COMPLETED: "gray",
} as const;

// Type inféré précisément
type StatusColor = typeof STATUS_COLORS[keyof typeof STATUS_COLORS];
// "blue" | "green" | "gray"
```

---

## Checklist de Conformité

- [ ] Arrow functions uniquement (pas de `function`)
- [ ] `import type` pour les imports de types
- [ ] Zod schemas au lieu d'enums TypeScript
- [ ] Fichiers en kebab-case
- [ ] Types de retour explicites pour fonctions exportées
- [ ] Pas de `any` (utiliser `unknown` si nécessaire)
- [ ] Imports organisés par catégorie avec commentaires