#!/bin/bash
# Script de test manuel pour W3.1 SSE Integration

echo "=== ccboard W3.1 SSE Live Updates Test ==="
echo ""
echo "Ce script va:"
echo "1. Lancer le serveur web ccboard"
echo "2. Ouvrir le navigateur sur http://localhost:3333"
echo "3. Attendre que vous testiez les fonctionnalités SSE"
echo ""
echo "Tests à effectuer:"
echo "  - Dashboard: Modifier ~/.claude/stats-cache.json → voir toast + update"
echo "  - Sessions: Créer une session Claude → voir toast + nouvelle session"
echo "  - Analytics: Modifier stats → voir toast + graphiques mis à jour"
echo "  - Reconnexion: Arrêter/redémarrer serveur → vérifier reconnexion"
echo "  - Multiple toasts: Déclencher plusieurs events → voir stack"
echo ""
read -p "Appuyez sur Entrée pour lancer le serveur..."

# Build du projet
echo "Building ccboard..."
cargo build --release

# Lancer le serveur web
echo ""
echo "Starting web server on http://localhost:3333..."
echo "Press Ctrl+C to stop"
echo ""

cargo run --release -- web --port 3333
