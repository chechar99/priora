# Despliegue en producción — Priora

> Dominio: **https://priora.ceapps.top**  
> Frontend: `https://priora.ceapps.top/for/{barrio}`  
> API: `https://priora.ceapps.top/api` (mismo origen → sin CORS)  
> Infraestructura: **Coolify** (dos aplicaciones en el mismo proyecto)

---

## 1. Arquitectura

```
https://priora.ceapps.top
         │
    Traefik (Coolify)
         │
    ┌────┴────────────────────────────────────┐
    │                                         │
 /api/*  /uploads/*                    /*  /for/*
    │                                         │
    ▼                                         ▼
 priora-api                            priora-web
 (Docker :3000)                         (nginx :80)
```

| Ruta | Ejemplo | Servicio |
|------|---------|----------|
| Frontend — selector | `/for` | priora-web |
| Frontend — barrio | `/for/barrio-test` | priora-web |
| Frontend — auth | `/login`, `/auth/callback`, `/completar-perfil` | priora-web |
| API global | `/api/auth/*`, `/api/namespaces`, `/api/categories` | priora-api |
| API por barrio | `/api/barrio-test/proposals` | priora-api |
| Archivos | `/uploads/*` | priora-api |

El cliente (`priora-web/src/api/client.js`) usa rutas relativas (`/api/...`) con `VITE_API_URL` vacío: mismo origen, sin CORS.

---

## 2. Requisitos previos

- Coolify en `https://coolify.ceapps.top`
- DNS **A**: `priora.ceapps.top` → IP del VPS
- Puertos **80** y **443** abiertos
- Puertos **80** y **443** abiertos
- **Deploy desde Mac** (recomendado): `scripts/deploy.sh` — sin GitHub

---

## 3. Proyecto Coolify

Proyecto **Priora** con dos aplicaciones tipo **Docker Image** (ya creadas):

| App | UUID | Imagen | Puerto | Dominio |
|-----|------|--------|--------|---------|
| `priora-api` | `arwyqa5vgjb1bp1yxz6pf3ma` | `127.0.0.1:5000/priora-api:latest` | 3000 | `/api`, `/uploads` |
| `priora-web` | `svwgcroypeuxqb71dq9s9ygw` | `127.0.0.1:5000/priora-web:latest` | 80 | `priora.ceapps.top` |

Volúmenes en API: `/app/data`, `/app/uploads`.

> **Strip Prefix:** en `priora-api`, Coolify puede agregar middleware que quita `/api` antes de llegar al backend. Hay que **desactivarlo** — el backend espera rutas con `/api/...`.

---

## 4. Backend (`priora-api`)

### Volúmenes

| Ruta en contenedor | Uso |
|--------------------|-----|
| `/app/data` | SQLite (`priora.db`) |
| `/app/uploads` | Imágenes de propuestas |

### Variables de entorno

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
GOOGLE_CLIENT_SECRET=<secret>
GOOGLE_REDIRECT_URI=https://priora.ceapps.top/api/auth/google/callback
```

### Dominio en Coolify

En **Domains**, agregar con prefijo de ruta (si la UI lo permite):

- `https://priora.ceapps.top/api`
- `https://priora.ceapps.top/uploads`

Si Coolify no permite path en dominio, dejar la app **sin dominio público** y enrutar con Traefik (§6).

### Health check

- Path: `/api/health`
- Puerto: `3000`

---

## 5. Frontend (`priora-web`)

### Build

El Dockerfile hace `npm ci && npm run build` y sirve con **nginx** (imagen final ~25 MB).

### Variables de build

| Variable | Valor |
|----------|-------|
| `VITE_API_URL` | *(vacío)* — peticiones a `/api` en el mismo dominio |

En Coolify → **Build Variables** dejar `VITE_API_URL` vacío o no definirla.

### Dominio en Coolify

- `https://priora.ceapps.top`

Traefik envía el resto del tráfico (incluido `/for/*`, `/login`, assets) al contenedor nginx.

### Rutas del frontend

Definidas en `priora-web/src/routes.js`:

| Ruta | Pantalla |
|------|----------|
| `/` | Redirige a `/for` |
| `/for` | Selector de barrio |
| `/for/:namespace` | Home del barrio |
| `/for/:namespace/propuestas/:id` | Detalle |
| `/login` | Login |
| `/auth/callback` | Callback OAuth (redirect del backend) |
| `/completar-perfil` | Completar dirección |

---

## 6. Traefik — mismo dominio, dos contenedores

Si ambas apps compiten por el mismo FQDN, configurar prioridades en  
`/data/coolify/proxy/dynamic/priora.yaml` (ajustar nombre del contenedor API):

```yaml
http:
  routers:
    priora-api:
      entryPoints: [http, https]
      rule: Host(`priora.ceapps.top`) && (PathPrefix(`/api`) || PathPrefix(`/uploads`))
      service: priora-api
      tls:
        certResolver: letsencrypt
      priority: 100

    priora-web:
      entryPoints: [http, https]
      rule: Host(`priora.ceapps.top`)
      service: priora-web
      tls:
        certResolver: letsencrypt
      priority: 1

  services:
    priora-api:
      loadBalancer:
        servers:
          - url: http://<contenedor-api>:3000

    priora-web:
      loadBalancer:
        servers:
          - url: http://<contenedor-web>:80
```

Obtener nombres de contenedor: `docker ps | grep priora`.

Reiniciar proxy: `docker restart coolify-proxy`.

---

## 7. Google OAuth

| Campo | Valor |
|-------|-------|
| Authorized JavaScript origins | `https://priora.ceapps.top` |
| Authorized redirect URIs | `https://priora.ceapps.top/api/auth/google/callback` |

El backend redirige tras OAuth a `https://priora.ceapps.top/auth/callback?token=...` (ruta del frontend, no de la API).

---

## 8. Asignar roles (`scripts/set-role.sh`)

Promueve o degrada usuarios por email (`admin` | `proponent` | `regular`). El rol se lee de la DB en cada request; no hace falta regenerar el JWT.

**Requisito:** el usuario debe haber iniciado sesión con Google al menos una vez (debe existir la fila en `users`).

### Local

```bash
chmod +x scripts/set-role.sh

./scripts/set-role.sh list
./scripts/set-role.sh admin cesarrian@gmail.com
./scripts/set-role.sh proponent vecino@ejemplo.com
./scripts/set-role.sh regular vecino@ejemplo.com
```

Usa la DB en `priora-api/priora.db` (o `PRIORA_DB` si la definís).

### Producción

Requiere `deploy.env` (mismo que el deploy) y acceso SSH al VPS.

```bash
./scripts/set-role.sh --prod list
./scripts/set-role.sh --prod admin cesarrian@gmail.com
./scripts/set-role.sh --prod proponent vecino@ejemplo.com
./scripts/set-role.sh --prod regular vecino@ejemplo.com
```

Tras cambiar el rol, recargar la app (o volver a entrar) para que el frontend tome el rol desde `/auth/me`.

### Backup de la base de datos

Descarga una copia consistente de la SQLite de producción (backup online vía API de SQLite, sin detener la API):

```bash
chmod +x scripts/backup-db.sh

./scripts/backup-db.sh                    # → backups/priora-YYYYMMDD-HHMMSS.db
./scripts/backup-db.sh ~/Desktop/copia.db
```

Requiere el mismo `deploy.env` y SSH al VPS. Los archivos en `backups/` están en `.gitignore`.

---

## 9. Deploy local desde Mac (`scripts/deploy.sh`)

Sin GitHub. Compilás en tu Mac, subís imágenes al registry local del VPS y Coolify redeploya.

### Setup (una vez)

```bash
cp deploy.env.example deploy.env
# Completar COOLIFY_TOKEN en deploy.env
./scripts/deploy.sh bootstrap
```

Agregar en Coolify → **priora-api** → Environment: `JWT_SECRET`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` (ver `priora-api/.env.production.example`).

### Cada release

```bash
./scripts/deploy.sh          # api + web
./scripts/deploy.sh api      # solo backend Rust
./scripts/deploy.sh web      # solo frontend (Vite → nginx)
```

### Qué hace el script

**API:** `docker build` local (Rust compilado en Docker) → imagen al registry del VPS.

**Web:** `npm run build` local → solo empaqueta `dist/` en nginx (~360 KB).

1. `docker save` → `scp` → `docker load` en el VPS
2. `docker push` al registry `127.0.0.1:5000`
3. `POST /api/v1/deploy` a Coolify (o redeploy desde UI / MCP)

### Traefik en Coolify

- **priora-api:** `/api` y `/uploads`, prioridad 100, **sin** Strip Prefix
- **priora-web:** excluir `/api` y `/uploads`, prioridad 1

Si `/api/health` devuelve HTML, revisá las labels Traefik.

---

## 10. Verificación

```bash
curl -s https://priora.ceapps.top/api/health          # ok
curl -sI https://priora.ceapps.top/for/barrio-test    # 200 HTML
curl -sI https://priora.ceapps.top/                   # 301/302 → /for
```

En el navegador:

1. `https://priora.ceapps.top/for` — selector de barrios
2. `https://priora.ceapps.top/for/barrio-test` — listado
3. Login con Google → callback en `/auth/callback` → vuelta al barrio

---

## 11. Checklist

```
[ ] deploy.env con COOLIFY_TOKEN
[ ] ./scripts/deploy.sh bootstrap
[ ] Variables JWT_SECRET y Google OAuth en priora-api (Coolify UI)
[ ] Strip Prefix desactivado en priora-api
[ ] ./scripts/deploy.sh
[ ] Primer admin: login con Google + ./scripts/set-role.sh --prod admin <email>
```

---

*Última actualización: julio 2026*
