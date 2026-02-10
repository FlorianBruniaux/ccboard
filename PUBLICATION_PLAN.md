# Plan d'Action - Publication sur crates.io

**Date de préparation** : 2026-02-10
**Status** : ✅ READY - Tous les blockers résolus
**Commit** : 611027b

---

## ✅ Pré-requis Complétés

- [x] Edition Rust 2024 → 2021 (Cargo.toml)
- [x] 4 tests invocations fixés (168 tests passed)
- [x] CITATION.cff mis à jour (v0.5.0)
- [x] unwrap() critiques remplacés (4 fichiers)
- [x] Tests unsafe isolés (#[serial])
- [x] SECURITY.md créé
- [x] Cross-références ajoutées (README)
- [x] Commit pushé sur GitHub

**Métriques** :
- Tests : 280+ passed, 0 failed
- Clippy : 0 warnings critiques
- Binary : 4.1MB (release)
- Dry-run : ✅ ccboard-types, ✅ ccboard-core

---

## 📦 Ordre de Publication (Critique)

Le workspace contient 5 crates interdépendants. **Publier dans cet ordre exact** :

```
1. ccboard-types    (base, pas de dépendances internes)
2. ccboard-core     (dépend de ccboard-types)
3. ccboard-tui      (dépend de ccboard-core)
4. ccboard-web      (dépend de ccboard-core)
5. ccboard          (dépend de tui + web + core)
```

---

## 🚀 Commandes de Publication

### Étape 1 : ccboard-types

```bash
# Publier
cargo publish -p ccboard-types

# Attendre indexation crates.io (~30-60 secondes)
# Vérifier : https://crates.io/crates/ccboard-types
```

**Résultat attendu** :
```
✅ Uploaded ccboard-types v0.5.0
   17 files, 25.0KiB compressed
```

---

### Étape 2 : ccboard-core

```bash
# Attendre que ccboard-types soit indexé
sleep 60

# Publier
cargo publish -p ccboard-core

# Attendre indexation (~30-60 secondes)
# Vérifier : https://crates.io/crates/ccboard-core
```

**Résultat attendu** :
```
✅ Uploaded ccboard-core v0.5.0
   46 files, 91.6KiB compressed
⚠️  1 warning: dead_code (extract_invocations) - non bloquant
```

**Note** : Le warning `extract_invocations` est mineur et n'empêche pas la publication.

---

### Étape 3 : ccboard-tui

```bash
# Attendre que ccboard-core soit indexé
sleep 60

# Publier
cargo publish -p ccboard-tui

# Attendre indexation (~30-60 secondes)
# Vérifier : https://crates.io/crates/ccboard-tui
```

---

### Étape 4 : ccboard-web

```bash
# Attendre que ccboard-core soit indexé (si pas déjà fait)
sleep 60

# Publier
cargo publish -p ccboard-web

# Attendre indexation (~30-60 secondes)
# Vérifier : https://crates.io/crates/ccboard-web
```

**Note** : ccboard-tui et ccboard-web peuvent être publiés en parallèle (tous deux dépendent de ccboard-core).

---

### Étape 5 : ccboard (binaire principal)

```bash
# Attendre que tui + web + core soient indexés
sleep 60

# Publier le binaire principal
cargo publish -p ccboard

# Vérifier : https://crates.io/crates/ccboard
```

**Résultat final attendu** :
```
✅ Uploaded ccboard v0.5.0 to crates.io
```

---

## 🤖 Script Automatisé (Optionnel)

```bash
#!/bin/bash
# publish.sh - Publication automatisée du workspace

set -e  # Exit on error

echo "🚀 Publication ccboard workspace v0.5.0"
echo "========================================"

# 1. ccboard-types
echo ""
echo "📦 [1/5] Publishing ccboard-types..."
cargo publish -p ccboard-types
echo "⏳ Waiting 60s for crates.io indexing..."
sleep 60

# 2. ccboard-core
echo ""
echo "📦 [2/5] Publishing ccboard-core..."
cargo publish -p ccboard-core
echo "⏳ Waiting 60s for crates.io indexing..."
sleep 60

# 3. ccboard-tui
echo ""
echo "📦 [3/5] Publishing ccboard-tui..."
cargo publish -p ccboard-tui
echo "⏳ Waiting 60s for crates.io indexing..."
sleep 60

# 4. ccboard-web
echo ""
echo "📦 [4/5] Publishing ccboard-web..."
cargo publish -p ccboard-web
echo "⏳ Waiting 60s for crates.io indexing..."
sleep 60

# 5. ccboard (main binary)
echo ""
echo "📦 [5/5] Publishing ccboard (main)..."
cargo publish -p ccboard

echo ""
echo "✅ All crates published successfully!"
echo "🔗 Check: https://crates.io/crates/ccboard"
```

**Usage** :
```bash
chmod +x publish.sh
./publish.sh
```

---

## 🏷️ Après Publication

### 1. Créer le tag Git

```bash
git tag v0.5.0
git push origin v0.5.0
```

### 2. Créer GitHub Release

```bash
# Via gh CLI
gh release create v0.5.0 \
  --title "ccboard v0.5.0 - Public Release" \
  --notes-file CHANGELOG.md

# Ou manuellement sur GitHub
# https://github.com/FlorianBruniaux/ccboard/releases/new
```

### 3. Mettre à jour la documentation

- [ ] README : Changer "coming soon" → lien crates.io actif
- [ ] Cargo.toml : Vérifier homepage/documentation
- [ ] Annoncer sur les réseaux sociaux (LinkedIn, Twitter)

### 4. Vérifications post-publication

```bash
# Installer depuis crates.io
cargo install ccboard

# Tester l'installation
ccboard --version  # Should show 0.5.0
ccboard stats      # Should work

# Vérifier docs.rs
# https://docs.rs/ccboard/0.5.0
```

---

## ⚠️ Problèmes Potentiels

### Échec d'indexation crates.io

**Symptôme** : `no matching package named ccboard-core found`
**Cause** : crates.io pas encore indexé
**Solution** : Attendre 1-2 minutes supplémentaires, réessayer

### Erreur de token

**Symptôme** : `error: no upload token found`
**Solution** :
```bash
cargo login
# Entrer le token depuis https://crates.io/settings/tokens
```

### Limite de taille

**Symptôme** : `error: package size exceeds 10MB`
**Solution** : Vérifier `.gitignore` et `Cargo.toml` exclude

---

## 📊 Checklist Finale

Avant de lancer la publication :

- [ ] `git status` propre (aucun fichier modifié)
- [ ] `cargo test --all` passe (280+ tests)
- [ ] `cargo clippy --all-targets` passe (0 warnings critiques)
- [ ] `cargo build --release` réussit
- [ ] Token crates.io configuré (`cargo login`)
- [ ] Connexion internet stable
- [ ] ~5 minutes disponibles pour la publication complète

---

## 🎯 Résumé Rapide

```bash
# Publication complète (5 commandes)
cargo publish -p ccboard-types && sleep 60 && \
cargo publish -p ccboard-core && sleep 60 && \
cargo publish -p ccboard-tui && sleep 60 && \
cargo publish -p ccboard-web && sleep 60 && \
cargo publish -p ccboard

# Tag et release
git tag v0.5.0 && git push origin v0.5.0
gh release create v0.5.0 --notes-file CHANGELOG.md

# Installation test
cargo install ccboard
ccboard --version
```

---

**Durée estimée** : ~6-8 minutes (5 crates × 1 min + attentes indexation)
**Dernière mise à jour** : 2026-02-10
**Contact** : florian.bruniaux@gmail.com
