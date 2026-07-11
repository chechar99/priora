#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CADDY_CONFIG="${CADDY_CONFIG:-$HOME/.config/caddy/Caddyfile}"
SNIPPET="$ROOT/Caddyfile.snippet"

ensure_caddy() {
  if ! command -v caddy >/dev/null 2>&1; then
    echo "Caddy no está instalado."
    echo "  macOS: brew install caddy"
    echo "  docs:  https://caddyserver.com/docs/install"
    exit 1
  fi
}

ensure_global_caddyfile() {
  if [ -f "$CADDY_CONFIG" ]; then
    return 0
  fi
  mkdir -p "$(dirname "$CADDY_CONFIG")"
  cat > "$CADDY_CONFIG" <<EOF
# Global local reverse proxy (shared across projects).
# Add each app as another http://{app}.localhost:8080 { ... } block.

{
	admin off
}

EOF
  cat "$SNIPPET" >> "$CADDY_CONFIG"
  echo "Creado $CADDY_CONFIG (con bloque Priora)."
}

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
  proxy)
    ensure_caddy
    ensure_global_caddyfile
    echo "Config: $CADDY_CONFIG"
    echo "Proxy:  http://priora.localhost:8080"
    echo "(Caddy global — sumá otras apps en ese archivo)"
    caddy run --config "$CADDY_CONFIG" --adapter caddyfile
    ;;
  install)
    cd "$ROOT/priora-api" && cp -n .env.example .env 2>/dev/null || true
    cd "$ROOT/priora-web" && npm install
    if ! command -v caddy >/dev/null 2>&1; then
      echo ""
      echo "Opcional: instalá Caddy para http://priora.localhost:8080"
      echo "  brew install caddy"
    else
      echo ""
      echo "Caddy OK ($(caddy version 2>/dev/null | head -1))"
      ensure_global_caddyfile
      echo "Config global: $CADDY_CONFIG"
      echo "Snippet Priora: $SNIPPET"
    fi
    echo ""
    echo "Si ya tenías priora-api/.env, actualizá PORT=3100 y"
    echo "FRONTEND_URL=http://priora.localhost:8080 (ver .env.example)."
    ;;
  *)
    echo "Usage: ./scripts/dev.sh [install|api|web|proxy]"
    echo ""
    echo "  install  — instalar dependencias (y chequear Caddy)"
    echo "  api      — backend en 127.0.0.1:3100"
    echo "  web      — Vite en 127.0.0.1:5190"
    echo "  proxy    — Caddy global → http://priora.localhost:8080"
    echo ""
    echo "Config Caddy: \${CADDY_CONFIG:-~/.config/caddy/Caddyfile}"
    echo "Snippet app:  Caddyfile.snippet"
    echo "Flujo: api → web → proxy (un solo Caddy para todos los proyectos)"
    echo "Abrí: http://priora.localhost:8080"
    exit 1
    ;;
esac
