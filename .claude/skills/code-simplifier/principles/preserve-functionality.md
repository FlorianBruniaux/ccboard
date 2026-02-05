# Principe: Préserver la Fonctionnalité

## Règle Fondamentale

> Ne JAMAIS modifier CE QUE le code fait, seulement COMMENT il le fait.

Ce principe est **NON-NÉGOCIABLE**. La simplification de code est un exercice de refactoring, pas de réécriture fonctionnelle.

---

## Définition Précise

### Ce qui doit rester IDENTIQUE

1. **Valeurs de retour** - Mêmes outputs pour mêmes inputs
2. **Effets de bord** - Mêmes mutations, appels API, logs
3. **Exceptions levées** - Mêmes erreurs dans mêmes conditions
4. **Ordre d'exécution** - Si l'ordre impacte le résultat
5. **Types exposés** - Signatures publiques inchangées

### Ce qui PEUT changer

1. **Structure interne** - Découpage en sous-fonctions
2. **Nommage interne** - Variables locales (non exportées)
3. **Organisation du code** - Early returns, réduction d'imbrication
4. **Performance** - Si amélioration sans changement comportemental

---

## Méthodologie de Vérification

### Avant Simplification

```typescript
// 1. Documenter le comportement actuel
// Input: { userId: "123", status: "active" }
// Output attendu: User avec sessions actives
// Effets: Log dans Sentry si user non trouvé
```

### Après Simplification

```typescript
// 2. Vérifier que le comportement est IDENTIQUE
// - Mêmes tests passent
// - Mêmes logs produits
// - Mêmes erreurs levées
```

---

## Cas Particuliers

### Fonctions Pures

Plus facile à simplifier - vérifier uniquement les retours:

```typescript
// Avant
const calculateTotal = (items: Item[]) => {
  let total = 0;
  for (const item of items) {
    total = total + item.price * item.quantity;
  }
  return total;
};

// Après - COMPORTEMENT IDENTIQUE
const calculateTotal = (items: Item[]) =>
  items.reduce((acc, item) => acc + item.price * item.quantity, 0);
```

### Fonctions avec Effets de Bord

Plus délicat - vérifier TOUS les effets:

```typescript
// Avant
const createSession = async (data: SessionInput) => {
  const session = await db.session.create({ data });
  await sendNotification(session.tutorId);  // Effet de bord 1
  logger.info("Session created", { id: session.id });  // Effet de bord 2
  return session;
};

// Après - TOUS LES EFFETS PRÉSERVÉS
const createSession = async (data: SessionInput) => {
  const session = await db.session.create({ data });

  // Effets de bord préservés dans le même ordre
  await sendNotification(session.tutorId);
  logger.info("Session created", { id: session.id });

  return session;
};
```

---

## Signaux d'Alerte

### STOP si tu observes:

- Tests existants qui échouent après modification
- Nouveaux types d'erreurs possibles
- Changement dans l'ordre des opérations async
- Modification de types exportés/publics
- Suppression de validations ou guards

### Action en cas de doute

1. **Ne pas modifier**
2. Demander clarification à l'utilisateur
3. Proposer des tests pour valider le comportement

---

## Exemples de Violations

### INTERDIT - Changement de comportement

```typescript
// Avant - retourne null si non trouvé
const findUser = async (id: string) => {
  return db.user.findUnique({ where: { id } });
};

// VIOLATION - lance une erreur au lieu de null
const findUser = async (id: string) => {
  const user = await db.user.findUnique({ where: { id } });
  if (!user) throw new ResourceNotFoundError("User", id);
  return user;
};
```

### AUTORISÉ - Même comportement, code plus clair

```typescript
// Avant
const findUser = async (id: string) => {
  const u = await db.user.findUnique({ where: { id } });
  return u;
};

// AUTORISÉ - même comportement, meilleur nommage
const findUser = async (id: string) => {
  const user = await db.user.findUnique({ where: { id } });
  return user;
};
```
