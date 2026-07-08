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
```

### 2. Iniciar el backend (terminal 1)

```bash
./scripts/dev.sh api
```

API en `http://127.0.0.1:3000`

### 3. Iniciar el frontend (terminal 2)

```bash
./scripts/dev.sh web
```

App en `http://localhost:5173`

## Probar el prototipo

En la pantalla de login hay **usuarios de prueba** (modo desarrollo):

| Usuario | Rol |
|---------|-----|
| Juan Vecino | Usuario regular |
| María Proponente | Proponente |
| Administrador | Admin |

También puedes usar **Google OAuth** configurando las variables en `priora-api/.env`.

### Flujo sugerido

1. Entra como **Juan Vecino** → completa tu dirección.
2. Explora el listado de propuestas (datos de ejemplo incluidos).
3. Ve a **Priorizar** y reordena las propuestas.
4. Comenta en una propuesta.
5. Entra como **María Proponente** → crea una nueva propuesta.
6. Entra como **Administrador** → cambia el estado de una propuesta.
7. Impersona a otro usuario: `http://localhost:5173/?priora_as=carlos.mendez@priora.local`

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
├── doc/              # Especificaciones (español)
├── priora-api/       # Backend Rust
├── priora-web/       # Frontend React
└── scripts/          # Scripts de desarrollo
```

## Configuración

Copia `priora-api/.env.example` a `priora-api/.env`. La base de datos SQLite se crea automáticamente en `priora-api/priora.db`.

## Documentación

Ver [doc/especificaciones.md](doc/especificaciones.md) para el detalle funcional completo.
