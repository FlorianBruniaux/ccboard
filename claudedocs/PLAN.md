---
date: 2026-03-20
last_updated: 2026-03-20
version: 0.15.5
title: ccboard — Plan phases restantes vers v1.0.0
status: ACTIVE
---

# Plan: Phases restantes vers v1.0.0

**Dernière mise à jour**: 2026-03-20
**Version actuelle**: v0.15.5
**Dernier commit**: 4520c2e — feat(search): MA4 — extend FTS5 search with date, detail pane, search-as-you-type

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

## 📋 Phase L: Plugin System (v0.16.0)

**Priorité**: 🟢 LOW | **Durée**: 12-15h

Plugin API, dynamic loading (.so/.dylib), hooks customs.
Aucun prérequis bloquant mais forte complexité. À faire après M.

---

## 📋 Phase N: Plan-Aware Completion (v0.16.5)

**Priorité**: 🟢 LOW | **Durée**: 10-14h | **GitHub**: #4, #10-13

PLAN.md parsing, task dependency DAG, session-to-task mapping.
Prérequis : Phase H partiellement fait en v0.8.0.

---

## 🏁 Vers v1.0.0

| Phase | Status | Version |
|-------|--------|---------|
| K-Analytics | ✅ Done | v0.15.0 |
| **M — Conversation** | **✅ Done** | v0.15.5 |
| L — Plugins | 📋 Backlog | v0.16.0 |
| N — Plan-aware | 📋 Backlog | v0.16.5 |
| **v1.0.0** | 🎯 Q3 2026 | L + N + docs |

**Critères v1.0.0** : 500+ tests, 0 bugs critiques, user guide, Phase L livrée.
