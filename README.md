# Priora

Plataforma para crear, priorizar y discutir propuestas de mejora vecinal.

## Stack

- **Backend:** Rust (Axum) + SQLite
- **Frontend:** React (Vite)

## Inicio rápido

### 1. Instalar dependencias

```bash
chmod +x scripts/dev.sh
./scripts/dev.sh install
brew install caddy   # si aún no lo tenés
```

### 2. Tres terminales

```bash
./scripts/dev.sh api     # 127.0.0.1:3100
./scripts/dev.sh web     # 127.0.0.1:5190
./scripts/dev.sh proxy   # Caddy global → http://priora.localhost:8080
```

Abrí la app en **http://priora.localhost:8080** (API y frontend detrás del mismo host).

El proxy es compartido: `~/.config/caddy/Caddyfile`. El bloque de Priora está en `Caddyfile.snippet` (para copiar/actualizar el global).
## Probar el prototipo

En la pantalla de login hay **usuarios de prueba** (modo desarrollo):

| Usuario | Rol |
|---------|-----|
| `admin@priora.local` | Administrador |
| `proponente@priora.local` / Sofía Navarro | Proponente |
| `carlos.mendez@priora.local` (y otros) | Regular |

También puedes usar **Google OAuth** configurando las variables en `priora-api/.env`.

### Tests BDD (API)

```bash
cd priora-api && cargo test --test bdd
```

Escenarios Gherkin de membresía/aprobación por espacio. Ver `doc/especificaciones.md` §11.1.

### Flujo sugerido

1. Entra como **Juan Vecino** → completa tu dirección.
2. Explora el listado de propuestas (datos de ejemplo incluidos).
3. Ve a **Priorizar** y reordena las propuestas.
4. Comenta en una propuesta.
5. Entra como **María Proponente** → crea una nueva propuesta.
6. Entra como **Administrador** → cambia el estado de una propuesta.
7. Impersona a otro usuario: `http://priora.localhost:8080/?priora_as=carlos.mendez@priora.local`

### Impersonación

| Modo | URL | Requisito |
|------|-----|-----------|
| Admin | `/?priora_as=email@priora.local` | Sesión de administrador |
| Dev | `/?priora_as=email@priora.local` | `DEV_IMPERSONATION=true` en `.env` |

### Datos de prueba incluidos

- **6 usuarios** (admin, 2 proponentes, 3 vecinos)
- **8 propuestas** (activas, en análisis, rechazadas)
- **Comentarios** y **priorizaciones** de ejemplo (ranking visible al cargar)

## Estructura

```
priora/
├── Caddyfile.snippet # Bloque Priora → ~/.config/caddy/Caddyfile
├── doc/              # Especificaciones (español)
├── priora-api/       # Backend Rust
├── priora-web/       # Frontend React
└── scripts/          # Scripts de desarrollo
```

## Configuración

Copia `priora-api/.env.example` a `priora-api/.env`. La base de datos SQLite se crea automáticamente en `priora-api/priora.db`.

## Documentación

Ver [doc/especificaciones.md](doc/especificaciones.md) para el detalle funcional completo.
