# Priora — Guía para agentes

Contexto del proyecto para asistentes de código. Leer esto antes de implementar cambios.

## Qué es Priora

Aplicación web de participación vecinal: los residentes **ven**, **priorizan** (orden relativo) y **comentan** propuestas de mejora del barrio. Solo **Proponentes** y **Administradores** pueden crear propuestas.

Especificación funcional completa (en español): [`doc/especificaciones.md`](doc/especificaciones.md).  
Guía de despliegue en producción: [`doc/despliegue.md`](doc/despliegue.md) — dominio **priora.ceapps.top**.

## Monorepo

```
priora/
├── agent.md           # Este archivo
├── doc/               # Documentación funcional (solo español)
├── priora-api/        # Backend Rust
├── priora-web/        # Frontend React
└── scripts/
    ├── dev.sh         # Arranque local
    ├── deploy.sh      # Deploy a Coolify/VPS
    ├── set-role.sh    # Asignar roles (admin/proponent/regular)
    └── backup-db.sh   # Backup SQLite de prod → local
```

**Importante:** Todos los paths de scripts y comandos deben ser relativos a la **raíz del repo** (`priora/`), no a `scripts/`.

## Stack

| Capa | Tecnología |
|------|------------|
| Backend | Rust, Axum, SQLx, SQLite |
| Frontend | React 18+, Vite, React Router, TanStack Query |
| Auth | JWT + Google OAuth (opcional) + dev login |
| DB | SQLite (`priora-api/priora.db`) |

## Deploy local desde Mac (`scripts/deploy.sh`)

Sin GitHub: build en Mac → registry local en VPS → redeploy Coolify API.

```bash
cp deploy.env.example deploy.env
./scripts/deploy.sh bootstrap   # una vez
./scripts/deploy.sh             # cada release
```

Detalle: [`doc/despliegue.md`](doc/despliegue.md) §9.

### Roles (`scripts/set-role.sh`)

```bash
./scripts/set-role.sh --prod list
./scripts/set-role.sh --prod admin cesarrian@gmail.com
```

Detalle: [`doc/despliegue.md`](doc/despliegue.md) §8.

### Backup DB (`scripts/backup-db.sh`)

```bash
./scripts/backup-db.sh                    # → backups/priora-YYYYMMDD-HHMMSS.db
./scripts/backup-db.sh ~/Desktop/copia.db
```

Requiere `deploy.env` + SSH. Detalle: [`doc/despliegue.md`](doc/despliegue.md) §8.

## Desarrollo local

Desde la raíz del proyecto:

```bash
chmod +x scripts/dev.sh
./scripts/dev.sh install   # primera vez (+ brew install caddy)
./scripts/dev.sh api       # 127.0.0.1:3100
./scripts/dev.sh web       # 127.0.0.1:5190
./scripts/dev.sh proxy     # http://priora.localhost:8080 (Caddy global)
```

Configuración: copiar `priora-api/.env.example` → `priora-api/.env`.  
App local: **http://priora.localhost:8080**.  
Caddy global: `~/.config/caddy/Caddyfile` (snippet del repo: `Caddyfile.snippet`).
## Convenciones de idioma

| Ámbito | Idioma |
|--------|--------|
| `doc/` | Español |
| UI (frontend) | Español |
| Código, API, nombres de campos/estados | Inglés |

Estados de propuesta en DB/API: `activa`, `en_analisis`, `rechazada`.  
Roles globales: `regular`, `proponent`, `admin`.  
Roles por espacio (`namespace_members.role`): `regular`, `proponent`, `space_admin`.  
Estados de membresía: `pending`, `active`, `disabled`, `rejected`.

## Backend (`priora-api/`)

### Estructura

```
src/
├── main.rs           # Entry, migraciones, seed
├── config.rs         # Variables de entorno
├── auth.rs           # JWT, usuarios, permisos
├── membership.rs     # Membresía por espacio y aprobación
├── db.rs             # Seed de datos demo
├── ranking.rs        # Agregación Borda count
├── error.rs          # AppError + IntoResponse
├── models.rs         # Tipos y DTOs
└── handlers/
    ├── mod.rs        # Router, AuthSession, CORS
    ├── auth.rs       # OAuth, dev-login, impersonación
    ├── users.rs
    ├── namespaces.rs
    ├── membership.rs # Solicitar / aprobar / miembros
    ├── proposals.rs
    ├── comments.rs
    └── rankings.rs
migrations/           # SQLx migrate
```

### API base

Prefijo: `/api`. Health: `GET /api/health`.

| Área | Rutas principales |
|------|-------------------|
| Auth | `GET /auth/google`, `POST /auth/dev-login`, `GET /auth/impersonate?priora_as=`, `POST /auth/stop-impersonate`, `GET /auth/me` |
| Usuarios | `GET/PATCH /users/me`, `GET /users` (admin), `PATCH /users/:id/role` (admin) |
| Namespaces | `GET/POST /namespaces` (POST admin), `GET/PATCH /namespaces/:slug` |
| Membresía | `GET /{ns}/membership/me`, `POST /{ns}/membership/request`, `GET /{ns}/members`, `PATCH /{ns}/members/:user_id` |
| Propuestas | `GET/POST /proposals`, `GET/PATCH /proposals/:id`, `PATCH .../status`, `PATCH .../tracker` |
| Comentarios | `GET/POST /proposals/:id/comments`, `DELETE /comments/:id` |
| Ranking | `GET/PUT /rankings/me` |

### Membresía por espacio

- Flag `namespaces.require_member_approval` (default `false`): con la app recién lanzada la participación es libre.
- Si está activo: priorización se guarda pero **no cuenta** en Borda hasta `status=active`; comentar requiere membresía activa.
- `space_admin`: permisos de proponente en el espacio + aprobar/rechazar/deshabilitar miembros.
- Admin de plataforma siempre puede gestionar cualquier espacio.

### Autenticación

- JWT en header `Authorization: Bearer <token>`.
- `AuthSession` extractor: usuario actual + `impersonator_id` opcional en claims.
- Rutas públicas (sin token): listado/detalle de propuestas y lectura de comentarios.
- `ensure_profile()` en handlers que requieren dirección completada.

### Variables de entorno relevantes

| Variable | Uso |
|----------|-----|
| `DATABASE_URL` | SQLite (default `sqlite:priora.db?mode=rwc`) |
| `DEV_AUTH` | Login de prueba sin Google |
| `DEV_IMPERSONATION` | Impersonar con `?priora_as=` sin sesión |
| `SEED_DEMO_DATA` | Carga usuarios/propuestas demo al arrancar |
| `IMPERSONATE_QUERY_KEY` | Clave query (default `priora_as`) |
| `FRONTEND_URL` | CORS y redirects OAuth |

### Impersonación

- Admin autenticado o `DEV_IMPERSONATION=true`.
- Query: `priora_as=<email|uuid>`.
- JWT incluye `impersonator_id` cuando un admin suplanta a otro usuario.
- Ver §4.5 en `doc/especificaciones.md`.

### Ranking

Algoritmo **Borda count**: cada usuario ordena propuestas activas/en_análisis; puntos = `n - posición`. Implementado en `ranking.rs`.

Si el espacio tiene `require_member_approval`, solo cuentan rankings de miembros `active` (y admins de plataforma).

### Tests BDD

```bash
cd priora-api && cargo test --test bdd
```

Features Gherkin (español) en `priora-api/tests/features/`. Harness cucumber-rs (`tests/bdd.rs`, `harness = false`).

Cobertura actual: aprobación de usuarios por espacio y configuración/admin del espacio. Detalle en `doc/especificaciones.md` §11.1.

## Frontend (`priora-web/`)

### Rutas

Prefijo de barrio: `/for/{namespace}` (ver `src/routes.js`).

| Ruta | Pantalla |
|------|----------|
| `/for` | Selector de barrio |
| `/for/:namespace` | Home |
| `/settings` | Admin de plataforma o admin de espacio: toggle de aprobación, autorizaciones, miembros (usa el último espacio visitado) |
| `/login`, `/auth/callback`, `/completar-perfil` | Auth (fuera de `/for`) |

La API usa `/api` en el mismo dominio en producción (`VITE_API_URL` vacío).

### Estructura

```
src/
├── api/client.js              # Cliente REST + token localStorage
├── context/AuthContext.jsx    # Sesión, impersonación
├── components/                # Layout, ImpersonationHandler, etc.
└── pages/                     # Home, Login, Prioritize, ...
```

### Proxy dev

Caddy global (`~/.config/caddy/Caddyfile`): `priora.localhost:8080` → Vite `:5190` + API `:3100`.  
Snippet en repo: `Caddyfile.snippet`. Vite también proxea `/api` y `/uploads` → `http://127.0.0.1:3100`.

### Flujos clave

1. Login → dev users o Google OAuth → callback con `?token=`.
2. Sin dirección → `/completar-perfil`.
3. `ImpersonationHandler` lee `?priora_as=` y llama `/api/auth/impersonate`.
4. Banner de impersonación en `Layout.jsx` si `impersonator` está presente.

## Datos de prueba

Con `SEED_DEMO_DATA=true`, al arrancar el API se upsertan:

- 6 usuarios (`admin@priora.local`, `proponente@priora.local`, `carlos.mendez@priora.local`, etc.)
- 8 propuestas, comentarios y priorizaciones de ejemplo

Para reset limpio: borrar `priora-api/priora.db` y reiniciar el API.

## Principios al modificar código

1. **Cambios mínimos** — no refactorizar fuera del alcance pedido.
2. **Seguir patrones existentes** — handlers Axum, React funcional, TanStack Query.
3. **No editar `doc/`** salvo que el usuario pida actualizar especificaciones.
4. **No commitear** `.env`, `priora.db` ni `target/`.
5. **Scripts** — resolver raíz con `$(dirname "$0")/..` desde `scripts/`.
6. **SQLite** — no asumir PostgreSQL; UUIDs como `TEXT`, booleanos como `INTEGER`.

## Criterios de aceptación del prototipo

Resumen (detalle en `doc/especificaciones.md` §11):

- Login (Google o dev) + dirección obligatoria
- Listado priorizado + filtro rechazadas
- Priorización drag & drop
- Crear propuestas (proponent/admin)
- Detalle con comentarios
- Admin: cambiar estado
- Impersonación operativa en dev
- Membresía por espacio: toggle de aprobación, solicitud, aprobar/rechazar (BDD: `cargo test --test bdd`)
