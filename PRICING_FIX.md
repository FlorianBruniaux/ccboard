# Pricing Fix - February 2025

## Problem

ccboard affichait des coûts 200% trop élevés pour Opus 4.5/4.6 et sous-estimait de 80% les coûts de cache write pour tous les modèles.

### Erreurs identifiées

1. **Opus 4.5/4.6** : Utilisait les anciens prix Opus 4.0/4.1 ($15/$75) au lieu des nouveaux prix ($5/$25)
2. **Cache write multiplier** : Utilisait 0.25 au lieu de 1.25 pour TOUS les modèles
3. **Opus 4.6** : Model ID manquant dans la table de pricing

## Impact

### Exemple concret (Opus 4.5 - 1M input + 500K output)

- **Avant** : $52.50 (❌ FAUX)
- **Après** : $17.50 (✅ CORRECT)
- **Erreur** : 200% de surestimation

### Cache write (Sonnet 4.5 - 1M cache_create)

- **Avant** : $0.75 (❌ FAUX)
- **Après** : $3.75 (✅ CORRECT)
- **Erreur** : 80% de sous-estimation

## Solution appliquée

### 1. Prix Opus 4.5/4.6 corrigés

```rust
// AVANT (FAUX)
let opus_pricing = ModelPricing {
    input_price_per_million: 15.0,  // ❌
    output_price_per_million: 75.0, // ❌
};

// APRÈS (CORRECT)
let opus_45_pricing = ModelPricing {
    input_price_per_million: 5.0,   // ✅
    output_price_per_million: 25.0, // ✅
    cache_write_multiplier: 1.25,   // ✅ (fix global)
};
```

### 2. Opus 4.0/4.1 legacy préservé

Les anciens modèles Opus 4.0/4.1 gardent leurs prix historiques pour rétrocompatibilité :

```rust
let opus_legacy_pricing = ModelPricing {
    input_price_per_million: 15.0,
    output_price_per_million: 75.0,
    cache_write_multiplier: 1.25, // ✅ Fix appliqué aussi
};
m.insert("claude-opus-4-0-20250514", opus_legacy_pricing.clone());
m.insert("claude-opus-4-1-20250805", opus_legacy_pricing);
```

### 3. Cache write multiplier corrigé partout

Tous les modèles (Opus, Sonnet, Haiku) utilisent maintenant `cache_write_multiplier: 1.25` au lieu de `0.25`.

### 4. Default average mis à jour

```rust
// AVANT
input_price_per_million: 3.5,   // Basé sur anciens prix Opus
output_price_per_million: 17.5,

// APRÈS
input_price_per_million: 3.2,   // (0.7×$3 + 0.2×$5 + 0.1×$1)
output_price_per_million: 16.0, // (0.7×$15 + 0.2×$25 + 0.1×$5)
```

## Prix officiels (Février 2025)

| Model | Input | Output | Cache Write | Cache Read |
|-------|-------|--------|-------------|------------|
| **Opus 4.5/4.6** | $5/M | $25/M | $6.25/M (1.25×$5) | $0.50/M (0.1×$5) |
| Opus 4.0/4.1 (legacy) | $15/M | $75/M | $18.75/M (1.25×$15) | $1.50/M (0.1×$15) |
| Sonnet 4.5 | $3/M | $15/M | $3.75/M (1.25×$3) | $0.30/M (0.1×$3) |
| Haiku 4.5 | $1/M | $5/M | $1.25/M (1.25×$1) | $0.10/M (0.1×$1) |

Source : https://www.anthropic.com/api#pricing

## Validation

Tests automatiques (15 tests passent) :
```bash
cargo test -p ccboard-core pricing
```

Test manuel :
```bash
# Opus 4.5 : 1M input + 500K output = $17.50 ✅
# Cache write : 1M cache_create Sonnet = $3.75 ✅
```

## Impact utilisateurs

Les utilisateurs de ccboard vont voir :
- **Opus 4.5/4.6** : Coûts divisés par 3 (plus réalistes)
- **Cache write** : Coûts multipliés par 5 (correction de sous-estimation)
- **Historique** : Les anciennes sessions gardent les mêmes calculs (pas de recalcul rétroactif)

## Next steps (Phase 2 - v0.6.1)

Command `ccboard pricing update` pour fetch automatique depuis LiteLLM :
- Source : https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json
- Stockage : `~/.cache/ccboard/pricing.json`
- Fallback : Embedded pricing si offline

## Commit

```
fix: correct Opus 4.5/4.6 pricing and cache_write_multiplier

- Opus 4.5/4.6: $5/$25 (was $15/$75, 200% overestimate)
- Cache write: 1.25x multiplier (was 0.25x, 80% underestimate)
- Add Opus 4.0/4.1 legacy pricing for retrocompatibility
- Add claude-opus-4-6-20250212 model ID
- Update default_average() to reflect new Opus pricing
- Fix all tests and docstring examples

Fixes #42 (pricing discrepancy reported by Frédéric)
