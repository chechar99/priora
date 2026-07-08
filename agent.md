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
└── scripts/dev.sh     # Arranque local (paths relativos a la raíz)
```

**Importante:** Todos los paths de scripts y comandos deben ser relativos a la **raíz del repo** (`priora/`), no a `scripts/`.

## Stack

| Capa | Tecnología |
|------|------------|
| Backend | Rust, Axum, SQLx, SQLite |
| Frontend | React 18+, Vite, React Router, TanStack Query |
| Auth | JWT + Google OAuth (opcional) + dev login |
| DB | SQLite (`priora-api/priora.db`) |

## Desarrollo local

Desde la raíz del proyecto:

```bash
chmod +x scripts/dev.sh
./scripts/dev.sh install   # primera vez
./scripts/dev.sh api       # http://127.0.0.1:3000
./scripts/dev.sh web       # http://localhost:5173
```

Configuración: copiar `priora-api/.env.example` → `priora-api/.env`.

## Convenciones de idioma

| Ámbito | Idioma |
|--------|--------|
| `doc/` | Español |
| UI (frontend) | Español |
| Código, API, nombres de campos/estados | Inglés |

Estados de propuesta en DB/API: `activa`, `en_analisis`, `rechazada`.  
Roles: `regular`, `proponent`, `admin`.

## Backend (`priora-api/`)

### Estructura

```
src/
├── main.rs           # Entry, migraciones, seed
├── config.rs         # Variables de entorno
├── auth.rs           # JWT, usuarios, permisos
├── db.rs             # Seed de datos demo
├── ranking.rs        # Agregación Borda count
├── error.rs          # AppError + IntoResponse
├── models.rs         # Tipos y DTOs
└── handlers/
    ├── mod.rs        # Router, AuthSession, CORS
    ├── auth.rs       # OAuth, dev-login, impersonación
    ├── users.rs
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
| Usuarios | `GET/PATCH /users/me` |
| Propuestas | `GET/POST /proposals`, `GET/PATCH /proposals/:id`, `PATCH .../status`, `PATCH .../tracker` |
| Comentarios | `GET/POST /proposals/:id/comments`, `DELETE /comments/:id` |
| Ranking | `GET/PUT /rankings/me` |

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

## Frontend (`priora-web/`)

### Estructura

```
src/
├── api/client.js              # Cliente REST + token localStorage
├── context/AuthContext.jsx    # Sesión, impersonación
├── components/                # Layout, ImpersonationHandler, etc.
└── pages/                     # Home, Login, Prioritize, ...
```

### Proxy dev

Vite proxy en `vite.config.js`: `/api` y `/uploads` → `http://127.0.0.1:3000`.

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
