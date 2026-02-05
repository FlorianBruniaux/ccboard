# Gestion de Sessions

## Définition

Mécanisme permettant de maintenir l'état d'authentification d'un utilisateur à travers plusieurs requêtes HTTP.

## Sévérité

🔴 **CRITIQUE** - Session hijacking, fixation, vol de compte

## Génération de Session ID

### Exigences

**Cryptographiquement Sécurisé**
```
- Générateur de nombres aléatoires cryptographique
- Minimum 128 bits d'entropie (256 bits recommandé)
- Imprévisible et non-séquentiel
- Unique
```

**Format**
```
✅ Bon : 32+ caractères hexadécimaux aléatoires
❌ Mauvais : Incrémental, timestamp, hash(username)
```

### Longueur Minimale
- **128 bits minimum** (32 caractères hex)
- **256 bits recommandé** (64 caractères hex)

## Stockage des Sessions

### Côté Serveur (Recommandé)

**Stockage**
```
Options :
- En mémoire (RAM)
- Base de données
- Redis / Memcached
- Filesystem (moins performant)
```

**Données Stockées**
```
- User ID
- Timestamp création
- Timestamp dernière activité
- IP address (optionnel)
- User agent (optionnel)
```

### Côté Client

**Cookies**
```
Attributs obligatoires :
- HttpOnly (JavaScript ne peut pas lire)
- Secure (HTTPS uniquement)
- SameSite=Lax ou Strict (protection CSRF)
- Expires/Max-Age approprié
```

**JWT (Stateless)**
```
Avantages :
- Pas de stockage serveur
- Scalabilité horizontale

Inconvénients :
- Difficile à invalider
- Taille plus importante
- Exposé aux attaques si mal utilisé
```

## Configuration Cookie

### Attributs Sécurisés

```
Set-Cookie: sessionid=<value>;
  HttpOnly;
  Secure;
  SameSite=Lax;
  Path=/;
  Max-Age=3600;
  Domain=example.com
```

**HttpOnly**
- Empêche accès JavaScript
- Protection contre XSS

**Secure**
- Transmission HTTPS uniquement
- Jamais sur HTTP

**SameSite**
- **Strict** : Cookie jamais envoyé cross-site
- **Lax** : Cookie sur navigation GET top-level
- **None** : Cookie toujours envoyé (nécessite Secure)

**Path**
- Limiter scope du cookie
- Généralement /

**Domain**
- Limiter aux domaines nécessaires
- Éviter wildcards si pas nécessaire

**Max-Age / Expires**
- Durée de vie limitée
- 15-30 minutes pour sessions sensibles
- 1-7 jours maximum pour sessions normales

## Session Lifecycle

### Création

**Après Login Réussi**
```
1. Générer nouveau session ID sécurisé
2. Stocker session côté serveur
3. Envoyer cookie avec attributs sécurisés
4. Log événement
```

**Régénération ID**
```
Régénérer après :
- Login
- Changement de privilèges
- Actions sensibles
```

### Validation

**Chaque Requête**
```
1. Vérifier session ID valide
2. Vérifier non expirée
3. Vérifier non révoquée
4. Mettre à jour last activity
5. Vérifier IP/User-Agent (optionnel, strict)
```

### Expiration

**Types d'Expiration**
```
Absolute timeout :
- Durée maximum session (ex: 24h)
- Reconnexion obligatoire après

Idle timeout :
- Inactivité maximum (ex: 30 min)
- Renouvellement si activité

Sliding expiration :
- Extension à chaque activité
- Jusqu'à absolute timeout
```

**Valeurs Recommandées**
```
Sessions normales :
- Idle : 30 minutes
- Absolute : 24 heures

Sessions sensibles :
- Idle : 10-15 minutes
- Absolute : 2-4 heures

Remember me :
- 30 jours maximum
- Avec token séparé
```

### Destruction

**Logout**
```
1. Invalider session côté serveur
2. Supprimer cookie
3. Invalider refresh tokens
4. Log événement
```

**Automatic Logout**
```
Après :
- Timeout expiré
- Détection activité suspecte
- Changement de password
- Demande utilisateur (autre device)
```

## Protection Session Fixation

### Principe d'Attaque

```
1. Attaquant obtient session ID valide
2. Victime utilise ce session ID
3. Victime se connecte
4. Attaquant utilise le même session ID (maintenant authentifié)
```

### Mitigation

**Régénérer Session ID**
```
Régénérer après :
✅ Login (essentiel)
✅ Changement privilèges
✅ Actions sensibles
```

**Invalider Anciennes Sessions**
```
Après login :
- Invalider session anonyme précédente
- Créer nouvelle session authentifiée
```

## Protection Session Hijacking

### Mesures de Sécurité

**Transport Sécurisé**
- HTTPS obligatoire
- HSTS activé
- Pas de cookies sur HTTP

**Bind Session**
```
Optionnel mais recommandé :
- IP address (attention proxies/NAT)
- User-Agent (attention updates)
- Fingerprinting

Balance : Sécurité vs Usability
```

**Détection Anomalies**
```
Alertes si :
- Changement IP drastique
- Changement User-Agent
- Géolocalisation impossible
- Activité inhabituelle
```

**Revalidation**
```
Demander password pour :
- Actions sensibles
- Changement email/password
- Ajout méthode paiement
- Accès données sensibles
```

## Sessions Concurrentes

### Gestion Multiple Devices

**Options**
```
1. Illimité (moins sécurisé)
2. Limité (ex: 5 devices actifs)
3. Single session (plus sécurisé)
```

**Notification**
```
Notifier utilisateur :
- Nouveau login depuis device inconnu
- Liste devices actifs
- Option déconnecter tous
```

## Session Storage

### Redis (Recommandé)

**Avantages**
- Rapide (in-memory)
- TTL automatique
- Scalabilité
- Atomic operations

**Configuration**
```
Expiration automatique
Persistence pour durabilité
Replication pour HA
```

### Database

**Avantages**
- Persistence
- Requêtes complexes

**Considérations**
- Performance moindre
- Cleanup régulier nécessaire

### In-Memory

**Avantages**
- Plus rapide

**Inconvénients**
- Perdu au restart
- Pas de scalabilité horizontale
- Limite mémoire

## Checklist d'Audit

### Génération
- [ ] Session ID cryptographiquement sécurisé ?
- [ ] Longueur suffisante (128+ bits) ?
- [ ] Imprévisible ?
- [ ] Unique ?

### Cookies
- [ ] HttpOnly activé ?
- [ ] Secure activé ?
- [ ] SameSite configuré (Lax/Strict) ?
- [ ] Max-Age/Expires approprié ?
- [ ] Path et Domain correctement configurés ?

### Lifecycle
- [ ] Régénération après login ?
- [ ] Timeout idle configuré ?
- [ ] Timeout absolute configuré ?
- [ ] Invalidation au logout ?

### Sécurité
- [ ] HTTPS obligatoire ?
- [ ] Protection session fixation ?
- [ ] Détection anomalies ?
- [ ] Gestion sessions concurrentes ?

### Storage
- [ ] Stockage serveur sécurisé ?
- [ ] Pas de données sensibles côté client ?
- [ ] Cleanup sessions expirées ?

## Erreurs Courantes

### ❌ Session ID Prévisible
UUID v1 (timestamp), séquentiel, hash simple

### ❌ Session ID dans URL
Leaks dans referrer, logs, historique

### ❌ Pas de Régénération
Session fixation possible

### ❌ Timeout Trop Long
Sessions zombies, risque hijacking

### ❌ Pas de HttpOnly
XSS peut voler session

### ❌ Pas de Secure
MITM peut intercepter

### ❌ Session Fixation
Ne pas accepter session ID fourni par utilisateur

## Références

- **OWASP** : Session Management Cheat Sheet
- **OWASP** : Testing for Session Management
- **CWE-384** : Session Fixation
- **CWE-613** : Insufficient Session Expiration
