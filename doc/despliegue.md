# Despliegue en producción — Priora

> Dominio de producción: **https://priora.ceapps.top**  
> Infraestructura prevista: VPS con **Coolify** (Traefik como proxy).

---

## 1. Arquitectura

Priora se despliega en **un solo dominio** con dos componentes:

| Componente | Tecnología | Contenedor |
|------------|------------|------------|
| **Backend** (`priora-api`) | Rust / Axum / SQLite | **Sí** — Docker en Coolify |
| **Frontend** (`priora-web`) | React / Vite (build estático) | **No** — archivos servidos por Traefik |

```
                    priora.ceapps.top
                           │
                    Traefik (Coolify)
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    /api/*            /uploads/*             /*
         │                 │                 │
         ▼                 ▼                 ▼
   priora-api         priora-api      /var/www/priora
   (Docker :3000)     (Docker :3000)   (HTML/JS/CSS)
```

**Por qué el frontend sin Docker:** el build de Vite genera archivos estáticos (~pocos MB). Servirlos directamente desde el host ahorra RAM y CPU frente a levantar un contenedor nginx adicional.

---

## 2. Requisitos previos

- VPS con Coolify operativo y Traefik en puertos 80/443.
- Registro DNS **A** apuntando `priora.ceapps.top` → IP del VPS.
- Proyecto en Google Cloud Console con OAuth 2.0 configurado.
- Repositorio git accesible desde Coolify (GitHub, GitLab, etc.).

---

## 3. Backend — Docker en Coolify

### 3.1 Dockerfile

El backend incluye un Dockerfile multi-stage en `priora-api/Dockerfile`:

- **Build:** imagen `rust:1-bookworm`, compila el binario en release.
- **Runtime:** imagen `debian:bookworm-slim` con `libsqlite3` y usuario no-root (`appuser`).
- **Puerto:** `3000`.
- **Health check:** `GET /api/health`.

### 3.2 Crear la aplicación en Coolify

1. **New Resource → Application**.
2. Conectar el repositorio y seleccionar la rama de producción.
3. **Base Directory:** `priora-api`.
4. **Build Pack:** Dockerfile (detecta `priora-api/Dockerfile` automáticamente).
5. **Puerto interno:** `3000`.
6. **Health check path:** `/api/health`.

### 3.3 Volúmenes persistentes

Montar en el contenedor (Coolify → Storage):

| Ruta en contenedor | Contenido |
|--------------------|-----------|
| `/app/data` | Base SQLite (`priora.db`) |
| `/app/uploads` | Logos de propuestas |

### 3.4 Variables de entorno

Copiar desde `priora-api/.env.production.example` y ajustar en Coolify:

```env
DATABASE_URL=sqlite:/app/data/priora.db?mode=rwc
JWT_SECRET=<secreto-largo-aleatorio>
HOST=0.0.0.0
PORT=3000
FRONTEND_URL=https://priora.ceapps.top

DEV_AUTH=false
DEV_IMPERSONATION=false
SEED_DEMO_DATA=false

GOOGLE_CLIENT_ID=<client-id>
GOOGLE_CLIENT_SECRET=<client-secret>
GOOGLE_REDIRECT_URI=https://priora.ceapps.top/api/auth/google/callback
```

> **Importante:** en producción `DEV_AUTH`, `DEV_IMPERSONATION` y `SEED_DEMO_DATA` deben estar en `false`.

### 3.5 Dominio en Coolify (solo API)

En la configuración de dominios de la aplicación, agregar rutas internas. Traefik enrutará `/api` y `/uploads` hacia el contenedor (ver §5). Por ahora publicar el servicio en la red interna de Coolify; el enrutamiento público se configura en el paso 5.

Alternativa simple: asignar dominio `priora.ceapps.top` en Coolify con path `/api` si la versión lo soporta directamente desde la UI.

---

## 4. Frontend — build estático (sin Docker)

### 4.1 Build local o en CI

Desde la raíz del repo:

```bash
cd priora-web
cp .env.production.example .env.production
npm ci
npm run build
```

Con `VITE_API_URL` vacío, el frontend usa rutas relativas (`/api/...`) contra el mismo dominio.

### 4.2 Publicar en el servidor

Copiar el contenido de `priora-web/dist/` al VPS:

```bash
rsync -avz --delete priora-web/dist/ root@<IP-VPS>:/var/www/priora/
```

Crear el directorio si no existe:

```bash
ssh root@<IP-VPS> 'mkdir -p /var/www/priora && chown -R www-data:www-data /var/www/priora'
```

### 4.3 Actualizaciones del frontend

Cada vez que cambie el frontend:

```bash
cd priora-web && npm run build
rsync -avz --delete dist/ root@<IP-VPS>:/var/www/priora/
```

No hace falta reiniciar Docker ni el backend salvo que cambien variables `VITE_*` (esas se embeben en el build).

---

## 5. Traefik — enrutamiento en un solo dominio

Agregar configuración dinámica en el servidor (`/data/coolify/proxy/dynamic/priora.yaml`). Ajustar el nombre del contenedor si Coolify lo genera distinto (`docker ps` para verificar).

```yaml
http:
  routers:
    priora-api:
      entryPoints:
        - http
        - https
      rule: Host(`priora.ceapps.top`) && (PathPrefix(`/api`) || PathPrefix(`/uploads`))
      service: priora-api
      tls:
        certResolver: letsencrypt
      priority: 100

    priora-web:
      entryPoints:
        - http
        - https
      rule: Host(`priora.ceapps.top`)
      service: priora-static
      tls:
        certResolver: letsencrypt
      priority: 1

  services:
    priora-api:
      loadBalancer:
        servers:
          - url: http://<nombre-contenedor-priora-api>:3000

    priora-static:
      loadBalancer:
        servers:
          - url: http://host.docker.internal:8081
```

Para servir archivos estáticos desde el host, levantar **lighttpd** en el VPS (sin Docker):

```bash
# Instalar una sola vez
apt install -y lighttpd

# Configuración mínima en /etc/lighttpd/conf-enabled/priora.conf
cat > /etc/lighttpd/conf-enabled/priora.conf << 'EOF'
server.document-root = "/var/www/priora"
server.port = 8081
server.bind = "127.0.0.1"
index-file.names = ( "index.html" )
url.rewrite-if-not-file = ( "^/(.*)$" => "/index.html" )
EOF

systemctl enable --now lighttpd
```

Traefik alcanza `host.docker.internal:8081` (ya configurado en Coolify proxy) y lighttpd sirve los estáticos en localhost.

Reiniciar el proxy tras cambios:

```bash
docker restart coolify-proxy
```

### Certificado SSL

Con `certResolver: letsencrypt` y el DNS apuntando al VPS, Traefik obtiene el certificado automáticamente. Los puertos 80 y 443 deben ser accesibles desde internet durante la emisión.

---

## 6. Google OAuth

En [Google Cloud Console](https://console.cloud.google.com/) → APIs & Services → Credentials:

| Campo | Valor |
|-------|-------|
| Authorized JavaScript origins | `https://priora.ceapps.top` |
| Authorized redirect URIs | `https://priora.ceapps.top/api/auth/google/callback` |

Las mismas URLs deben coincidir con `FRONTEND_URL` y `GOOGLE_REDIRECT_URI` en las variables del backend.

---

## 7. Checklist de despliegue

```
[ ] DNS A: priora.ceapps.top → IP del VPS
[ ] App Coolify creada (base dir: priora-api, Dockerfile)
[ ] Volúmenes /app/data y /app/uploads montados
[ ] Variables de entorno de producción configuradas
[ ] DEV_AUTH=false, SEED_DEMO_DATA=false
[ ] Google OAuth con URIs de producción
[ ] Frontend buildeado y rsync a /var/www/priora
[ ] lighttpd en 127.0.0.1:8081 sirviendo estáticos
[ ] Traefik dynamic config priora.yaml aplicada
[ ] HTTPS responde en https://priora.ceapps.top
[ ] GET https://priora.ceapps.top/api/health → ok
[ ] Login con Google funciona end-to-end
```

---

## 8. Desarrollo vs producción

| Aspecto | Desarrollo | Producción |
|---------|------------|------------|
| Frontend | Vite dev server `:5173` | Build estático + lighttpd |
| API | `cargo run` `:3000` | Docker Coolify `:3000` |
| Dominio | `localhost` | `priora.ceapps.top` |
| Auth dev | `DEV_AUTH=true` | `DEV_AUTH=false` |
| Datos demo | `SEED_DEMO_DATA=true` | `SEED_DEMO_DATA=false` |
| SQLite | `priora-api/priora.db` | `/app/data/priora.db` (volumen) |

---

## 9. Build local del Docker (verificación)

```bash
cd priora-api
docker build -t priora-api:local .
docker run --rm -p 3000:3000 \
  -e JWT_SECRET=test \
  -e FRONTEND_URL=http://localhost:5173 \
  -e DEV_AUTH=true \
  -v priora-data:/app/data \
  priora-api:local
curl http://localhost:3000/api/health
```

---

*Última actualización: julio 2026*
