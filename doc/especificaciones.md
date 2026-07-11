# Priora — Especificaciones del prototipo

> Documento de especificaciones funcionales y técnicas para el prototipo de **Priora**.  
> Stack previsto: **Rust** (backend) · **React JS** (frontend).

---

## 1. Visión y objetivo

**Priora** es una aplicación web orientada a la participación ciudadana en el ámbito de un barrio o comunidad local. Su objetivo es facilitar la **creación**, **priorización** y **discusión** de propuestas de mejora vecinal.

Los residentes pueden:

- Ver las propuestas existentes ordenadas según la priorización colectiva.
- Expresar su preferencia relativa entre propuestas (priorización).
- Comentar y debatir cada propuesta en un foro asociado.

Solo usuarios con roles especiales (**Administrador** o **Proponente**) pueden crear propuestas. El resto de usuarios participa en la priorización y el debate.

---

## 2. Alcance del prototipo

### 2.1 Incluido en el prototipo

| Área | Alcance |
|------|---------|
| Autenticación | Google OAuth + registro de dirección obligatorio |
| Propuestas | Listado, detalle, creación y edición (roles restringidos), estados, categoría, timeline |
| Logos | Subida multipart (JPEG/PNG/WebP ≤ 2 MB) o URL; servidos desde `/uploads` |
| Categorías | Catálogo fijo con 6 categorías por defecto; filtro en listado |
| Priorización | Orden relativo de propuestas por usuario autenticado |
| Comentarios | Foro por propuesta, últimos 10 visibles en detalle; borrado por autor o admin |
| Filtros | Propuestas activas por defecto; opción de ver rechazadas |
| Roles | Administrador, Proponente, Usuario regular, Admin de espacio (por namespace) |
| Membresía | Aprobación opcional por espacio (`require_member_approval`) |
| Invitaciones | Link `/for/{slug}?invite=…` para unir vecinos; canje → membresía `active` |

### 2.2 Fuera de alcance (fase posterior)

- Notificaciones por correo o push.
- Moderación avanzada de comentarios (más allá de permisos básicos).
- Múltiples barrios o comunidades en una misma instancia (parcialmente cubierto por namespaces; falta UX avanzada).
- Aplicación móvil nativa.
- Integración con sistemas municipales externos.
- Historial completo de comentarios en la vista de detalle (solo últimos 10 en prototipo).

---

## 3. Roles y permisos

### 3.1 Usuario regular

Usuario autenticado con Google y con dirección registrada.

| Acción | Permitido |
|--------|-----------|
| Ver listado de propuestas | Sí |
| Ver detalle de propuesta | Sí |
| Priorizar propuestas (ordenar) | Sí |
| Comentar en propuestas | Sí |
| Crear propuestas | No |
| Cambiar estado de propuestas | No |
| Asignar tracker | No |
| Gestionar roles de otros usuarios | No |

### 3.2 Proponente

Usuario regular con permiso adicional para crear propuestas.

| Acción | Permitido |
|--------|-----------|
| Todo lo de usuario regular | Sí |
| Crear propuestas | Sí |
| Editar propuestas propias (solo en estado *activa*) | Sí (ver §5.4) |
| Cambiar estado / asignar tracker | No |

> Un **Administrador** puede promover a un usuario regular a **Proponente**.

### 3.3 Administrador (plataforma)

Usuario con control total sobre el ciclo de vida de las propuestas y la comunidad.

| Acción | Permitido |
|--------|-----------|
| Todo lo de usuario regular | Sí |
| Crear propuestas | Sí |
| Cambiar estado de cualquier propuesta | Sí |
| Asignar o cambiar tracker | Sí |
| Promover usuarios a Proponente / Admin | Sí |
| Crear espacios y asignar admin de espacio | Sí |
| Activar aprobación de miembros por espacio | Sí |
| Compartir / regenerar link de invitación del espacio | Sí |

### 3.4 Admin de espacio (`space_admin`)

Rol **por espacio** (tabla `namespace_members`), no global.

| Acción | Permitido |
|--------|-----------|
| Todo lo de proponente en ese espacio | Sí |
| Aprobar / rechazar solicitudes de membresía | Sí |
| Deshabilitar / rehabilitar usuarios del espacio | Sí |
| Activar/desactivar aprobación requerida del espacio | Sí |
| Compartir / regenerar link de invitación del espacio | Sí |
| Cambiar roles globales o crear espacios | No |

### 3.5 Aprobación de usuarios por espacio

Cada espacio tiene el flag `require_member_approval` (default **apagado**):

- **Apagado:** cualquiera con perfil completo puede priorizar y comentar; el ranking cuenta para todos.
- **Encendido:** el usuario ve un banner para solicitar autorización; puede priorizar pero **no cuenta** hasta `active`; **no puede comentar** hasta ser aprobado. Los admins ven la cola de pendientes en Configuración.
- **Invitación válida** (§3.7): el canje deja la membresía en `active` de inmediato (salta la cola de aprobación).

Estados de membresía: `pending` → `active` | `rejected` | `disabled`.

### 3.6 Matriz de permisos resumida

```
                    Ver  Priorizar*  Comentar*  Crear  Editar  Estado  Tracker  Roles  Invitar
Usuario regular      ✓      ✓           ✓         ✗      ✗       ✗       ✗        ✗       ✗
Proponente           ✓      ✓           ✓         ✓      propia  ✗       ✗        ✗       ✗
Admin de espacio     ✓      ✓           ✓         ✓      propia  ✗       ✗      miembros   ✓
Administrador        ✓      ✓           ✓         ✓      todas   ✓       ✓        ✓       ✓
```

\*Con `require_member_approval`, priorizar/comentar con efecto requieren membresía `active` (salvo canje de invitación).

### 3.7 Invitaciones al barrio

Crecimiento social: un admin comparte un link; el vecino entra sin depender de SEO.

| Aspecto | Detalle |
|---------|---------|
| Formato | `/for/{slug}?invite={invite_code}` |
| Quién genera | Admin de plataforma o `space_admin` (Configuración → Espacio) |
| Secreto | `namespaces.invite_code` **no** se expone en `GET /namespaces/{slug}` |
| Canje | Tras login + perfil completo → `POST /{ns}/membership/accept-invite` |
| Efecto | Membresía `active` (rol `regular`); si ya era `pending`/`rejected`, pasa a `active` |
| Regenerar | Invalida el link anterior (`POST /{ns}/invite`) |
| Usuario deshabilitado | No puede canjear; debe rehabilitarlo un admin |

**Flujo UI:**

1. Admin copia texto listo para compartir (incluye URL).
2. El invitado abre el link → banner “Te invitaron a {espacio}”.
3. Si no hay sesión → CTA de login; el código se guarda en `sessionStorage`.
4. Con perfil completo → canje automático y confirmación.

---

## 4. Autenticación y registro

### 4.1 Flujo de autenticación

1. El usuario accede a Priora y pulsa **Iniciar sesión con Google**.
2. El frontend redirige al flujo OAuth 2.0 de Google (autorización delegada al backend o al proveedor configurado).
3. Tras autenticación exitosa, el backend crea o recupera el registro de usuario vinculado al `sub` (ID único) de Google.
4. Si el usuario **no tiene dirección registrada**, se le redirige obligatoriamente al formulario de **completar perfil** antes de acceder al resto de la aplicación.
5. Si el perfil está completo, se redirige a la **página principal** (listado de propuestas).

### 4.2 Datos obtenidos de Google

| Campo | Uso |
|-------|-----|
| `sub` | Identificador único e inmutable del usuario |
| `email` | Contacto y visualización (si se expone en UI) |
| `name` | Nombre mostrado en comentarios y perfil |
| `picture` | Avatar opcional en comentarios |

No se almacena contraseña local: la autenticación es exclusivamente federada.

### 4.3 Completar perfil — Dirección

Tras el primer inicio de sesión, el usuario debe indicar su **dirección** para identificarse dentro del barrio.

**Campos del formulario (prototipo):**

| Campo | Tipo | Obligatorio | Descripción |
|-------|------|-------------|-------------|
| Calle y número | texto | Sí | Ej.: "Av. Corrientes 1234" |
| Piso / Depto | texto | No | Ej.: "3° B" |
| Ciudad / Barrio | texto | Sí | Referencia geográfica de la comunidad |
| Código postal | texto | No | Según convención local |

**Validaciones:**

- Longitud mínima de calle: 5 caracteres.
- No se permite acceder a rutas protegidas sin dirección completada (middleware en frontend y backend).
- La dirección puede editarse desde el perfil del usuario (prototipo: sí).

**Privacidad (prototipo):**

- La dirección completa **no** se muestra públicamente en comentarios ni en listados.
- Solo el nombre del usuario (de Google) aparece en comentarios.
- Los administradores pueden ver dirección para verificación vecinal (decisión de producto a confirmar).

### 4.4 Sesión

- El backend emite un **token de sesión** (JWT o cookie HTTP-only) tras OAuth exitoso.
- Duración sugerida: 7 días con renovación silenciosa si el usuario sigue activo.
- Cierre de sesión: invalidación del token en cliente y, si aplica, lista de revocación en servidor.

### 4.5 Impersonación de usuarios (suplantación)

Funcionalidad para que un **administrador** actúe temporalmente como otro usuario, útil para soporte, depuración y pruebas de permisos.

#### 4.5.1 Mecanismo — query string `priora_as`

Los administradores autenticados pueden impersonar visitando cualquier URL de la aplicación con el parámetro:

```
https://priora.ejemplo/?priora_as=<email_o_id>
```

| Parámetro | Valor | Descripción |
|-----------|-------|-------------|
| `priora_as` | email o UUID | Usuario objetivo a impersonar |

**Ejemplo:** `http://localhost:5173/?priora_as=carlos.mendez@priora.local`

**Flujo:**

1. El frontend detecta `priora_as` en la URL.
2. Llama a `GET /api/auth/impersonate?priora_as=...` con el token del administrador.
3. El backend valida que quien solicita sea **admin** y emite un nuevo JWT para el usuario objetivo.
4. El JWT incluye el claim `impersonator_id` con el ID del administrador original.
5. Se muestra una **banda de aviso** en la UI: *"Actuando como [usuario]"* con botón **Volver a [admin]**.
6. `POST /api/auth/stop-impersonate` restaura la sesión del administrador.

#### 4.5.2 Modo desarrollo — `DEV_IMPERSONATION`

Para pruebas locales sin Google OAuth, el backend acepta la variable de entorno:

```
DEV_IMPERSONATION=true
```

Cuando está activa:

- Cualquiera puede impersonar con `?priora_as=<email>` **sin estar autenticado**.
- No se registra `impersonator_id` (no hay sesión previa que restaurar).
- **Solo debe usarse en desarrollo.** Nunca habilitar en producción.

#### 4.5.3 Reglas de seguridad

| Regla | Descripción |
|-------|-------------|
| Solo admins (prod) | Sin `DEV_IMPERSONATION`, solo usuarios con rol `admin` pueden impersonar |
| Auditoría | El claim `impersonator_id` en el JWT permite identificar suplantación |
| UI visible | Siempre se muestra banner cuando hay impersonación activa |
| Sin escalada | No se puede impersonar para obtener rol superior al del objetivo |
| Desactivar en prod | `DEV_IMPERSONATION=false` en despliegue real |

#### 4.5.4 API

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/api/auth/impersonate?priora_as=` | Inicia impersonación |
| POST | `/api/auth/stop-impersonate` | Termina impersonación (requiere JWT con `impersonator_id`) |
| GET | `/api/auth/me` | Incluye campo `impersonator` si aplica |

---

## 5. Propuestas

### 5.1 Definición

Una **propuesta** es una idea de mejora vecinal presentada por un Proponente o Administrador. Incluye descripción, estado, seguimiento opcional y espacio de comentarios.

### 5.2 Campos de una propuesta

| Campo | Tipo | Obligatorio | Descripción |
|-------|------|-------------|-------------|
| `id` | UUID | — | Identificador interno |
| `titulo` | texto (max 200) | Sí | Título breve y descriptivo |
| `descripcion` | texto largo (max 5000) | Sí | Detalle de la propuesta |
| `logo_url` | URL / archivo | No | Imagen representativa (opcional) |
| `estado` | enum | Sí | Ver §5.3 |
| `tracker` | referencia usuario | No | Responsable de seguimiento (admin asigna) |
| `autor_id` | referencia usuario | Sí | Quien creó la propuesta |
| `category_id` | referencia categoría | Sí | Categoría temática (ver §5.7) |
| `creado_en` | timestamp | — | Fecha de creación |
| `actualizado_en` | timestamp | — | Última modificación |

### 5.3 Estados de una propuesta

| Estado | Código | Visible por defecto en home | Descripción |
|--------|--------|----------------------------|-------------|
| Activa | `activa` | Sí | Propuesta vigente, abierta a priorización y comentarios |
| En análisis | `en_analisis` | Sí | En evaluación por autoridades o administración; sigue visible y comentable |
| Rechazada | `rechazada` | No | Propuesta descartada; solo visible con filtro explícito |

**Transiciones permitidas (solo Administrador):**

```
activa ──────────► en_analisis
activa ──────────► rechazada
en_analisis ─────► activa
en_analisis ─────► rechazada
rechazada ───────► activa   (reapertura excepcional)
```

Los usuarios regulares y proponentes **no** pueden cambiar el estado.

### 5.4 Creación y edición

**Crear propuesta** (Proponente / Administrador / admin de espacio con permiso de creación):

1. Formulario: título, descripción, categoría (obligatoria), logo opcional (subida de imagen o URL).
2. Estado inicial: siempre `activa`.
3. Tras guardar, redirección al detalle de la propuesta creada.
4. Ruta UI: `/for/{ns}/propuestas/nueva`.

**Editar propuesta:**

- El **autor** puede editar título, descripción, categoría y logo mientras el estado sea `activa`.
- Un **Administrador** de plataforma puede editar cualquier propuesta en cualquier estado.
- Ruta UI: `/for/{ns}/propuestas/{id}/editar` (botón **Editar** en el detalle).
- Enviar `logo_url: ""` en el PATCH limpia el logo.
- Cambios de estado no borran comentarios ni priorizaciones.

### 5.5 Tracker (responsable de seguimiento)

- Campo opcional asignado por un **Administrador**.
- Referencia a un usuario del sistema (típicamente otro admin o persona de contacto).
- Se muestra en la página de detalle con nombre y, opcionalmente, avatar.
- No implica permisos adicionales en el prototipo; es informativo.

### 5.6 Logo / imagen

- Formatos aceptados: JPEG, PNG, WebP.
- Tamaño máximo: 2 MB.
- **Subida:** `POST /api/uploads/logo` (multipart, campo `file`) → `{ "url": "/uploads/{uuid}.ext" }`.
- **Almacenamiento:** directorio `uploads/` del API, servido en estático en `/uploads/…`.
- Alternativa: pegar una URL externa en el formulario (se guarda en `logo_url`).
- En detalle se muestra la imagen si hay `logo_url`; si no, no se fuerza placeholder en listado (opcional en UI).

### 5.7 Categorías

Cada propuesta pertenece a **una categoría** del catálogo predefinido. Las categorías facilitan la navegación y el filtrado en el listado.

**Campos de una categoría:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | texto (slug) | Identificador estable, ej.: `seguridad` |
| `name` | texto | Nombre visible en UI, ej.: `Seguridad` |

**Categorías por defecto (seed en migración):**

| id | name |
|----|------|
| `seguridad` | Seguridad |
| `transito` | Tránsito |
| `movilidad` | Movilidad |
| `recreacion` | Recreación |
| `convivencia` | Convivencia |
| `servicios` | Servicios |

**Reglas:**

- El catálogo es **cerrado** en el prototipo: no se crean categorías desde la UI.
- Al crear o editar una propuesta, `category_id` es **obligatorio** y debe existir en la tabla `categories`.
- El listado admite filtro opcional por categoría (`?category=seguridad`).
- Una propuesta tiene exactamente **una** categoría.

---

## 6. Priorización y ranking

### 6.1 Concepto

La **priorización** no es un voto binario (sí/no) ni un puntaje absoluto. Cada usuario autenticado define un **orden relativo** de preferencia entre las propuestas visibles. El listado de la página principal refleja el **consenso agregado** de todas las priorizaciones individuales.

### 6.2 Experiencia de usuario

**Opción A — Reordenar lista (recomendada para prototipo):**

- En la home, el usuario ve el ranking global actual.
- Botón **"Priorizar"** abre una vista donde arrastra propuestas para ordenarlas según su preferencia (de mayor a menor prioridad).
- Al guardar, se persiste su orden personal.
- El ranking global se recalcula.

**Opción B — Comparaciones pareadas (alternativa):**

- El sistema presenta pares de propuestas: "¿Cuál prefieres?"
- Tras suficientes comparaciones, se infiere un orden individual (algoritmo tipo Elo o rank centrado).

> Para el prototipo se recomienda la **Opción A** por simplicidad de implementación y claridad para el usuario.

### 6.3 Cálculo del ranking global

**Entrada:** Para cada usuario *u*, una permutación de propuestas activas (y en análisis) que haya priorizado: `P_u = [p1, p2, ..., pn]`.

**Método sugerido (prototipo) — Borda count:**

- A cada propuesta `p` en la posición `k` (0-indexed) del ranking de un usuario, se asignan `n - k` puntos.
- El score global de `p` es la suma de puntos de todos los usuarios que la incluyeron.
- El listado se ordena por score descendente; empates se resuelven por fecha de creación (más antigua primero) o aleatoriamente estable.

**Propuestas sin priorización de un usuario:**

- Si un usuario no ha priorizado aún, no aporta puntos al ranking.
- Se muestra un indicador en home: "Aún no has priorizado — ¡Tu opinión cuenta!" con enlace al flujo de priorización.

**Propuestas rechazadas:**

- No participan en el ranking ni en la priorización.

### 6.4 Reglas de negocio

- Solo usuarios con perfil completo (dirección) pueden priorizar.
- Un usuario puede actualizar su priorización en cualquier momento.
- Las nuevas propuestas se insertan al final del ranking global hasta acumular suficientes priorizaciones (o se muestran con badge "Nueva").
- El ranking se actualiza en tiempo casi real (recálculo al guardar priorización; cache de lectura opcional).

---

## 7. Comentarios y foro

### 7.1 Modelo

Cada propuesta tiene un **hilo de comentarios** de estilo foro: comentarios planos o con un nivel de respuesta (prototipo: **planos** para simplificar; respuestas anidadas en fase posterior).

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | UUID | Identificador |
| `propuesta_id` | UUID | Propuesta asociada |
| `autor_id` | UUID | Usuario que comenta |
| `contenido` | texto (max 2000) | Cuerpo del comentario |
| `creado_en` | timestamp | Fecha de publicación |
| `editado_en` | timestamp | Null si nunca editado |

### 7.2 Vista en detalle de propuesta

- Se muestran los **últimos 10 comentarios** ordenados por fecha descendente (más reciente primero).
- Si hay más de 10, enlace **"Ver todos los comentarios"** (página o modal con paginación — prototipo: paginación simple).

### 7.3 Publicar comentario

- Requiere autenticación y perfil completo.
- Disponible en propuestas `activa` y `en_analisis`.
- En propuestas `rechazada`: solo lectura de comentarios existentes; no se permiten nuevos (configurable).

### 7.4 Moderación (prototipo mínimo)

- El **autor** o un **Administrador** de plataforma pueden **eliminar** un comentario (confirmación en UI).
- Edición de comentario propio (ventana 15 min) y sistema de reportes: **fuera del prototipo actual**.

### 7.5 Visualización

Cada comentario muestra:

- Nombre del autor (desde Google); avatar si está disponible.
- Fecha de publicación.
- Contenido con saltos de línea preservados; sin HTML arbitrario (sanitización).
- Acción **Borrar** si el usuario es autor o admin.

---

## 8. Pantallas y flujos de UI

### 8.1 Mapa de pantallas

```
[Landing / Login]
       │
       ▼ (OAuth OK, sin dirección)
[Completar perfil — Dirección]
       │
       ▼
[Home — Listado priorizado] ◄──────────────────┐
       │                                        │
       ├──► [Priorizar — Reordenar]            │
       ├──► [Detalle propuesta]                 │
       │         ├──► [Editar propuesta]        │
       │         └──► [Todos los comentarios]   │
       ├──► [Crear propuesta] (roles)          │
       ├──► [Configuración] (admins / invite)  │
       └──► [Perfil usuario] ──────────────────┘
```

### 8.2 Home — Listado de propuestas

**Elementos:**

- Cabecera: logo Priora, nombre de usuario, enlace a perfil, cerrar sesión.
- Filtro de estado: toggle o pestañas — **Activas** (por defecto, incluye `activa` + `en_analisis`) | **Rechazadas**.
- Filtro de categoría: dropdown con **Todas** (por defecto) + las 6 categorías del catálogo.
- Lista ordenada por ranking global.
- Cada ítem de lista:
  - Posición en ranking (#1, #2, …)
  - Título
  - Extracto de descripción (primeras 150 caracteres)
  - Badge de categoría
  - Badge de estado (`en_analisis` resaltado)
  - Logo en miniatura si existe
  - Score o indicador visual de apoyo (opcional)
- CTA: **Priorizar** (si autenticado).
- CTA: **Nueva propuesta** (solo Proponente / Administrador).

**Usuario no autenticado (prototipo):**

- Puede ver listado y detalle en modo lectura.
- No puede priorizar ni comentar; se muestran CTAs de login.

### 8.3 Detalle de propuesta

**Secciones:**

1. **Cabecera:** título, logo (si hay), badges de estado y categoría; botón **Editar** si aplica (§5.4).
2. **Metadatos / seguimiento:** autor, fecha, categoría, tracker, historial (`timeline` de `proposal_events`).
3. **Descripción** completa.
4. **Acciones admin:** cambiar estado, asignar tracker (solo Administrador de plataforma).
5. **Comentarios:** últimos 10 + formulario de nuevo comentario; borrado por autor/admin.
6. **Navegación:** volver al listado.

### 8.4 Formulario crear / editar propuesta

- Título (input)
- Categoría (select obligatorio, opciones del catálogo)
- Descripción (textarea con contador de caracteres)
- Logo: file input con preview (subida multipart) **o** URL
- Botones: Guardar / Cancelar

### 8.5 Perfil de usuario

- Nombre y email (solo lectura, desde Google)
- Formulario de dirección (editable)
- Rol mostrado (solo lectura)
- Historial de propuestas creadas (si Proponente/Admin) — opcional en prototipo

### 8.6 Configuración del espacio (admins)

Ruta: `/settings` (usa el último espacio visitado).

- Toggle `require_member_approval`.
- **Invitar vecinos:** mostrar URL de invitación, copiar texto para compartir, regenerar código.
- Cola de autorizaciones pendientes y gestión de miembros (si aplica).

---

## 9. Arquitectura técnica

### 9.1 Vista general

```
┌─────────────────┐     HTTPS      ┌─────────────────┐
│   React (SPA)   │ ◄──────────────► │  API Rust       │
│   Vite + React  │    REST/JSON     │  (Axum/Actix)   │
└────────┬────────┘                  └────────┬────────┘
         │                                  │
         │ OAuth redirect                   │ SQL
         ▼                                  ▼
┌─────────────────┐                  ┌─────────────────┐
│  Google OAuth   │                  │  PostgreSQL     │
└─────────────────┘                  └─────────────────┘
```

### 9.2 Backend (Rust)

**Stack sugerido:**

| Componente | Tecnología |
|------------|------------|
| Framework HTTP | [Axum](https://github.com/tokio-rs/axum) o Actix-web |
| Runtime async | Tokio |
| ORM / queries | SQLx o Diesel |
| Base de datos | PostgreSQL |
| Autenticación | `oauth2` crate + JWT (`jsonwebtoken`) o sesiones con `tower-sessions` |
| Validación | `validator` |
| Serialización | `serde` + `serde_json` |
| Migraciones | SQLx migrate o Refinery |
| CORS | `tower-http` |

**Módulos lógicos:**

```
priora-api/
├── auth/          # OAuth, sesiones, middleware
├── users/         # Perfil, dirección, roles
├── proposals/     # CRUD, estados, tracker
├── categories/    # Listado de categorías
├── rankings/      # Priorización usuario, agregación Borda
├── comments/      # CRUD comentarios
└── uploads/       # Logos (POST /uploads/logo + ServeDir)
```

### 9.3 Frontend (React JS)

**Stack sugerido:**

| Componente | Tecnología |
|------------|------------|
| Build | Vite |
| UI | React 18+ |
| Enrutamiento | React Router v6 |
| Estado servidor | TanStack Query (React Query) |
| Estado cliente | Context API o Zustand (ligero) |
| HTTP | fetch o axios |
| Estilos | CSS Modules o Tailwind CSS |
| Drag & drop (priorizar) | `@dnd-kit/core` |
| OAuth | Redirección a endpoint backend `/auth/google` |

**Estructura de carpetas sugerida:**

```
priora-web/
├── src/
│   ├── api/           # Clientes REST
│   ├── components/    # UI reutilizable
│   ├── pages/         # Home, Detail, Profile, etc.
│   ├── hooks/
│   ├── context/       # AuthContext
│   └── utils/
```

### 9.4 API REST — Endpoints

Prefijo: `/api`. Las rutas de propuestas, comentarios, ranking y membresía van bajo `/{namespace}/…` (slug del espacio).

#### Autenticación

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/auth/google` | Inicia flujo OAuth |
| GET | `/auth/google/callback` | Callback OAuth |
| POST | `/auth/logout` | Cierra sesión |
| GET | `/auth/me` | Usuario actual (incluye `impersonator` si aplica) |
| GET | `/auth/impersonate` | Impersonar usuario (`?priora_as=`) |
| POST | `/auth/stop-impersonate` | Terminar impersonación |

#### Usuarios

| Método | Ruta | Descripción |
|--------|------|-------------|
| PATCH | `/users/me` | Actualizar dirección |
| GET | `/users/me` | Perfil completo |
| GET | `/users` | Listado (admin) |
| PATCH | `/users/:id/role` | Cambiar rol global (admin) |

#### Namespaces e invitaciones

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/namespaces` | Listado de espacios |
| POST | `/namespaces` | Crear espacio (admin) |
| GET | `/namespaces/:slug` | Detalle (**sin** `invite_code`) |
| PATCH | `/namespaces/:slug` | Actualizar (`require_member_approval`) |
| GET | `/{ns}/invite` | Obtener código y path de invitación (managers) |
| POST | `/{ns}/invite` | Regenerar código de invitación (managers) |
| POST | `/{ns}/membership/accept-invite` | Canjear invitación → membresía `active` |

#### Categorías

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/categories` | Listado de categorías disponibles |

#### Uploads

| Método | Ruta | Descripción |
|--------|------|-------------|
| POST | `/uploads/logo` | Multipart `file` → `{ url: "/uploads/…" }` |
| GET | `/uploads/:file` | Estático (fuera de `/api`, raíz del servidor) |

#### Propuestas (`/{ns}/…`)

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/proposals` | Listado con ranking y filtros (`?filter=active\|rejected&category=<id>`) |
| GET | `/proposals/:id` | Detalle (incluye `timeline`) |
| POST | `/proposals` | Crear (Proponente/Admin) |
| PATCH | `/proposals/:id` | Editar (autor en `activa` o admin); `logo_url: ""` limpia |
| PATCH | `/proposals/:id/status` | Cambiar estado (Admin) |
| PATCH | `/proposals/:id/tracker` | Asignar tracker (Admin) |

#### Priorización (`/{ns}/…`)

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/rankings/me` | Orden personal del usuario |
| PUT | `/rankings/me` | Guardar orden personal (`{ proposal_ids: [uuid, ...] }`) |

#### Comentarios (`/{ns}/…`)

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/proposals/:id/comments` | Listado paginado (`?limit=10&offset=0`) |
| POST | `/proposals/:id/comments` | Crear comentario |
| DELETE | `/comments/:id` | Eliminar (Admin o autor) |

### 9.5 Modelo de datos (PostgreSQL)

```sql
-- Usuarios
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    google_sub      TEXT UNIQUE NOT NULL,
    email           TEXT NOT NULL,
    name            TEXT NOT NULL,
    picture_url     TEXT,
    role            TEXT NOT NULL DEFAULT 'regular',  -- regular | proponent | admin
    street          TEXT,
    floor_apt       TEXT,
    city            TEXT,
    postal_code     TEXT,
    profile_complete BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Categorías
CREATE TABLE categories (
    id              TEXT PRIMARY KEY NOT NULL,
    name            TEXT UNIQUE NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO categories (id, name) VALUES
    ('seguridad', 'Seguridad'),
    ('transito', 'Tránsito'),
    ('movilidad', 'Movilidad'),
    ('recreacion', 'Recreación'),
    ('convivencia', 'Convivencia'),
    ('servicios', 'Servicios');

-- Propuestas
CREATE TABLE proposals (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title           TEXT NOT NULL,
    description     TEXT NOT NULL,
    logo_url        TEXT,
    status          TEXT NOT NULL DEFAULT 'activa',  -- activa | en_analisis | rechazada
    author_id       UUID NOT NULL REFERENCES users(id),
    tracker_id      UUID REFERENCES users(id),
    category_id     TEXT NOT NULL REFERENCES categories(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Priorización por usuario (orden explícito)
CREATE TABLE user_rankings (
    user_id         UUID NOT NULL REFERENCES users(id),
    proposal_id     UUID NOT NULL REFERENCES proposals(id),
    position        INT NOT NULL,  -- 0 = mayor prioridad
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, proposal_id)
);

-- Comentarios
CREATE TABLE comments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id     UUID NOT NULL REFERENCES proposals(id) ON DELETE CASCADE,
    author_id       UUID NOT NULL REFERENCES users(id),
    content         TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at       TIMESTAMPTZ
);
```

---

## 10. Requisitos no funcionales

| Categoría | Requisito |
|-----------|-----------|
| Seguridad | HTTPS obligatorio; cookies `Secure` + `HttpOnly`; validación de entrada en backend |
| Rendimiento | Listado home < 500 ms p95 con hasta 500 propuestas (con índices y cache de ranking) |
| Disponibilidad | Prototipo: despliegue single-instance aceptable |
| i18n | UI en español; código y API en inglés (convención técnica) |
| Accesibilidad | Contraste WCAG AA; navegación por teclado en formularios |
| Responsive | Diseño adaptable mobile-first (listado y detalle usables en móvil) |

---

## 11. Criterios de aceptación del prototipo

1. Un usuario puede iniciar sesión con Google y completar su dirección.
2. La home muestra propuestas activas y en análisis ordenadas por ranking global.
3. Un usuario autenticado puede guardar su priorización personal y ver el listado actualizado.
4. Un Proponente puede crear una propuesta con título, descripción, categoría y logo opcional (archivo o URL).
5. El autor puede editar su propuesta en estado `activa`; un admin puede editar cualquiera.
6. La página de detalle muestra campos, categoría, tracker, estado, timeline y últimos 10 comentarios.
7. Los usuarios pueden publicar comentarios en propuestas activas y en análisis; autor o admin pueden borrarlos.
8. El filtro permite ver propuestas rechazadas y filtrar por categoría; por defecto se muestran activas sin filtro de categoría.
9. Un Administrador puede cambiar el estado y asignar tracker.
10. Usuarios sin rol especial no ven el botón de crear propuesta.
11. El catálogo de categorías incluye las 6 categorías por defecto del seed.
12. Por defecto un espacio **no** exige aprobación: cualquier usuario con perfil completo puede priorizar y comentar con efecto.
13. Con `require_member_approval` activo, el usuario no autorizado puede guardar priorización pero **no cuenta** en el ranking y **no puede comentar**.
14. El usuario puede solicitar autorización; queda en `pending` hasta que un admin (plataforma o de espacio) apruebe o rechace.
15. Tras aprobación (`active`), priorización y comentarios tienen efecto; tras rechazo (`rejected`) siguen bloqueados.
16. Un admin de espacio puede aprobar solicitudes y deshabilitar miembros regulares de su espacio.
17. Un admin puede compartir un link de invitación; al canjearlo el usuario queda `active` sin pasar por la cola.

### 11.1 Pruebas BDD (API)

Los escenarios de membresía viven en Gherkin (español) y se ejecutan con cucumber-rs:

```bash
cd priora-api && cargo test --test bdd
```

| Feature | Escenarios cubiertos |
|---------|----------------------|
| `tests/features/membership_approval.feature` | Participación libre sin aprobación; bloqueo de comentarios con aprobación; solicitud + priorización sin efecto + aprobación; rechazo; aprobación por admin de espacio |
| `tests/features/admin_settings.feature` | `membership/me` para admin y regular; toggle de aprobación; listado de miembros (admin sí / regular no); admin de espacio puede administrar |

---

## 12. Riesgos y decisiones abiertas

| Tema | Opciones | Recomendación prototipo |
|------|----------|-------------------------|
| Algoritmo de ranking | Borda count vs Elo vs promedio de posiciones | Borda count |
| UI de priorización | Drag & drop vs votación por pares | Drag & drop |
| Comentarios anidados | Planos vs árbol | Planos |
| Visibilidad de dirección | Solo admin vs privada total | Solo admin (también visible a admin de espacio al aprobar) |
| Usuarios anónimos | Solo lectura vs redirección a login | Solo lectura en listado/detalle |
| Primer administrador | Seed en DB vs variable de entorno vs script | `scripts/set-role.sh` (ver `doc/despliegue.md` §8) |
| Aprobación al activar el toggle | Resetear miembros existentes vs solo nuevos | Solo nuevos: quien ya está `active` sigue; quien no tiene membresía debe solicitar |
| Almacenamiento de logos | Object storage vs directorio local | Directorio `uploads/` en el API (prototipo) |
| Invitación vs aprobación | ¿El invite salta la cola? | Sí: canje → `active` de inmediato |

---

## 13. Próximos pasos de implementación

1. ~~Inicializar monorepo~~ `priora-api` + `priora-web`.
2. ~~Autenticación, propuestas, ranking, comentarios, namespaces~~.
3. ~~Membresía por espacio y aprobación opcional~~.
4. ~~Edición de propuestas, borrado de comentarios, upload de logo~~.
5. ~~Invitaciones al barrio (`?invite=`)~~.
6. Ampliar cobertura BDD a auth, propuestas, ranking e invitaciones.
7. Notificaciones, dashboard de admins y resto del backlog de potenciación.

---

## 14. Despliegue en producción

| Aspecto | Valor |
|---------|-------|
| Dominio | **https://priora.ceapps.top** |
| Frontend | `/for/{barrio}` — app Coolify `priora-web` (nginx) |
| API | `/api` — app Coolify `priora-api` (Docker) |
| CORS | No aplica (mismo origen) |
| Base de datos | SQLite en volumen persistente (`/app/data`) |

Guía detallada: [`doc/despliegue.md`](despliegue.md).

---

## 15. Modelo de datos — membresía e invitaciones (SQLite)

```sql
-- Flag por espacio (default 0 = aprobación automática / libre)
-- namespaces.require_member_approval INTEGER NOT NULL DEFAULT 0
-- namespaces.invite_code TEXT UNIQUE  -- no se serializa en GET público

CREATE TABLE namespace_members (
    namespace_id TEXT NOT NULL REFERENCES namespaces(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    role TEXT NOT NULL DEFAULT 'regular',   -- regular | proponent | space_admin
    status TEXT NOT NULL DEFAULT 'active',  -- pending | active | disabled | rejected
    requested_at TEXT NOT NULL,
    reviewed_at TEXT,
    reviewed_by TEXT REFERENCES users(id),
    PRIMARY KEY (namespace_id, user_id)
);
```

API relevante (prefijo `/api`):

| Método | Ruta | Quién |
|--------|------|--------|
| GET | `/{ns}/membership/me` | autenticado / anónimo |
| POST | `/{ns}/membership/request` | usuario con perfil (si hay aprobación) |
| POST | `/{ns}/membership/accept-invite` | usuario con perfil + código válido |
| GET | `/{ns}/members?status=` | admin plataforma o space_admin |
| PATCH | `/{ns}/members/{user_id}` | admin plataforma o space_admin |
| PATCH | `/namespaces/{slug}` | admin plataforma o space_admin (`require_member_approval`) |
| GET | `/{ns}/invite` | admin plataforma o space_admin |
| POST | `/{ns}/invite` | regenerar código (admin plataforma o space_admin) |

---

*Versión del documento: 1.2 — Edición, logos, borrado de comentarios e invitaciones*  
*Última actualización: julio 2026*
