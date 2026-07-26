# Skill Usage Analytics, plan consolidé

Date : 2026-07-26
Statut : validé pour implémentation, à exécuter phase par phase

## Contexte

ccboard est le tableau de bord TUI et Web de Claude Code, il lit `~/.claude` pour exposer sessions, stats, config, hooks, agents, coûts et historique. Cette fonctionnalité ajoute une analytique d'usage des Skills : compter les invocations réelles de Skills Claude Code (jamais les simples mentions textuelles d'un nom de skill) sur l'ensemble des projets de la machine, à partir des transcripts JSONL sous `~/.claude/projects/`, avec une attribution fine des tokens et du coût par skill.

L'outil de référence est `cc-skill-usage`, un CLI Python autonome qui fonctionne déjà (leaderboard, détail par skill, liste récente, filtres, environ 380 lignes, 5.9s à froid et 0.07s à chaud sur environ 2000 sessions). ccboard ne cherche pas à le copier mais à le dépasser sur deux points qu'un CLI one-shot ne peut structurellement pas offrir : le croisement coût/tokens réel (ccboard a déjà un modèle de pricing, le script Python n'en a aucun) et la fraîcheur permanente des données grâce au file watcher existant de ccboard (un CLI reste, par nature, toujours aussi frais que sa dernière exécution).

Ce document synthétise une critique adversariale et trois analyses spécialisées indépendantes (system-architect, backend-architect, rust-ccboard) en un plan unique. Il tranche explicitement les quatre points où les analyses divergeaient, avant de dérouler le plan phase par phase.

## Arbitrages sur les points de désaccord

### 1. CACHE_VERSION : pas de bump

system-architect et backend-architect ont chacun lu le code réel de `metadata_cache.rs` et sont arrivés à la même conclusion : les `CREATE TABLE IF NOT EXISTS` s'exécutent sans condition à chaque ouverture, avant toute vérification de version. Le bloc de mismatch de version ne fait qu'un `DELETE FROM` ciblé sur une liste fixe de quatre tables existantes. Deux tables neuves sont donc purement additives et sans danger sur une base déjà peuplée. Le plan initial de rust-ccboard mentionnait encore un passage de `CACHE_VERSION` de 9 à 10 dans son tableau de fichiers Phase 1, cette ligne est fausse et annulée par la lecture de code vérifiée des deux architectes. Un bump de version n'apporterait aucun bénéfice ici et aurait un coût réel : il viderait le cache de métadonnées de sessions de chaque utilisateur, forçant un rescan complet à froid.

Décision : `CACHE_VERSION` reste à 9. Deux mesures de sécurité restent néanmoins obligatoires. D'abord, ajouter un `DELETE FROM` sur les deux nouvelles tables à l'intérieur du bloc existant de mismatch de version, pour qu'un futur bump sans lien avec les skills ne laisse pas de lignes orphelines. Ensuite, étendre `MetadataCache::clear()` pour qu'il vide aussi ces deux tables (la commande CLI `clear-cache` supprime déjà tout le fichier de base, ce cas est donc déjà couvert côté CLI).

### 2. Fraîcheur du watcher en Phase 0 : approche incrémentale

Deux designs concurrents pour le même problème. `update_session()` (store.rs:1381) reparse un seul fichier à chaque tick du file watcher mais ne touche jamais aux stats d'invocations, tandis que `compute_invocations()` (store.rs:1411) est un rescan complet de toutes les sessions, appelé une seule fois au démarrage. system-architect propose un design incrémental : une nouvelle map `invocation_by_path: RwLock<HashMap<PathBuf, InvocationStats>>` sur `DataStore`, et une fonction `refresh_invocations_for(path)` qui ne rescanne que le fichier modifié puis replie son résultat dans l'agrégat global. rust-ccboard propose une solution plus simple, appeler `compute_invocations()` en entier à chaque `update_session()`, tout en signalant lui-même le risque : une rafale de changements de fichiers (un `git checkout` qui touche 50 transcripts d'un coup) déclencherait autant de rescans complets de l'ensemble des sessions, pas seulement des fichiers modifiés.

Décision : le design incrémental de system-architect est retenu. Il ne se contente pas d'accepter le risque identifié par rust-ccboard comme une dette, il le résout directement. Le coût additionnel, un repli de HashMap en mémoire pour environ 2000 entrées, est négligeable comparé à un rescan disque complet à chaque tick.

### 3. Formule d'attribution coût/tokens : répartition proportionnelle

Les trois analyses s'accordent sur l'absence de lookahead en v1 (pas de report des tokens du tour suivant sur l'invocation). Sur la formule elle-même, backend-architect et rust-ccboard proposent, indépendamment l'un de l'autre, de réutiliser exactement le partage proportionnel déjà éprouvé dans `session_index.rs` (`message_tokens / tool_count_in_that_message`, le reste allant au premier bloc), appliqué au nom de skill précis lu dans `input.skill` plutôt qu'au littéral générique `"Skill"`. system-architect propose une alternative différente : attribuer l'intégralité des tokens du tour assistant au skill, sans aucun partage proportionnel, avec l'argument que c'est plus exact et plus auditable qu'une répartition estimée.

Décision : la répartition proportionnelle l'emporte, à deux voix contre une, mais pas seulement pour cette raison de majorité. Elle permet un test de cohérence peu coûteux et précieux, conçu par backend-architect : `test_parsers_agree_on_token_totals`, qui vérifie que la somme `tool_token_usage["Skill"]` déjà calculée par `session_index.rs` correspond exactement à la somme des parts de tokens par skill calculées par le nouveau code sur le même fichier. L'alternative de system-architect casserait cette réconciliation dès qu'un message contient plusieurs blocs `tool_use`, ce qui ferait perdre un filet de sécurité gratuit pour un gain de simplicité marginal. `test_parsers_agree_on_token_totals` est retenu comme test obligatoire de la Phase 1, quelle que soit la formule choisie, car il reste utile dans les deux cas.

### 4. Duplication du parsing d'usage entre `session_index.rs` et `invocations.rs` : dupliquer en v1, extraire plus tard

backend-architect propose d'extraire tout de suite un module partagé, `parsers/token_attribution.rs`, utilisé par les deux fichiers pour éliminer le risque que deux implémentations indépendantes de « comment lire l'usage d'une ligne » dérivent l'une de l'autre avec le temps. rust-ccboard propose d'accepter une petite duplication (environ 15 lignes), explicitement pour ne toucher à rien dans `session_index.rs`, une décision déjà verrouillée dans ce projet pour éviter tout risque de régression sur les consommateurs existants (Costs, Analytics, `plugin_usage.rs`).

Décision : en Phase 1, la duplication est acceptée, dans l'esprit de la garantie de zéro-modification sur `session_index.rs`. En contrepartie, `test_parsers_agree_on_token_totals` devient obligatoire comme filet de détection de dérive, ce qui rend la duplication sûre sans nécessiter le refactor immédiatement. L'extraction du module partagé est notée comme candidate de Phase 2 ou plus tard, avec un déclencheur explicite : si le test constate un jour une divergence entre les deux implémentations, ou si un troisième consommateur de cette logique apparaît, l'extraction doit être faite à ce moment-là, pas avant.

## Éléments convergents (non renégociés)

Les trois analyses s'accordent sur le reste, il est repris tel quel.

Les skills retirés ou renommés reçoivent un badge `[retired]` : l'historique d'invocation existe mais le skill n'est plus défini sur le disque. Aucun flag « defined » n'est jamais persisté, l'appartenance au catalogue sur disque est recalculée à l'affichage contre la liste de skills existants, ce qui donne quatre cas. Défini et utilisé, cas normal. Défini et jamais utilisé, l'onglet Agents l'affiche comme « unused ». Non défini et utilisé, l'onglet Skills affiche le badge `[retired]` avec une infobulle expliquant qu'il n'est plus défini sur le disque. Non défini et jamais utilisé, invisible. Le label « dead code » de l'onglet Agents est resserré pour ne se déclencher que lorsque le compte d'invocations est exactement zéro, ce qui met fin à la contradiction actuelle entre les deux onglets.

Les noms de skills sont espacés par namespace, par exemple `superpowers:brainstorming`, et doivent être stockés et groupés tels quels, sans jamais être découpés sur le caractère `:`. `superpowers:brainstorming` et un `brainstorming` local sont deux lignes distinctes. Un risque réel a été identifié à ce sujet : `classify_plugin()` dans `plugin_usage.rs` utilise aujourd'hui un test de sous-chaîne (`.contains()`), ce qui produirait de faux positifs entre namespaces différents. C'est noté comme correctif de Phase 2, en passant à une correspondance exacte d'abord, avec la sous-chaîne en repli seulement si l'exact ne matche rien.

Les nombres de fichiers touchés restent volontairement bas. Phase 0, deux fichiers. Phase 1, quatre à cinq fichiers. Phase 2, six fichiers, dont quatre sont de simples enregistrements d'une ligne (enum d'onglet, `tabs/mod.rs`, route Axum, navigation Leptos) imposés par la convention « onglet dédié avec parité TUI/Web ». Ce chiffrage répond directement à la critique adversariale initiale qui pointait un risque de sur-ingénierie au-delà de dix fichiers, il ne l'est pas.

Un bug préexistant et sans lien avec cette fonctionnalité a été découvert dans la regex de commande d'`invocations.rs` : `^/([a-z][a-z0-9-]*)` tronque les commandes namespacées comme `/boldguy:talks:bg-extract` en `/boldguy`. Ce bug est hors périmètre de ce plan, il sera déposé comme ticket séparé et ne doit pas être corrigé au passage, par discipline de scope.

## Plan consolidé

### Phase 0 : prérequis, sans code de skill

Objectif unique : rendre les statistiques d'invocation maintenables de façon incrémentale, avant d'ajouter la moindre ligne de code sur les skills.

| Fichier | Modification |
|---|---|
| `store.rs` | Ajout de `invocation_by_path: RwLock<HashMap<PathBuf, InvocationStats>>` sur `DataStore`. `compute_invocations()` peuple cette map fichier par fichier puis la replie dans l'agrégat existant. Nouvelle fonction `refresh_invocations_for(path)`, appelée depuis `update_session()`, qui rescanne uniquement le fichier modifié, remplace son entrée dans la map, puis replie de nouveau l'agrégat. |
| (test dédié) | `test_watcher_recompute_skills` : écrit un JSONL temporaire, appelle `update_session()`, vérifie que les stats sont vides ; ajoute une ligne `tool_use` de type Skill ou Task, rappelle `update_session()`, vérifie que l'agrégat a bougé, sans redémarrage du processus. |

Le verrou d'écriture n'est tenu que pendant l'échange de la map, conformément à l'ADR-001 (écriture courte, jamais tenue à travers un `await`).

Porte de sortie de Phase 0, tout doit passer avant d'ouvrir la branche Phase 1 :
1. Le test `test_watcher_recompute_skills` passe.
2. `cargo fmt --all && cargo clippy --all-targets && cargo test --all` est propre.
3. Test manuel : lancer la TUI, exécuter un skill dans un autre terminal, confirmer que le compteur de l'onglet Agents bouge sans redémarrage.

### Phase 1 : preuve par le CLI, sans interface

Objectif : extraire les invocations de skills des transcripts, les persister, calculer un leaderboard exact, exposer le tout par un sous-commande CLI, sans toucher à la TUI ni au Web.

Schéma SQL complet, construit dès la Phase 1 pour éviter une migration ultérieure :

```sql
CREATE TABLE IF NOT EXISTS skill_scan_inventory (
    path TEXT PRIMARY KEY, mtime INTEGER NOT NULL, scanned_at INTEGER NOT NULL,
    event_count INTEGER NOT NULL DEFAULT 0, parser_revision INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_skill_inv_mtime ON skill_scan_inventory(mtime);

CREATE TABLE IF NOT EXISTS skill_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT, path TEXT NOT NULL, skill TEXT NOT NULL,
    args TEXT, ts TEXT, day TEXT, project TEXT NOT NULL, is_subagent INTEGER NOT NULL DEFAULT 0,
    model TEXT, tokens_total INTEGER NOT NULL DEFAULT 0, tokens_input INTEGER NOT NULL DEFAULT 0,
    tokens_output INTEGER NOT NULL DEFAULT 0, tokens_cache_read INTEGER NOT NULL DEFAULT 0,
    tokens_cache_write INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_skill_events_path ON skill_events(path);
CREATE INDEX IF NOT EXISTS idx_skill_events_skill ON skill_events(skill);
CREATE INDEX IF NOT EXISTS idx_skill_events_day ON skill_events(day);
CREATE INDEX IF NOT EXISTS idx_skill_events_project ON skill_events(project);
```

Pas de `FOREIGN KEY`, `PRAGMA foreign_keys` n'est jamais activé dans ce projet. L'idempotence passe par un `DELETE FROM skill_events WHERE path = ?` suivi de l'insertion, dans la même transaction que l'upsert de l'inventaire. `parser_revision` permet à un futur changement de logique d'attribution de forcer un rescan ciblé sans toucher à `CACHE_VERSION`.

Formule d'attribution retenue (voir arbitrage 3). Pour chaque ligne assistant, `usage = line.usage.or(line.message.usage)`, `message_tokens` est le total. `tools` est la liste des noms de blocs `tool_use` dans `message.content`, dans l'ordre du tableau. `n = tools.len()`, `base = message_tokens / n`, `rem = message_tokens % n`. Le bloc d'indice i reçoit `base`, plus `rem` si i vaut 0. Pour un bloc nommé `Skill`, le crédit va au nom lu dans `input.skill`, jamais au littéral `"Skill"`, et une ligne `skill_events` est émise avec la part de tokens de ce bloc. C'est la règle exacte de numérateur, diviseur et reste déjà utilisée dans `session_index.rs`, seule la clé change. Le coût n'est jamais stocké en base, `calculate_cost(tokens, model)` est calculé à la lecture.

Fichiers Phase 1 :

| Fichier | Rôle |
|---|---|
| `models/invocations.rs` | Ajout de `SkillInvocationEvent`, `SkillLeaderboardEntry`, extension de `InvocationStats` avec `skill_events: Vec<SkillInvocationEvent>` |
| `parsers/invocations.rs` | Extension de `scan_session` pour émettre un `SkillInvocationEvent` par bloc `tool_use` de type Skill, avec partage proportionnel des tokens |
| `cache/metadata_cache.rs` | Nouvelles tables `skill_scan_inventory` et `skill_events`, invalidation par mtime, sans bump de `CACHE_VERSION` |
| `store.rs` | Accesseur `skill_leaderboard()`, coût calculé à la lecture |
| `ccboard/src/main.rs` + `cli.rs` | Sous-commande `ccboard skills [--json] [--since] [--project] [--include-subagents] [<skill>]`, sortie leaderboard uniquement |

Signatures de référence :

```rust
pub struct SkillInvocationEvent {
    pub skill: String, // verbatim, ex. "superpowers:brainstorming", jamais découpé sur ':'
    pub timestamp: Option<DateTime<Utc>>,
    pub project: ProjectId,
    pub session_path: PathBuf, // clé d'identité, jamais le sessionId du JSON
    pub is_subagent: bool,
    pub tokens: u64,
    pub model: Option<String>,
}

pub struct SkillLeaderboardEntry {
    pub skill: String, pub invocations: usize, pub projects: usize,
    pub total_tokens: u64, pub estimated_cost_usd: f64,
    pub first_seen: Option<DateTime<Utc>>, pub last_seen: Option<DateTime<Utc>>,
}

pub fn skill_leaderboard(&self) -> Vec<SkillLeaderboardEntry> // sur DataStore
```

Tests requis pour la Phase 1 :

| Test | Ce qu'il vérifie |
|---|---|
| `test_skill_event_records_tokens` | Un événement de skill enregistre correctement ses tokens |
| `test_prose_mention_not_counted` | Le texte « utilise le skill flow-lean » ne produit aucun événement |
| `test_subagent_path_dedup` | Un fichier parent et un `subagents/agent-x.jsonl` partageant le même sessionId dans leur contenu produisent bien deux événements distincts, sur deux chemins distincts |
| `test_multi_skill_same_message` | Plusieurs blocs Skill dans un même message assistant sont répartis correctement |
| `test_namespaced_skill_name` | `superpowers:brainstorming` est stocké et groupé sans être coupé sur `:` |
| `test_skill_invocations_roundtrip` | Écriture puis lecture en cache sans perte |
| `test_skill_invocations_migration_from_populated_v9_db` | Une base SQLite réaliste, préexistante et déjà peuplée de `session_metadata`, s'ouvre proprement avec le nouveau code, sans perte de lignes |
| `test_skill_leaderboard_aggregation` | Agrégation correcte multi-projets et multi-sessions |
| `test_parsers_agree_on_token_totals` | Obligatoire quelle que soit la formule (voir arbitrage 3), la somme `tool_token_usage["Skill"]` de `session_index.rs` égale la somme des parts calculées par le nouveau code sur le même fichier |
| `test_additive_schema_preserves_existing_rows` | Une base créée avec le DDL pré-changement, peuplée de N sessions, reste lisible à l'identique après réouverture avec le nouveau code |

Ordre de construction recommandé : structures de modèle, puis extension du parseur avec ses tests, puis table de cache avec le test de migration, puis accesseur leaderboard sur le store avec son test, puis sous-commande CLI, puis la porte de validation sur données réelles ci-dessous, puis un passage complet fmt/clippy/test avant commit.

Avant d'écrire le code des nouvelles tables, une vérification de 20 à 30 minutes est demandée : copier la base `~/.ccboard` réelle vers un chemin de test, appliquer uniquement le DDL additif, confirmer que les comptes de lignes de `session_metadata` et le temps de démarrage à chaud restent identiques.

Note d'effort honnête, reprise de rust-ccboard : les ajouts de structures et la plomberie CLI sont rapides. L'extension du parseur est simple à copier-adapter, mais obtenir les noms de champs exacts de `SessionLine.timestamp` et `.model` face aux bizarreries réelles des transcripts est l'endroit où une heure peut disparaître. Le SQL des nouvelles tables est du gabarit classique, mais le test de migration contre une base déjà peuplée est la partie la plus susceptible de surprendre, il faut une fixture v9 réaliste, pas une base vide. L'étape de validation sur données réelles n'est pas un travail en plus, c'est l'échéance elle-même : si le compte de `flow-lean` est faux, les coupables probables sont un double comptage de subagents ou des événements silencieusement perdus à cause de timestamps malformés, et c'est ce débogage-là, pas une simple relance, qui consomme le temps.

**Porte de sortie Phase 1, non négociable** : lancer `ccboard skills --json --include-subagents` contre le vrai `~/.claude/projects` de l'utilisateur et confirmer que le compte d'invocations du skill `flow-lean` correspond à une vérification croisée indépendante, manuelle ou via `cc-skill-usage --json --include-subagents`. Ce contrôle précis a déjà détecté des bugs plus tôt dans l'histoire de ce projet, c'est la porte de non-régression permanente. En plus du total global, les totaux avec `--include-subagents` doivent eux aussi correspondre exactement entre les deux outils. Enfin, le temps de démarrage à froid ne doit pas régresser de plus de 10 % par rapport à la branche `main` actuelle, ce qui est mesurable puisque toute la conception repose sur le principe de zéro lecture de fichier supplémentaire au démarrage.

### Phase 2 : extension UI et TUI/Web parity

Une fois la Phase 1 validée sur données réelles, l'exposition visuelle suit la convention du projet, un onglet dédié avec parité TUI et Web.

| Fichier | Rôle |
|---|---|
| Nouveau tab TUI (`ccboard-tui/src/tabs/skills.rs` ou équivalent) | Leaderboard, détail par skill, badge `[retired]` |
| `tabs/mod.rs` | Enregistrement d'une ligne du nouvel onglet |
| Route Axum (`ccboard-web/src/routes/`) | Endpoint JSON pour le leaderboard des skills |
| Navigation Leptos | Enregistrement d'une ligne du nouvel onglet côté Web |
| `plugin_usage.rs` | Correctif `classify_plugin()` : correspondance exacte sur le namespace en priorité, sous-chaîne uniquement en repli |
| Onglet Agents existant | Resserrement du label « dead code » pour ne se déclencher que si le compte d'invocations est exactement zéro |

Ce sont six fichiers, dont quatre sont des enregistrements d'une ligne dictés par la convention existante du projet, pas six modifications de fond.

### Hors scope pour cette fonctionnalité

`--show-context` pour afficher le contexte autour d'une invocation, l'attribution causale ou en aval du coût (tokens du tour suivant), le succès, l'échec ou la durée d'exécution d'un skill, la diffusion en temps réel via SSE des événements de skill, les graphiques ou sparklines (parité de tableau uniquement en Phase 2), et le bug de regex de commande dans `invocations.rs` (`^/([a-z][a-z0-9-]*)` qui tronque `/boldguy:talks:bg-extract` en `/boldguy`), réel mais sans lien avec cette fonctionnalité, à déposer comme ticket séparé.

### Rollback

À chaque phase, le retrait est peu coûteux. Phase 0 est un ajout isolé dans `store.rs`, un revert Git suffit sans effet de bord sur les autres onglets. Phase 1 n'a pas bumpé `CACHE_VERSION`, un ancien binaire ccboard qui rouvre une base contenant les nouvelles tables les ignore simplement, aucune migration à l'envers n'est nécessaire. Phase 2 n'ajoute que des vues de lecture sur les données déjà persistées en Phase 1, son retrait n'affecte ni le cache ni les données existantes.

### Critères de succès mesurables

| Critère | Seuil |
|---|---|
| Lectures de fichiers au démarrage à froid | Inchangées, toujours deux lectures par session au maximum |
| Correspondance du compte de `flow-lean` avec la vérification croisée | Exacte, y compris avec `--include-subagents` |
| Temps de démarrage à froid vs `main` | Régression inférieure à 10 % |
| Leaderboard à chaud depuis SQLite | Moins de 200 ms |
| Modifications dans `session_index.rs` | Zéro |
| Tests existants de `plugin_usage.rs` | Zéro changement, tous verts |
| Mise à jour des compteurs sans redémarrage après Phase 0 | Vérifiée manuellement (TUI ouverte, skill lancé ailleurs, compteur bouge) |
| `test_parsers_agree_on_token_totals` | Vert en continu, sert de garde-fou contre la dérive entre les deux implémentations dupliquées |
