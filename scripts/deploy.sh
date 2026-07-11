#!/usr/bin/env bash
# Deploy Priora desde tu Mac → VPS (Coolify, sin GitHub).
#
# Uso:
#   cp deploy.env.example deploy.env   # completar COOLIFY_TOKEN y secretos
#   ./scripts/deploy.sh bootstrap      # una vez: registry local en el VPS
#   ./scripts/deploy.sh              # api + web
#   ./scripts/deploy.sh api          # solo backend
#   ./scripts/deploy.sh web          # solo frontend
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="${DEPLOY_ENV:-$ROOT/deploy.env}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Falta $ENV_FILE — copiá deploy.env.example y completalo." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$ENV_FILE"

: "${VPS_HOST:?VPS_HOST requerido}"
: "${VPS_USER:=root}"
: "${DOCKER_PLATFORM:=linux/amd64}"
: "${PRIORA_API_IMAGE:=priora-api}"
: "${PRIORA_WEB_IMAGE:=priora-web}"
: "${IMAGE_TAG:=latest}"
: "${COOLIFY_URL:=https://coolify.ceapps.top}"
: "${PRIORA_API_APP_UUID:?PRIORA_API_APP_UUID requerido}"
: "${PRIORA_WEB_APP_UUID:?PRIORA_WEB_APP_UUID requerido}"
: "${REGISTRY_HOST:=127.0.0.1:5000}"

SSH_OPTS=(-o StrictHostKeyChecking=accept-new)
if [[ -n "${VPS_SSH_KEY:-}" ]]; then
  SSH_OPTS+=(-i "$VPS_SSH_KEY")
fi

ssh_vps() {
  ssh "${SSH_OPTS[@]}" "${VPS_USER}@${VPS_HOST}" "$@"
}

log() { echo "==> $*"; }

# --- Registry local en el VPS (sin Docker Hub) ---

bootstrap_registry() {
  log "Configurando registry local en ${VPS_HOST}:${REGISTRY_HOST}..."
  ssh_vps bash -s <<'REMOTE'
set -euo pipefail
REGISTRY_NAME=priora-registry
if ! docker inspect "$REGISTRY_NAME" >/dev/null 2>&1; then
  docker run -d -p 127.0.0.1:5000:5000 --restart=always --name "$REGISTRY_NAME" registry:2
  echo "Registry creado."
else
  echo "Registry ya existe."
fi
DAEMON=/etc/docker/daemon.json
if [[ ! -f "$DAEMON" ]] || ! grep -q '127.0.0.1:5000' "$DAEMON" 2>/dev/null; then
  python3 - <<'PY'
import json, pathlib
p = pathlib.Path("/etc/docker/daemon.json")
data = {}
if p.exists():
    data = json.loads(p.read_text() or "{}")
insecure = set(data.get("insecure-registries", []))
insecure.add("127.0.0.1:5000")
data["insecure-registries"] = sorted(insecure)
p.write_text(json.dumps(data, indent=2) + "\n")
print("daemon.json actualizado")
PY
  systemctl restart docker
  echo "Docker reiniciado (contenedores Coolify deberían volver solos)."
fi
REMOTE
  log "Bootstrap listo."
}

# --- Build en Mac ---

build_api() {
  log "Build API: compila Rust en Docker local → ${PRIORA_API_IMAGE}:${IMAGE_TAG}"
  docker build --platform "$DOCKER_PLATFORM" \
    -t "${PRIORA_API_IMAGE}:${IMAGE_TAG}" \
    "$ROOT/priora-api"
}

build_web() {
  log "Build Web: Vite local → solo empaqueta dist/ en nginx"
  (
    cd "$ROOT/priora-web"
    npm ci --silent
    VITE_API_URL= npm run build
  )
  if [[ ! -f "$ROOT/priora-web/dist/index.html" ]]; then
    echo "Error: priora-web/dist/index.html no existe tras el build." >&2
    exit 1
  fi
  docker build --platform "$DOCKER_PLATFORM" \
    -t "${PRIORA_WEB_IMAGE}:${IMAGE_TAG}" \
    "$ROOT/priora-web"
}

# --- Subir imagen al registry del VPS ---

push_to_vps_registry() {
  local local_image=$1
  local remote_name=$2
  local tar="/tmp/${remote_name}-${IMAGE_TAG}.tar.gz"

  log "Empaquetando ${local_image}:${IMAGE_TAG}..."
  docker save "${local_image}:${IMAGE_TAG}" | gzip > "$tar"

  log "Transfiriendo a ${VPS_HOST}..."
  scp "${SSH_OPTS[@]}" "$tar" "${VPS_USER}@${VPS_HOST}:/tmp/deploy-image.tar.gz"

  log "Cargando y publicando en ${REGISTRY_HOST}/${remote_name}:${IMAGE_TAG}..."
  ssh_vps bash -s <<REMOTE
set -euo pipefail
gunzip -c /tmp/deploy-image.tar.gz | docker load
docker tag ${local_image}:${IMAGE_TAG} ${REGISTRY_HOST}/${remote_name}:${IMAGE_TAG}
docker push ${REGISTRY_HOST}/${remote_name}:${IMAGE_TAG}
rm -f /tmp/deploy-image.tar.gz
REMOTE
  rm -f "$tar"
}

# --- Redeploy en Coolify ---

coolify_deploy() {
  local app_uuid=$1
  local label=$2
  if [[ -z "${COOLIFY_TOKEN:-}" ]]; then
    log "COOLIFY_TOKEN vacío — redeployá ${label} desde Coolify UI o completá deploy.env"
    return 0
  fi
  log "Coolify deploy: ${label} (${app_uuid})"
  curl -sf -X POST "${COOLIFY_URL}/api/v1/deploy" \
    -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json" \
    -d "{\"uuid\":\"${app_uuid}\",\"force\":true}"
  echo
}

deploy_api() {
  build_api
  push_to_vps_registry "$PRIORA_API_IMAGE" "priora-api"
  coolify_deploy "$PRIORA_API_APP_UUID" "priora-api"
}

deploy_web() {
  build_web
  push_to_vps_registry "$PRIORA_WEB_IMAGE" "priora-web"
  coolify_deploy "$PRIORA_WEB_APP_UUID" "priora-web"
}

# --- Main ---

cmd="${1:-all}"

case "$cmd" in
  bootstrap)
    bootstrap_registry
    ;;
  api)
    deploy_api
    ;;
  web)
    deploy_web
    ;;
  all|"")
    deploy_api
    deploy_web
    ;;
  *)
    echo "Uso: $0 [bootstrap|api|web|all]" >&2
    exit 1
    ;;
esac

log "Deploy terminado."
