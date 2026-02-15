#!/bin/bash
# grepai-start.sh - Lance les services requis pour grepai MCP
# Usage: ./grepai-start.sh [project_dir]
#
# Prérequis:
#   - Ollama installé (brew install ollama / curl install)
#   - grepai installé (https://github.com/yoanbernabeu/grepai/releases)
#   - grepai init exécuté dans le projet

set -e

PROJECT_DIR="${1:-$CLAUDE_PROJECT_DIR}"
OS="$(uname -s)"

# Helper functions for cross-platform process detection
check_process() {
  local name="$1" mode="${2:--x}"
  if command -v pgrep &>/dev/null; then
    pgrep "$mode" "$name" > /dev/null 2>&1
  else
    ps aux 2>/dev/null | grep -v grep | grep -qw "$name"
  fi
  return 0
}

get_pid() {
  local name="$1" mode="${2:--x}"
  if command -v pgrep &>/dev/null; then
    pgrep "$mode" "$name" | head -1
  else
    ps aux 2>/dev/null | grep -v grep | grep -w "$name" | awk '{print $2}' | head -1
  fi
  return 0
}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔧 Démarrage des services grepai..."
echo ""

# --- 1. Ollama ---
echo "1/3 Vérification Ollama..."

if ! command -v ollama &> /dev/null; then
    echo -e "${RED}❌ Ollama non installé${NC}"
    echo "   → macOS: brew install ollama"
    echo "   → Linux: curl -fsSL https://ollama.com/install.sh | sh"
    exit 1
fi

if ! check_process "ollama"; then
    echo "   Démarrage Ollama..."
    if [[ "$OS" == "Darwin" ]]; then
        brew services start ollama 2>/dev/null || ollama serve &> /dev/null &
    else
        ollama serve &> /dev/null &
    fi

    # Retry loop avec timeout pour vérifier le démarrage
    MAX_RETRIES=10
    RETRY_COUNT=0
    while ! check_process "ollama"; do
        RETRY_COUNT=$((RETRY_COUNT + 1))
        if [[ $RETRY_COUNT -ge $MAX_RETRIES ]]; then
            echo -e "${RED}❌ Ollama n'a pas démarré après ${MAX_RETRIES} tentatives${NC}"
            echo "   → Vérifiez les logs: brew services info ollama (macOS)"
            if [[ "$OS" == "Darwin" ]] || [[ "$OS" == "Linux" ]]; then
              echo "   → Port 11434 may be in use: lsof -i :11434"
            else
              echo "   → Port 11434 may be in use: netstat -ano | findstr :11434"
            fi
            exit 1
        fi
        echo "   Attente du démarrage Ollama... ($RETRY_COUNT/$MAX_RETRIES)"
        sleep 1
    done
fi

OLLAMA_PID=$(get_pid "ollama")
if [[ -n "$OLLAMA_PID" ]]; then
    echo -e "${GREEN}✅ Ollama: running (PID $OLLAMA_PID)${NC}"
else
    echo -e "${RED}❌ Ollama: échec du démarrage${NC}"
    echo "   → Essayez manuellement: ollama serve"
    exit 1
fi

# --- 2. Modèle embeddings ---
echo ""
echo "2/3 Vérification modèle embeddings..."

if ! ollama list 2>/dev/null | grep -q "nomic-embed-text"; then
    echo "   Téléchargement nomic-embed-text (première utilisation)..."
    ollama pull nomic-embed-text
fi
echo -e "${GREEN}✅ Model: nomic-embed-text loaded${NC}"

# --- 3. grepai watch ---
echo ""
echo "3/3 Vérification grepai watch..."

if ! command -v grepai &> /dev/null; then
    echo -e "${RED}❌ grepai non installé${NC}"
    echo "   → https://github.com/yoanbernabeu/grepai/releases"
    echo "   → Guide: doc/guides/tech/2026-01-15_grepai-mcp-setup.md"
    exit 1
fi

if [[ -z "$PROJECT_DIR" ]]; then
    echo -e "${YELLOW}⚠️  PROJECT_DIR non défini, utilisation du répertoire courant${NC}"
    PROJECT_DIR="$(pwd)"
fi

# Vérifier que le répertoire existe et est accessible
if [[ ! -d "$PROJECT_DIR" ]]; then
    echo -e "${RED}❌ PROJECT_DIR n'existe pas: $PROJECT_DIR${NC}"
    exit 1
fi

# Vérifier que grepai a été initialisé dans ce projet
if [[ ! -f "$PROJECT_DIR/.grepai/config.yaml" ]]; then
    echo -e "${RED}❌ grepai non initialisé dans ce projet${NC}"
    echo "   → Exécutez d'abord: cd $PROJECT_DIR && grepai init"
    exit 1
fi

if ! check_process "grepai watch" "-f"; then
    # Lock file pour éviter les race conditions (portable temp dir)
    LOCK_DIR="${TMPDIR:-${TEMP:-/tmp}}/grepai-watch.lock"
    if mkdir "$LOCK_DIR" 2>/dev/null; then
        echo "   Démarrage grepai watch dans $PROJECT_DIR..."
        cd "$PROJECT_DIR" || { echo -e "${RED}❌ Cannot access PROJECT_DIR: $PROJECT_DIR${NC}"; rmdir "$LOCK_DIR" 2>/dev/null; exit 1; }
        nohup grepai watch > /dev/null 2>&1 &
        disown
        sleep 1
        rmdir "$LOCK_DIR" 2>/dev/null || true
    else
        echo "   grepai watch semble déjà en cours de démarrage, attente..."
        sleep 2
    fi
fi

GREPAI_PID=$(get_pid "grepai watch" "-f")
if [[ -n "$GREPAI_PID" ]]; then
    echo -e "${GREEN}✅ grepai watch: running (PID $GREPAI_PID)${NC}"
else
    echo -e "${YELLOW}⚠️  grepai watch: démarré mais PID non trouvé${NC}"
fi

# --- Status ---
echo ""
echo "📊 Index status:"
cd "$PROJECT_DIR" || { echo -e "${RED}❌ Cannot access PROJECT_DIR: $PROJECT_DIR${NC}"; exit 1; }
if grepai status 2>/dev/null | head -5; then
    echo ""
    echo -e "${GREEN}🎉 Services grepai prêts !${NC}"
else
    echo -e "${YELLOW}⚠️  Index non trouvé. Exécutez 'grepai init' puis 'grepai index'${NC}"
fi
