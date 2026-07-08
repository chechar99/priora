#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

case "${1:-}" in
  api)
    cd "$ROOT/priora-api"
    if [ ! -f .env ]; then cp .env.example .env; fi
    cargo run
    ;;
  web)
    cd "$ROOT/priora-web"
    npm run dev
    ;;
  install)
    cd "$ROOT/priora-api" && cp -n .env.example .env 2>/dev/null || true
    cd "$ROOT/priora-web" && npm install
    ;;
  *)
    echo "Usage: ./scripts/dev.sh [install|api|web]"
    echo ""
    echo "  install  — instalar dependencias"
    echo "  api      — iniciar backend (puerto 3000)"
    echo "  web      — iniciar frontend (puerto 5173)"
    exit 1
    ;;
esac
