---
date: 2026-03-20
last_updated: 2026-03-27
version: 0.18.0
title: ccboard — Plan phases restantes vers v1.0.0
status: ACTIVE
---

# Plan: Phases restantes vers v1.0.0

**Dernière mise à jour**: 2026-03-27
**Version actuelle**: v0.18.0
**Dernier commit**: 108cbda — feat(sessions): SF4 LLM session summaries via ccboard summarize

> Pour l'historique des phases 0→K, voir [ROADMAP.md](ROADMAP.md).

---

## ✅ Phase M: Conversation Viewer Enhancements (v0.15.5) — DONE

**Priorité**: 🟡 MEDIUM | **GitHub**: #3, #7, #8

### ✅ MA1 — Tool Call Visualization (commit c213a65)
Parse tool_use/tool_result content blocks, expandable dans le replay viewer.
- `extract_tool_use_blocks`, `extract_tool_result_blocks`, `format_tool_input_summary`
- Collapsed: `▶ 2 tool call(s): Read, Bash [Enter]` — Expanded: nom + param clé
- 6 nouveaux tests

### ✅ MA2 — Regex Search dans le viewer (commit 11426b8)
- `/` active la search, `n`/`N` next/prev hit (cyclique), Esc clear/close
- Highlights jaune, fallback literal si regex invalide, compteur `[2/7]`
- 5 nouveaux tests

### ✅ MA3 — Export HTML enrichi (commit d87a25d)
Syntax highlighting via syntect dans l'export HTML des conversations.
- `render_content_as_html` : détecte code fences, applique InspiredGitHub theme
- Language badge `.code-lang` + conteneur `.code-block` responsive
- Fast path pour messages sans code blocks
- 6 nouveaux tests

### ✅ MA4 — Multi-session Search cross-sessions (commit 4520c2e)
Extension FTS5 + UX Search tab.
- `SearchResult` : ajout `first_timestamp` + `message_count` (SQL join session_metadata)
- Snippet étendu à 12 tokens (vs 8) pour plus de contexte
- Results list : affichage date (jaune) + nombre de messages par résultat
- Detail pane (40% width) : projet, date, session ID, messages, snippet complet
- Search-as-you-type : refresh auto à chaque caractère (min 2 chars)
- ↑/↓ naviguent les résultats même en mode input
- Enter ouvre la conversation overlay depuis les deux modes
- 8 nouveaux tests

**Total Phase M : 458 tests, 0 warnings**

---

## ✅ v0.16.x — Visual Redesign + Bug Fixes (2026-03-23)

- TUI visual redesign : palette system, rounded borders, sub-tabs, heatmap responsive
- Keybindings `?` / `:` fixés sur macOS (KeyModifiers::NONE)
- Web Activity + Analytics Tools CSS complet (440+ lignes)
- Pricing étendu : `claude-sonnet-4-6`, alias dot-style
- Plan auto-détection depuis `~/.claude.json`
- `cargo install` web embedding fixé (dist/ inclus dans crates.io)
- `docs/GUIDE.md` créé (700 lignes, guide complet)

---

## ✅ v0.17.0 — Waiting Answers + Hook TTL (2026-03-24)

- Waiting Answers panel dans Sessions tab (WaitingInput sessions)
- Max 20x tip contextuel dans le Dashboard
- Fix navigation Waiting Answers (h/l hint + cache immédiat)
- Hook state TTL : pruning des sessions Running/WaitingInput stale après 10 min
- Settings hot-reload toast

---

## ✅ v0.18.0 — Session Intelligence (2026-03-27)

Sprint 2 features — 472 tests, 0 warnings.

- **QW3 — Model switching timeline** : segments `Opus 4.5 (8) → Sonnet 4.6 (15)` dans le detail pane, calculés inline lors du scan JSONL
- **QW4 — Context saturation trend** : régression linéaire sur les N dernières sessions, prédit `↑ ~N sessions` avant 85% ou `↓ declining`
- **MF2 — Session bookmarks** : `b` toggle, `B` filtre starred, `★` icon, `BookmarkStore` persisté dans `~/.ccboard/bookmarks.json`
- **MF6 — Subagent graph** : `parent_session_id` capturé au scan, `has_subagents` rétro-rempli post-load, arbre `⤵/⤴` dans le detail pane, `subagent_children()` DataStore
- **SF4 — LLM summaries** : `ccboard summarize <id>` via `claude --print`, cache `~/.ccboard/summaries/`, affiché dans le detail pane

---

## 📋 Phase L: Plugin System (v0.19.0)

**Priorité**: 🟢 LOW | **Durée**: 12-15h

Plugin API, dynamic loading (.so/.dylib), hooks customs.
Aucun prérequis bloquant mais forte complexité. À faire après M.

---

## 📋 Phase N: Plan-Aware Completion (v0.19.5)

**Priorité**: 🟢 LOW | **Durée**: 10-14h | **GitHub**: #4, #10-13

PLAN.md parsing, task dependency DAG, session-to-task mapping.
Prérequis : Phase H partiellement fait en v0.8.0.

---

## 🏁 Vers v1.0.0

| Phase | Status | Version |
|-------|--------|---------|
| K-Analytics | ✅ Done | v0.15.0 |
| M — Conversation | ✅ Done | v0.15.5 |
| v0.16.x fixes | ✅ Done | v0.16.0–0.16.5 |
| v0.17.0 — Waiting Answers | ✅ Done | v0.17.0 |
| **v0.18.0 — Session Intelligence** | ✅ Done | v0.18.0 |
| **L — Plugins** | 📋 Backlog | v0.19.0 |
| N — Plan-aware | 📋 Backlog | v0.19.5 |
| **v1.0.0** | 🎯 Q3 2026 | L + N |

**Critères v1.0.0** : 500+ tests, 0 bugs critiques, Phase L livrée.
