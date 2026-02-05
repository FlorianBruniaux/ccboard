# Pattern: Composants React

## Conventions de Base

---

## 1. Structure de Composant

```typescript
// --- External ---
import { memo, useCallback, useMemo } from "react";

// --- UI Components ---
import { Button } from "~/components/ui/button";
import { Card } from "~/components/ui/card";
import { Typography } from "~/components/ui/typography";

// --- Internal ---
import { api } from "~/trpc/react";

// --- Types ---
import type { Session } from "@prisma/client";

type Props = {
  session: Session;
  onEdit: (id: number) => void;
};

export const SessionCard = memo(({ session, onEdit }: Props) => {
  const handleEdit = useCallback(() => {
    onEdit(session.id);
  }, [session.id, onEdit]);

  return (
    <Card>
      <Typography variant="h3">{session.title}</Typography>
      <Button onClick={handleEdit}>Modifier</Button>
    </Card>
  );
});

SessionCard.displayName = "SessionCard";
```

---

## 2. Typography Obligatoire

```typescript
// TOUJOURS - Composant Typography
import { Typography } from "~/components/ui/typography";

<Typography variant="h1">Titre Principal</Typography>
<Typography variant="h2">Sous-titre</Typography>
<Typography variant="sm" color="muted">Texte secondaire</Typography>
<Typography variant="body">Contenu principal</Typography>

// JAMAIS - Classes Tailwind directes pour typographie
<h1 className="text-2xl font-bold">Titre</h1>
<p className="text-sm text-muted-foreground">Texte</p>
```

---

## 3. Props Typing

```typescript
// BIEN - Type dédié
type SessionCardProps = {
  session: Session;
  onEdit: (id: number) => void;
  isLoading?: boolean;
};

export const SessionCard = ({ session, onEdit, isLoading = false }: SessionCardProps) => {
  // ...
};

// ÉVITER - Props inline complexes
export const SessionCard = ({
  session,
  onEdit,
  isLoading,
}: {
  session: Session;
  onEdit: (id: number) => void;
  isLoading?: boolean;
}) => {
  // ...
};
```

---

## 4. Hooks de Performance

### useCallback pour Handlers

```typescript
// BIEN - callbacks mémoïsés
const handleSubmit = useCallback(async (values: FormValues) => {
  await mutation.mutateAsync(values);
}, [mutation]);

const handleDelete = useCallback((id: number) => {
  deleteMutation.mutate({ id });
}, [deleteMutation]);

// ÉVITER - callbacks inline recréés à chaque render
<Button onClick={() => deleteMutation.mutate({ id: session.id })}>
  Supprimer
</Button>
```

### useMemo pour Calculs Coûteux

```typescript
// BIEN - valeurs mémoïsées
const filteredSessions = useMemo(
  () => sessions.filter((s) => s.status === selectedStatus),
  [sessions, selectedStatus]
);

const sortedData = useMemo(
  () => [...data].sort((a, b) => a.date.localeCompare(b.date)),
  [data]
);

// ÉVITER - recalcul à chaque render
const filteredSessions = sessions.filter((s) => s.status === selectedStatus);
```

### memo pour Composants

```typescript
// BIEN - composants mémoïsés quand pertinent
export const SessionCard = memo(({ session, onEdit }: Props) => {
  // Rendu coûteux ou props stables
});

SessionCard.displayName = "SessionCard";
```

---

## 5. Gestion d'État avec tRPC

```typescript
// Pattern standard tRPC + React Query
const SessionList = () => {
  const { data, isLoading, error } = api.session.getAll.useQuery();

  const deleteMutation = api.session.delete.useMutation({
    onSuccess: () => {
      appToast.success("Session supprimée");
      // Invalidation automatique via tRPC
    },
    onError: (error) => {
      appToast.error(error);
    },
  });

  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorDisplay error={error} />;

  return (
    <div>
      {data?.map((session) => (
        <SessionCard
          key={session.id}
          session={session}
          onDelete={deleteMutation.mutate}
        />
      ))}
    </div>
  );
};
```

---

## 6. Gestion des États de Chargement

```typescript
// Pattern standard
const Component = () => {
  const { data, isLoading, isError, error } = api.entity.get.useQuery();

  // Early returns clairs
  if (isLoading) {
    return <Skeleton className="h-[200px] w-full" />;
  }

  if (isError) {
    return <ErrorAlert message={error.message} />;
  }

  if (!data) {
    return <EmptyState message="Aucune donnée" />;
  }

  // Rendu principal
  return <DataDisplay data={data} />;
};
```

---

## 7. Formulaires avec React Hook Form

```typescript
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

const formSchema = z.object({
  title: z.string().min(1, "Titre requis"),
  description: z.string().optional(),
});

type FormValues = z.infer<typeof formSchema>;

const SessionForm = ({ onSubmit }: { onSubmit: (data: FormValues) => void }) => {
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      title: "",
      description: "",
    },
  });

  const handleSubmit = form.handleSubmit(onSubmit);

  return (
    <form onSubmit={handleSubmit}>
      <FormField
        control={form.control}
        name="title"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Titre</FormLabel>
            <FormControl>
              <Input {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
      <Button type="submit">Enregistrer</Button>
    </form>
  );
};
```

---

## 8. Patterns de Simplification

### Extraction de Logique

```typescript
// AVANT - logique dans le composant
const SessionList = () => {
  const [filter, setFilter] = useState("all");
  const { data } = api.session.getAll.useQuery();

  const filteredData = data?.filter((s) => {
    if (filter === "all") return true;
    return s.status === filter;
  });

  const stats = {
    total: data?.length ?? 0,
    active: data?.filter((s) => s.status === "STARTED").length ?? 0,
    // ... plus de calculs
  };

  // ... rendu
};

// APRÈS - hook dédié
const useSessionList = () => {
  const [filter, setFilter] = useState<SessionFilter>("all");
  const { data, isLoading, error } = api.session.getAll.useQuery();

  const filteredData = useMemo(() => {
    if (!data) return [];
    if (filter === "all") return data;
    return data.filter((s) => s.status === filter);
  }, [data, filter]);

  const stats = useMemo(() => ({
    total: data?.length ?? 0,
    active: data?.filter((s) => s.status === "STARTED").length ?? 0,
  }), [data]);

  return { filteredData, stats, filter, setFilter, isLoading, error };
};

const SessionList = () => {
  const { filteredData, stats, isLoading } = useSessionList();
  // Composant simplifié
};
```

---

## Checklist Composant React

- [ ] Arrow function avec memo si pertinent
- [ ] displayName défini
- [ ] Props typées avec type dédié
- [ ] Typography pour tout le texte
- [ ] useCallback pour les handlers
- [ ] useMemo pour les calculs coûteux
- [ ] Gestion explicite loading/error/empty states
