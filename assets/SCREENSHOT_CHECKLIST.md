# Screenshot Checklist - Manual Capture

**Tool**: `Cmd+Shift+4` puis `Space` (capture window) sur macOS

**Terminal size**: Redimensionner le terminal à ~1920x1080 ou plein écran

**Save location**: `assets/screenshots/` (ce répertoire existe déjà)

---

## ☐ Screenshot 1: Dashboard Tab

**Filename**: `01_dashboard.png`

**Steps**:
```bash
# 1. Lancer ccboard (attendre le chargement complet)
ccboard

# 2. Appuyer sur '1' pour s'assurer d'être sur Dashboard
# 3. Cmd+Shift+4 puis Space → cliquer sur la fenêtre Terminal
# 4. Sauvegarder dans assets/screenshots/ avec nom: 01_dashboard.png
```

**Contenu visible**:
- Stats cards (Tokens, Sessions, Messages, Cache Hit Rate)
- 7-day activity sparkline
- Top 5 model usage gauges

---

## ☐ Screenshot 2: Sessions Tab

**Filename**: `02_sessions.png`

**Steps**:
```bash
# 1. Depuis ccboard en cours
# 2. Appuyer sur '2' (Sessions tab)
# 3. Naviguer avec j/k pour montrer quelques sessions
# 4. Cmd+Shift+4 puis Space → cliquer fenêtre Terminal
# 5. Sauvegarder: 02_sessions.png
```

**Contenu visible**:
- Dual-pane layout (projects | sessions)
- Project tree avec expand/collapse
- Session metadata + first message preview

---

## ☐ Screenshot 3: Help Modal

**Filename**: `03_help_modal.png`

**Steps**:
```bash
# 1. Depuis ccboard en cours
# 2. Appuyer sur '?' (toggle help modal)
# 3. Vérifier que le modal est centré et visible
# 4. Cmd+Shift+4 puis Space → capture
# 5. Sauvegarder: 03_help_modal.png
```

**Contenu visible**:
- Modal centré avec border cyan
- Global keybindings (q, ?, :, F5, Tab, 1-8)
- Tab-specific shortcuts

---

## ☐ Screenshot 4: Search Highlighting

**Filename**: `04_search_highlighting.png`

**Steps**:
```bash
# 1. Depuis ccboard en cours
# 2. Appuyer sur '2' (Sessions tab si pas déjà)
# 3. Appuyer sur '/' (activer search)
# 4. Taper 'rtk' ou autre terme fréquent
# 5. Observer les matches en jaune dans la liste
# 6. Cmd+Shift+4 puis Space → capture
# 7. Sauvegarder: 04_search_highlighting.png
```

**Contenu visible**:
- Search bar actif avec query
- Matches highlighted en yellow background
- Plusieurs résultats visibles

---

## ☐ Screenshot 5: Loading Spinner

**Filename**: `05_loading_spinner.png`

**Steps**:
```bash
# 1. Quitter ccboard (q)
# 2. Effacer le cache pour forcer cold start
rm -f ~/.claude/cache/session-metadata.db*

# 3. Lancer ccboard ET capturer IMMÉDIATEMENT (dans les 2 premières secondes)
ccboard

# 4. Cmd+Shift+4 puis Space → capture RAPIDE pendant le spinner
# 5. Sauvegarder: 05_loading_spinner.png

# Note: Si raté, relancer et recapturer (le cache se reconstruit en 20s)
```

**Contenu visible**:
- Spinner Braille animé (⠋⠙⠹⠸...)
- "Loading..." message
- Progress indication

---

## Validation Post-Capture

Une fois les 5 screenshots pris, vérifier :

```bash
# Lister les screenshots
ls -lh assets/screenshots/

# Devrait montrer:
# 01_dashboard.png
# 02_sessions.png
# 03_help_modal.png
# 04_search_highlighting.png
# 05_loading_spinner.png
```

Dimensions attendues : ~1920x1080 ou similaire (16:9 ratio)

---

## Ensuite

Dire "✓ screenshots pris" et on passe à **A.1: README.md** qui utilisera ces images.
