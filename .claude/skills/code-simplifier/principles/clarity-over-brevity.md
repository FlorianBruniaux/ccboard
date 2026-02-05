# Principe: Clarté > Concision

## Règle Fondamentale

> La lisibilité du code prime TOUJOURS sur sa brièveté.

Un code plus long mais compréhensible est préférable à un code court mais obscur.

---

## Définition

### Clarté = Compréhension Immédiate

- Le code se lit comme une histoire
- L'intention est évidente sans commentaires
- Un nouveau développeur comprend en < 30 secondes

### Concision ≠ Qualité

- Moins de lignes n'est pas un objectif
- "Clever code" est souvent du code illisible
- La compression excessive crée de la dette technique

---

## Applications Concrètes

### Nommage Explicite

```typescript
// MAL - concis mais obscur
const p = users.filter(u => u.r === "T" && u.a);
const d = new Date().toISOString().split("T")[0];

// BIEN - clair et explicite
const activeTutors = users.filter(
  (user) => user.role === "TUTOR" && user.isActive
);
const todayDate = new Date().toISOString().split("T")[0];
```

### Éviter les One-Liners Complexes

```typescript
// MAL - one-liner illisible
const result = data?.items?.filter(i => i.status === "active")?.map(i => ({ ...i, processed: true }))?.reduce((a, b) => ({ ...a, [b.id]: b }), {}) ?? {};

// BIEN - étapes explicites
const activeItems = data?.items?.filter(
  (item) => item.status === "active"
) ?? [];

const processedItems = activeItems.map((item) => ({
  ...item,
  processed: true,
}));

const itemsById = processedItems.reduce(
  (acc, item) => ({ ...acc, [item.id]: item }),
  {}
);
```

### Ternaires Simples Uniquement

```typescript
// MAL - ternaires imbriqués
const message = isLoading
  ? "Chargement..."
  : hasError
    ? error.code === 404
      ? "Non trouvé"
      : "Erreur serveur"
    : data
      ? `${data.count} résultats`
      : "Aucun résultat";

// BIEN - fonction explicite
const getMessage = (): string => {
  if (isLoading) return "Chargement...";

  if (hasError) {
    return error.code === 404 ? "Non trouvé" : "Erreur serveur";
  }

  if (!data) return "Aucun résultat";

  return `${data.count} résultats`;
};
```

---

## Patterns de Clarté

### Early Returns

```typescript
// AVANT - imbrication profonde
const processSession = (session: Session) => {
  if (session) {
    if (session.status === "SCHEDULED") {
      if (session.tutor) {
        return startSession(session);
      }
    }
  }
  return null;
};

// APRÈS - early returns clairs
const processSession = (session: Session | null) => {
  if (!session) return null;
  if (session.status !== "SCHEDULED") return null;
  if (!session.tutor) return null;

  return startSession(session);
};
```

### Décomposition Logique

```typescript
// AVANT - fonction monolithique
const validateAndSubmitForm = async (values: FormValues) => {
  // 80 lignes de validation, transformation, soumission...
};

// APRÈS - responsabilités séparées
const validateFormValues = (values: FormValues): ValidationResult => {
  // Validation isolée
};

const transformToPayload = (values: FormValues): ApiPayload => {
  // Transformation isolée
};

const submitForm = async (payload: ApiPayload): Promise<Response> => {
  // Soumission isolée
};

const handleFormSubmit = async (values: FormValues) => {
  const validation = validateFormValues(values);
  if (!validation.success) {
    return { error: validation.errors };
  }

  const payload = transformToPayload(values);
  return submitForm(payload);
};
```

---

## Anti-Patterns à Éviter

### Compression Excessive

```typescript
// MAL
const f = (x: number) => x > 0 ? x * 2 : x < 0 ? x * -1 : 0;

// BIEN
const transformNumber = (value: number): number => {
  if (value > 0) return value * 2;
  if (value < 0) return Math.abs(value);
  return 0;
};
```

### Destructuring Excessif

```typescript
// MAL - destructuring illisible
const { data: { user: { profile: { settings: { theme } } } } } = response;

// BIEN - étapes claires
const userData = response.data?.user;
const userSettings = userData?.profile?.settings;
const theme = userSettings?.theme ?? "default";
```

### Chaînage Excessif

```typescript
// MAL
const result = await Promise.all(ids.map(id => fetch(`/api/${id}`)))
  .then(responses => Promise.all(responses.map(r => r.json())))
  .then(data => data.filter(d => d.active).map(d => d.name));

// BIEN
const fetchAllData = async (ids: string[]): Promise<string[]> => {
  const responses = await Promise.all(
    ids.map((id) => fetch(`/api/${id}`))
  );

  const data = await Promise.all(
    responses.map((response) => response.json())
  );

  const activeItems = data.filter((item) => item.active);
  return activeItems.map((item) => item.name);
};
```

---

## Règle des 30 Secondes

> Si un développeur expérimenté ne comprend pas une fonction en 30 secondes, elle est trop complexe.

**Actions correctives:**
1. Renommer pour expliciter l'intention
2. Extraire des sous-fonctions
3. Ajouter des early returns
4. Décomposer les expressions complexes