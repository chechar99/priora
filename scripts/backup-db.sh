#!/usr/bin/env bash
# Descarga un backup consistente de la SQLite de producción al Mac.
#
# Uso:
#   ./scripts/backup-db.sh
#   ./scripts/backup-db.sh backups/mi-copia.db
#
# Requiere: deploy.env (VPS_HOST, VPS_USER, …) y acceso SSH al VPS.
# El archivo queda en backups/priora-YYYYMMDD-HHMMSS.db por defecto.
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="${DEPLOY_ENV:-$ROOT/deploy.env}"
CONTAINER_DB="/app/data/priora.db"
BACKUP_DIR="${PRIORA_BACKUP_DIR:-$ROOT/backups}"

usage() {
  cat <<'EOF'
Uso: ./scripts/backup-db.sh [destino.db]

Sin argumentos, guarda en backups/priora-YYYYMMDD-HHMMSS.db
(relativo a la raíz del repo).

Variables:
  DEPLOY_ENV          Ruta a deploy.env (default: ./deploy.env)
  PRIORA_BACKUP_DIR   Directorio por defecto (default: ./backups)

Ejemplos:
  ./scripts/backup-db.sh
  ./scripts/backup-db.sh ~/Desktop/priora-prod.db
EOF
  exit 1
}

case "${1:-}" in
  -h|--help|help) usage ;;
esac

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Falta $ENV_FILE — copiá deploy.env.example y completalo." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$ENV_FILE"
: "${VPS_HOST:?VPS_HOST requerido en deploy.env}"
: "${VPS_USER:=root}"

SSH_OPTS=(-o StrictHostKeyChecking=accept-new)
if [[ -n "${VPS_SSH_KEY:-}" ]]; then
  SSH_OPTS+=(-i "$VPS_SSH_KEY")
fi

stamp="$(date +"%Y%m%d-%H%M%S")"
if [[ -n "${1:-}" ]]; then
  DEST="$1"
  if [[ "$DEST" != /* ]]; then
    DEST="$ROOT/$DEST"
  fi
else
  mkdir -p "$BACKUP_DIR"
  DEST="$BACKUP_DIR/priora-$stamp.db"
fi

mkdir -p "$(dirname "$DEST")"
REMOTE_TMP="/tmp/priora-backup-$stamp.db"

echo "==> Buscando DB en ${VPS_USER}@${VPS_HOST}…"

# En el VPS: localizar volumen, hacer backup online (seguro con la API en marcha)
# y dejar el archivo en /tmp listo para scp.
ssh "${SSH_OPTS[@]}" "${VPS_USER}@${VPS_HOST}" \
  "CONTAINER_DB='$CONTAINER_DB' REMOTE_TMP='$REMOTE_TMP' bash -s" <<'REMOTE'
set -euo pipefail

CID="$(docker ps --format '{{.ID}} {{.Names}} {{.Image}}' | awk '/priora-api/ { print $1; exit }')"
if [[ -z "$CID" ]]; then
  CID="$(docker ps --format '{{.ID}} {{.Names}}' | awk '/priora/ && /api/ { print $1; exit }')"
fi
if [[ -z "$CID" ]]; then
  echo "No encontré contenedor priora-api en el VPS." >&2
  docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}' >&2 || true
  exit 1
fi

SRC="$(docker inspect -f '{{range .Mounts}}{{if eq .Destination "/app/data"}}{{.Source}}{{end}}{{end}}' "$CID")"
if [[ -z "$SRC" ]]; then
  echo "No pude localizar el volumen /app/data del contenedor API." >&2
  exit 1
fi

DB="$SRC/priora.db"
if [[ ! -f "$DB" ]]; then
  echo "No existe $DB en el VPS." >&2
  exit 1
fi

echo "Contenedor: $CID"
echo "Origen:     $DB"

# Backup online: copia consistente aunque haya escrituras (API de backup de SQLite).
if command -v python3 >/dev/null 2>&1; then
  python3 - "$DB" "$REMOTE_TMP" <<'PY'
import sqlite3, sys
src, dst = sys.argv[1], sys.argv[2]
src_conn = sqlite3.connect(src, timeout=30)
try:
    dst_conn = sqlite3.connect(dst)
    try:
        src_conn.backup(dst_conn)
    finally:
        dst_conn.close()
finally:
    src_conn.close()
print("Backup online (python) OK")
PY
elif command -v sqlite3 >/dev/null 2>&1; then
  sqlite3 -cmd ".timeout 30000" "$DB" ".backup '$REMOTE_TMP'"
  echo "Backup online (sqlite3) OK"
elif docker exec "$CID" which sqlite3 >/dev/null 2>&1; then
  docker exec -i "$CID" sqlite3 -cmd ".timeout 30000" "$CONTAINER_DB" \
    ".backup '/tmp/priora-backup-inner.db'"
  docker cp "$CID:/tmp/priora-backup-inner.db" "$REMOTE_TMP"
  docker exec "$CID" rm -f /tmp/priora-backup-inner.db
  echo "Backup online (sqlite3 en contenedor) OK"
else
  # Último recurso: copia en frío del archivo (puede quedar inconsistente si hay writes).
  echo "Aviso: sin python3/sqlite3 — copiando archivo en crudo." >&2
  cp -f "$DB" "$REMOTE_TMP"
  # Incluir WAL/SHM si existen (modo WAL)
  [[ -f "${DB}-wal" ]] && cp -f "${DB}-wal" "${REMOTE_TMP}-wal"
  [[ -f "${DB}-shm" ]] && cp -f "${DB}-shm" "${REMOTE_TMP}-shm"
fi

ls -lh "$REMOTE_TMP"
REMOTE

echo "==> Descargando → $DEST"
scp "${SSH_OPTS[@]}" "${VPS_USER}@${VPS_HOST}:$REMOTE_TMP" "$DEST"

# Limpiar temporales en el VPS (y posibles -wal/-shm del fallback)
ssh "${SSH_OPTS[@]}" "${VPS_USER}@${VPS_HOST}" \
  "rm -f '$REMOTE_TMP' '${REMOTE_TMP}-wal' '${REMOTE_TMP}-shm'"

bytes="$(wc -c < "$DEST" | tr -d ' ')"
echo "==> Listo: $DEST ($bytes bytes)"

if command -v sqlite3 >/dev/null 2>&1; then
  tables="$(sqlite3 "$DEST" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")"
  echo "    Tablas: $tables"
fi
