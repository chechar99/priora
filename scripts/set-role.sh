#!/usr/bin/env bash
# Asigna rol a un usuario por email (admin | proponent | regular).
#
# Uso:
#   ./scripts/set-role.sh list
#   ./scripts/set-role.sh admin cesarrian@gmail.com
#   ./scripts/set-role.sh proponent vecino@ejemplo.com
#   ./scripts/set-role.sh regular vecino@ejemplo.com
#
# Producción (VPS vía SSH + contenedor API):
#   ./scripts/set-role.sh --prod list
#   ./scripts/set-role.sh --prod admin cesarrian@gmail.com
#
# Requiere: sqlite3 (local) o deploy.env + SSH (prod).
# El usuario debe haber iniciado sesión al menos una vez (fila en users).
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="${DEPLOY_ENV:-$ROOT/deploy.env}"
LOCAL_DB="${PRIORA_DB:-$ROOT/priora-api/priora.db}"
CONTAINER_DB="/app/data/priora.db"
VALID_ROLES="admin|proponent|regular"

TARGET=local
if [[ "${1:-}" == "--prod" || "${1:-}" == "-p" ]]; then
  TARGET=prod
  shift
fi

usage() {
  cat <<'EOF'
Uso: ./scripts/set-role.sh [--prod] <comando>

Comandos:
  list                         Listar usuarios (email, nombre, rol)
  admin <email>                Promover a administrador
  proponent <email>            Promover a proponente
  regular <email>              Dejar como usuario regular

Opciones:
  --prod, -p                   Ejecutar contra la DB del contenedor en el VPS
                               (usa deploy.env: VPS_HOST, VPS_USER, …)

Ejemplos:
  ./scripts/set-role.sh admin cesarrian@gmail.com
  ./scripts/set-role.sh --prod admin cesarrian@gmail.com
  ./scripts/set-role.sh --prod list
EOF
  exit 1
}

need_sqlite() {
  if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "Falta sqlite3. Instalalo (macOS: brew install sqlite)." >&2
    exit 1
  fi
}

sql_local() {
  need_sqlite
  if [[ ! -f "$LOCAL_DB" ]]; then
    echo "No existe $LOCAL_DB — ¿arrancaste la API alguna vez?" >&2
    exit 1
  fi
  sqlite3 -cmd ".timeout 5000" "$LOCAL_DB" "$1"
}

run_sql_prod() {
  local sql="$1"

  if [[ ! -f "$ENV_FILE" ]]; then
    echo "Falta $ENV_FILE — copiá deploy.env.example y completalo." >&2
    exit 1
  fi
  # shellcheck source=/dev/null
  source "$ENV_FILE"
  : "${VPS_HOST:?VPS_HOST requerido en deploy.env}"
  : "${VPS_USER:=root}"

  local ssh_opts=(-o StrictHostKeyChecking=accept-new)
  if [[ -n "${VPS_SSH_KEY:-}" ]]; then
    ssh_opts+=(-i "$VPS_SSH_KEY")
  fi

  # Base64 evita problemas de quoting con comillas en el SQL
  local sql_b64
  sql_b64="$(printf '%s' "$sql" | base64 | tr -d '\n')"

  ssh "${ssh_opts[@]}" "${VPS_USER}@${VPS_HOST}" \
    "SQL_B64='$sql_b64' CONTAINER_DB='$CONTAINER_DB' bash -s" <<'REMOTE'
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

SQL="$(printf '%s' "$SQL_B64" | base64 -d)"

# Preferir python3 en el host (escritura fiable sobre volúmenes Docker).
# sqlite3 CLI / imagen efímera suelen fallar con "readonly database".
if command -v python3 >/dev/null 2>&1; then
  python3 - "$DB" "$SQL" <<'PY'
import sqlite3, sys
db, sql = sys.argv[1], sys.argv[2]
conn = sqlite3.connect(db, timeout=5)
try:
    cur = conn.execute(sql)
    rows = cur.fetchall()
    conn.commit()
    for row in rows:
        print("|".join("" if c is None else str(c) for c in row))
finally:
    conn.close()
PY
elif command -v sqlite3 >/dev/null 2>&1; then
  sqlite3 -cmd ".timeout 5000" "$DB" "$SQL"
elif docker exec "$CID" which sqlite3 >/dev/null 2>&1; then
  docker exec -i "$CID" sqlite3 -cmd ".timeout 5000" "$CONTAINER_DB" "$SQL"
else
  docker run --rm --user root -v "$SRC:/data:rw" keinos/sqlite3:latest \
    sqlite3 -cmd ".timeout 5000" /data/priora.db "$SQL"
fi
REMOTE
}

run_sql() {
  local sql="$1"
  if [[ "$TARGET" == "prod" ]]; then
    run_sql_prod "$sql"
  else
    sql_local "$sql"
  fi
}

list_users() {
  local out
  out="$(run_sql "SELECT printf('%-40s %-28s %s', email, COALESCE(name,''), role) FROM users ORDER BY role DESC, email;")"
  if [[ -z "$out" ]]; then
    echo "No hay usuarios en la base ($TARGET)."
    echo "Iniciá sesión con Google al menos una vez y volvé a intentar."
    return
  fi
  printf '%-40s %-28s %s\n' "EMAIL" "NOMBRE" "ROL"
  printf '%s\n' "$out"
}

set_role() {
  local role="$1"
  local email="$2"

  if [[ ! "$role" =~ ^($VALID_ROLES)$ ]]; then
    echo "Rol inválido: $role (válidos: admin, proponent, regular)" >&2
    exit 1
  fi
  if [[ -z "$email" || "$email" != *@* ]]; then
    echo "Email inválido: ${email:-<vacío>}" >&2
    exit 1
  fi

  local email_sql="${email//\'/\'\'}"
  local now
  now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  local count
  count="$(run_sql "SELECT COUNT(*) FROM users WHERE lower(email) = lower('$email_sql');")"
  count="$(echo "$count" | tr -d '[:space:]')"

  if [[ "$count" == "0" ]]; then
    echo "No existe un usuario con email: $email" >&2
    echo "El usuario debe iniciar sesión con Google una vez antes de asignarle rol." >&2
    echo "" >&2
    echo "Usuarios actuales:" >&2
    list_users >&2 || true
    exit 1
  fi

  run_sql "UPDATE users SET role = '$role', updated_at = '$now' WHERE lower(email) = lower('$email_sql');" >/dev/null

  echo "OK ($TARGET): $email → rol '$role'"
  run_sql "SELECT printf('  %s | %s | %s', email, name, role) FROM users WHERE lower(email) = lower('$email_sql');"
}

CMD="${1:-}"
case "$CMD" in
  list|ls)
    list_users
    ;;
  admin|proponent|regular)
    [[ $# -ge 2 ]] || usage
    set_role "$CMD" "$2"
    ;;
  -h|--help|help|"")
    usage
    ;;
  *)
    echo "Comando desconocido: $CMD" >&2
    usage
    ;;
esac
