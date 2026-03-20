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
**Dernier commit**: c2d315b — fix(analytics): address all 10 code review findings

> Pour l'historique des phases 0→K, voir [ROADMAP.md](ROADMAP.md).

---

## ✅ Récemment livré : K-Analytics (v0.15.0)

- Streak detection, daily cost spikes, model recommendations
- AnalyticsData caching (no per-frame recomputation)
- 433 tests, 0 warnings — commits f17e747 + c2d315b

---

## 🎯 Phase M: Conversation Viewer Enhancements (v0.15.5) — NEXT

**Priorité**: 🟡 MEDIUM | **Durée**: 8-10h | **GitHub**: #3, #7, #8

### MA1 — Tool Call Visualization (~3h)
Expandable tool call nodes (input/output) dans le detail view de session.
- Fichiers : `crates/ccboard-tui/src/tabs/sessions.rs`, `crates/ccboard-core/src/parsers/session_index.rs`
- Prérequis : `ConversationMessage` déjà parsé ✅

### MA2 — Regex Search dans le viewer (~2h)
Remplacer la recherche plain-text par regex dans la vue conversation.
- Fichier : `crates/ccboard-tui/src/tabs/sessions.rs`
- FTS5 SQLite pour metadata, regex pour content display

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
