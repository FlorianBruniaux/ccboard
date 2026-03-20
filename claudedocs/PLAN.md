---
date: 2026-03-20
last_updated: 2026-03-20
version: 0.15.0
title: ccboard — Plan phases restantes vers v1.0.0
status: ACTIVE
---

# Plan: Phases restantes vers v1.0.0

**Dernière mise à jour**: 2026-03-20
**Version actuelle**: v0.15.0
**Dernier commit**: 11426b8 — feat(sessions): MA2 — regex search in replay viewer

> Pour l'historique des phases 0→K, voir [ROADMAP.md](ROADMAP.md).

---

## ✅ Récemment livré : K-Analytics (v0.15.0)

- Streak detection, daily cost spikes, model recommendations
- AnalyticsData caching (no per-frame recomputation)
- 433 tests, 0 warnings — commits f17e747 + c2d315b

---

## 🎯 Phase M: Conversation Viewer Enhancements (v0.15.5) — EN COURS

**Priorité**: 🟡 MEDIUM | **Durée**: 8-10h | **GitHub**: #3, #7, #8

### ✅ MA1 — Tool Call Visualization — DONE (commit c213a65)
Parse tool_use/tool_result content blocks, expandable dans le replay viewer.
- `extract_tool_use_blocks`, `extract_tool_result_blocks`, `format_tool_input_summary`
- Collapsed: `▶ 2 tool call(s): Read, Bash [Enter]` — Expanded: nom + param clé
- `extract_message_content` ne pollue plus avec `[tool_use]`
- 6 nouveaux tests

### ✅ MA2 — Regex Search dans le viewer — DONE (commit 11426b8)
Barre de recherche interactive avec regex + navigation hits dans le replay viewer.
- `/` active la search, `n`/`N` next/prev hit (cyclique), Esc clear/close
- Highlights jaune dans le texte (réutilise `highlight_matches`)
- Fallback literal si regex invalide
- Compteur `[2/7]` dans la barre de status
- 5 nouveaux tests

### MA3 — Export HTML enrichi (~2h)
Syntax highlighting dans l'export HTML des conversations.
- Fichier : `crates/ccboard-core/src/export.rs`
- Prérequis : export markdown Phase J ✅

### MA4 — Multi-session Search cross-session (~2h)
Full-text search cross-sessions depuis le Search tab (extend FTS5).
- Fichier : `crates/ccboard-tui/src/tabs/search.rs`

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
| **M — Conversation** | **⏳ Next** | v0.15.5 |
| L — Plugins | 📋 Backlog | v0.16.0 |
| N — Plan-aware | 📋 Backlog | v0.16.5 |
| **v1.0.0** | 🎯 Q3 2026 | M + L + docs |

**Critères v1.0.0** : 500+ tests, 0 bugs critiques, user guide, Phase M + L livrées.
