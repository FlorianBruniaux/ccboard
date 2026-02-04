# Phase H: Advanced Analytics Enhancements

**Durée estimée**: 8-12h
**Objectif**: Améliorer les analytics existantes avec de nouvelles métriques et visualisations

---

## 📊 État Actuel

### ✅ Déjà Implémenté

- **Forecasting** (forecasting.rs):
  - Régression linéaire pour prédictions 30 jours
  - Coefficient R² (confiance 0-100%)
  - Trend direction (Up/Down/Stable avec %)
  - Monthly cost estimate

- **Patterns** (patterns.rs):
  - Hourly distribution (heatmap des heures)
  - Weekday distribution
  - Model usage (tokens %)
  - Peak hours detection
  - Most productive day

- **Trends** (trends.rs):
  - Daily time series (tokens, sessions, cost)
  - Model usage over time
  - Sparklines dans Dashboard

- **Insights** (insights.rs):
  - Recommendations textuelles
  - Pattern-based advice

### 🔨 Ce Qui Manque

**1. Visualisations Avancées**
- [ ] Heatmap 2D (jour × heure) pour visualiser les patterns
- [ ] Forecast graph (30 jours futurs avec bande de confiance)
- [ ] Model usage pie chart / stacked bar
- [ ] Cost breakdown par modèle

**2. Métriques Additionnelles**
- [ ] Session duration trends (avg, median, p95)
- [ ] Messages per session trends
- [ ] Cache hit ratio trends (si disponible)
- [ ] Cost per message/session metrics

**3. Alertes & Anomalies**
- [ ] Spike detection (usage anormal)
- [ ] Budget warnings (si monthly estimate > budget)
- [ ] Anomaly detection simple (écart-type)

**4. Comparaisons**
- [ ] Week-over-week comparison
- [ ] Month-over-month comparison
- [ ] Model efficiency comparison (cost/token)

**5. Export & Reporting**
- [ ] Export analytics en JSON
- [ ] Generate text report (markdown)
- [ ] CSV export pour Excel

---

## 🎯 Proposition: 3 Tâches Prioritaires

### Task H.1: Forecast Visualization (3-4h)

**Objectif**: Visualiser graphiquement les prédictions

**Implémentation**:
```
Analytics > Trends tab:
┌─ Forecast (Next 30 Days) ─────────────────┐
│                                            │
│  Tokens  ▲                                │
│          │        ...future...            │
│   150K   │     ╱╱╱╱╱╱╱╱╱╱╱               │
│          │  ╱╱╱                           │
│   100K   │╱╱                              │
│          ├──────────────────────          │
│    50K   │ past (actual)                  │
│          │                                │
│          └─────────────────────────► Days │
│            7d   14d   21d   30d           │
│                                            │
│ Confidence: ██████████ 87% (R²)           │
│ Trend: ↗ Up 12% (next 30d)                │
└────────────────────────────────────────────┘
```

**Fichiers**:
- `crates/ccboard-tui/src/tabs/analytics.rs` - render_trends()
- Utiliser ratatui `Chart` widget avec 2 `Dataset`:
  - Historical (ligne solide)
  - Forecast (ligne pointillée)

**Tests**:
- [ ] Forecast avec confidence > 0.7 (ligne verte)
- [ ] Forecast avec confidence < 0.4 (ligne rouge)
- [ ] Gestion de trends.dates.len() < 7

---

### Task H.2: Session Duration Analytics (2-3h)

**Objectif**: Analyser la durée des sessions pour optimiser workflows

**Nouvelles métriques** (à ajouter dans TrendsData):
```rust
pub struct SessionDurationStats {
    pub avg_duration_secs: f64,
    pub median_duration_secs: f64,
    pub p95_duration_secs: f64,
    pub shortest_session_secs: u64,
    pub longest_session_secs: u64,
}
```

**Affichage** (Patterns tab):
```
Session Duration Distribution
─────────────────────────────
Avg:      12m 34s  (median: 8m 12s)
P95:      45m 10s  (95% sessions < this)
Shortest: 23s
Longest:  2h 14m

Distribution:
0-5m    ████████████ 45%
5-15m   ██████████████████ 68%
15-30m  ████████ 30%
30-60m  ████ 15%
60m+    ██ 8%
```

**Insight generation**:
- "Most sessions are 5-15 minutes - consider breaking long tasks"
- "95% of sessions < 45 minutes - workflows well-sized"

**Fichiers**:
- `crates/ccboard-core/src/analytics/trends.rs` - compute_trends()
- `crates/ccboard-tui/src/tabs/analytics.rs` - render_patterns()

---

### Task H.3: Budget & Alerts (3-4h)

**Objectif**: Alertes proactives sur budget et anomalies

**Implémentation**:
1. **Budget Config** (ajouter dans Settings):
```rust
pub struct BudgetConfig {
    pub monthly_budget_usd: Option<f64>,
    pub alert_threshold_pct: f64, // Default: 80%
}
```

2. **Alert Detection** (dans insights.rs):
```rust
pub enum Alert {
    BudgetWarning { current: f64, budget: f64, pct: f64 },
    UsageSpike { day: String, tokens: u64, avg: u64 },
    CostAnomaly { day: String, cost: f64, expected: f64 },
}
```

3. **Affichage** (Overview tab):
```
Budget Status
─────────────
Monthly Est:   $45.20
Budget:        $50.00 ━━━━━━━━━━━━━━━━━━ 90%
Remaining:     $4.80 (10%)

⚠️  WARNING: Approaching budget limit (90%)
💡 TIP: Projected overage: $2.15 if trend continues
```

**Fichiers**:
- `crates/ccboard-core/src/models/config.rs` - BudgetConfig
- `crates/ccboard-core/src/analytics/insights.rs` - generate_budget_alerts()
- `crates/ccboard-tui/src/tabs/analytics.rs` - render_overview()

---

## 🚀 Tâches Optionnelles (Si Temps Restant)

### Task H.4: Heatmap Visualization (2-3h)

Heatmap jour × heure pour visualiser les patterns d'usage:
```
     00 02 04 06 08 10 12 14 16 18 20 22
Mon  ░░ ░░ ░░ ▒▒ ▓▓ ██ ██ ██ ▓▓ ▒▒ ░░ ░░
Tue  ░░ ░░ ░░ ▒▒ ▓▓ ██ ██ ██ ▓▓ ▒▒ ░░ ░░
Wed  ░░ ░░ ░░ ▒▒ ██ ██ ██ ██ ██ ▓▓ ▒▒ ░░
...
```

### Task H.5: Export & Reporting (1-2h)

- JSON export: `/api/analytics/export.json`
- Markdown report: Generate summary report
- CSV data: For Excel analysis

### Task H.6: Week/Month Comparisons (2h)

Compare current week vs previous week:
```
Comparison: This Week vs Last Week
───────────────────────────────────
Tokens:    +15% ↗️  (850K → 978K)
Sessions:  +8%  ↗️  (120 → 130)
Cost:      +12% ↗️  ($12.50 → $14.00)
Avg/sess:  +6%  ↗️  (7.1K → 7.5K tokens)
```

---

## 📦 Résumé Phase H

**Core Tasks** (obligatoires):
- H.1: Forecast Visualization (3-4h)
- H.2: Session Duration Analytics (2-3h)
- H.3: Budget & Alerts (3-4h)

**Total**: 8-11h

**Optional** (si temps):
- H.4: Heatmap (2-3h)
- H.5: Export (1-2h)
- H.6: Comparisons (2h)

**Total max**: 8-11h + 5-7h = **13-18h**

---

## ✅ Critères de Succès

**Phase H Complete si**:
1. ✅ Forecast graph affiché (Trends tab)
2. ✅ Session duration stats (Patterns tab)
3. ✅ Budget warnings (Overview tab)
4. ✅ Tests unitaires pour nouvelles métriques
5. ✅ Documentation dans CLAUDE.md

**Bonus**:
- ✅ Heatmap jour×heure
- ✅ Export JSON/CSV
- ✅ Week-over-week comparison

---

## 🎯 Questions pour Toi

Avant de commencer, dis-moi ce qui t'intéresse le plus :

1. **Les 3 tâches core** (H.1-H.3) - 8-11h ?
2. **+ Heatmap** (bonus visuel sympa) ?
3. **+ Export** (pour analyser dans Excel) ?
4. **Autre priorité** ?

Je peux aussi créer un ordre différent si tu préfères certaines features avant d'autres.
