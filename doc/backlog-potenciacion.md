# Priora — Backlog de potenciación

Ideas para potenciar Priora más allá del prototipo actual.  
Orden sugerido: cerrar el ciclo cívico → adopción → diferenciación → fase 2.

**Cómo usarlo:** trabajar una a una. Marcar estado y anotar decisiones / PR al cerrar cada ítem.

| Estado | Significado |
|--------|-------------|
| `pendiente` | Sin empezar |
| `en curso` | En desarrollo |
| `hecho` | Entregado |
| `descartado` | Decidimos no hacerlo |

---

## Orden de trabajo sugerido

1. ~~Seguimiento visible de propuestas (tracker + estados)~~ ✅
2. Notificaciones (email primero)
3. ~~Cerrar superficies incompletas (edit / delete / upload)~~ ✅
4. ~~Invitaciones al barrio~~ ✅
5. Dashboard para space admins
6. Explicar el ranking Borda
7. Consenso vs conflicto
8. Historia del perfil
9. Adjuntos / evidencia
10. Digest semanal del barrio
11. Moderación básica
12. Roles más claros en onboarding
13. Fase 2 (integraciones, mapa, app nativa, etc.)

---

## Alto impacto

### 1. Seguimiento visible de propuestas

| Campo | Valor |
|-------|-------|
| Estado | `hecho` |
| Prioridad | Alta |

**Qué:** Mostrar responsable (tracker), estado (`activa` → `en_analisis` → resultado) y una línea de tiempo corta en el detalle de la propuesta.

**Por qué:** Convierte Priora de “encuesta” en herramienta de gestión vecinal. El tracker ya existe en API; falta UI y trazabilidad.

**Notas / decisiones:**

- Tabla `proposal_events` + campo `timeline` en detalle de propuesta.
- Admin puede cambiar estado (transiciones según spec) y asignar tracker desde el detalle.
- Panel “Seguimiento” en sidebar con responsable e historial.

---

### 2. Notificaciones (email primero)

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Alta |

**Qué:** Avisar por correo cuando: membresía aprobada/rechazada, propuesta nueva en el barrio, cambio de estado, respuesta a un comentario.

**Por qué:** Sin feedback externo, el ranking se olvida entre visitas.

**Notas / decisiones:**

-

---

### 3. Cerrar superficies incompletas

| Campo | Valor |
|-------|-------|
| Estado | `hecho` |
| Prioridad | Alta |

**Qué:** Completar lo que ya está (parcialmente) en API/specs:

- Editar propuesta (UI)
- Borrar comentarios (UI)
- Subir logo (multipart, no solo URL)

**Por qué:** Detalles que restan credibilidad frente a usuarios reales.

**Notas / decisiones:**

- Página `/for/{ns}/propuestas/:id/editar` (autor en `activa` o admin).
- Borrar comentario: autor o admin, con confirmación.
- `POST /api/uploads/logo` (JPEG/PNG/WebP ≤ 2 MB) → `/uploads/…`; el form también admite URL.

---

### 4. Invitaciones al barrio

| Campo | Valor |
|-------|-------|
| Estado | `hecho` |
| Prioridad | Alta |

**Qué:** Link de invitación tipo `/for/{barrio}?invite=…` con copy claro para compartir.

**Por qué:** En apps de barrio el crecimiento es social: un admin invita vecinos, no SEO.

**Notas / decisiones:**

- `namespaces.invite_code` (no se expone en GET público).
- Admins: Configuración → copiar / regenerar link.
- Al abrir el link e iniciar sesión, `POST …/membership/accept-invite` deja membresía `active` (salta la cola de aprobación).

---

### 5. Dashboard simple para space admins

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Alta |

**Qué:** Panel con métricas básicas: % de miembros que priorizaron, propuestas más consensuadas vs polarizadas, pendientes de aprobación.

**Por qué:** Da argumento concreto para usar Priora en la reunión del barrio.

**Notas / decisiones:**

-

---

## Diferenciación

### 6. Explicar el ranking Borda en lenguaje humano

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Media |

**Qué:** Textos del estilo “Tu #1 suma X puntos; esta propuesta está #3 porque N vecinos la pusieron alto”.

**Por qué:** Reduce desconfianza en el método de priorización.

**Notas / decisiones:**

-

---

### 7. Consenso vs conflicto

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Media |

**Qué:** Destacar propuestas con ranking estable y las que dividen opiniones.

**Por qué:** Útil para mediación vecinal, no solo para “ganar”.

**Notas / decisiones:**

-

---

### 8. Historia del perfil

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Media |

**Qué:** En el perfil: “mis propuestas”, “mi ranking actual”, “comentarios recientes”.

**Por qué:** Refuerza identidad y retorno a la app.

**Notas / decisiones:**

-

---

### 9. Adjuntos / evidencia

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Media |

**Qué:** Fotos del problema (bache, plaza, etc.), no solo logo de la propuesta.

**Por qué:** En civic tech, la evidencia visual convierte discusión en acción.

**Notas / decisiones:**

-

---

## Crecimiento y retención

### 10. Digest semanal del barrio

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Media |

**Qué:** Resumen semanal por email: top 3 del ranking + 1–2 propuestas nuevas.

**Por qué:** Mantiene el hábito sin necesidad de app nativa.

**Notas / decisiones:**

-

---

### 11. Moderación básica

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Media |

**Qué:** Reportar comentario + cola de revisión para space admin / admin.

**Por qué:** Inevitable cuando hay más de un puñado de usuarios activos.

**Notas / decisiones:**

-

---

### 12. Roles más claros en onboarding

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Media |

**Qué:** Tutorial / copy que explique: quién propone, quién prioriza, qué hace un admin.

**Por qué:** Baja fricción en el primer uso y reduce confusión de permisos.

**Notas / decisiones:**

-

---

## Fase 2 (más adelante)

### 13. Integraciones y expansiones

| Campo | Valor |
|-------|-------|
| Estado | `pendiente` |
| Prioridad | Baja |

**Qué (ítems sueltos):**

- [ ] Integración municipal / export CSV para llevar a la junta
- [ ] Mapa por dirección (ya se pide calle / CP)
- [ ] App nativa (solo si el engagement web lo justifica)
- [ ] Alternativas de votación (Elo, presupuestos participativos) si el Borda se queda corto en la práctica

**Notas / decisiones:**

-

---

## Registro de avance

| # | Ítem | Estado | Fecha | Notas |
|---|------|--------|-------|-------|
| 1 | Seguimiento visible | hecho | 2026-07-11 | Timeline + tracker UI + admin assign |
| 2 | Notificaciones | pendiente | | |
| 3 | Superficies incompletas | hecho | 2026-07-11 | Edit UI + delete comments + logo upload |
| 4 | Invitaciones | hecho | 2026-07-11 | invite_code + share + accept-invite |
| 5 | Dashboard admins | pendiente | | |
| 6 | Explicar Borda | pendiente | | |
| 7 | Consenso vs conflicto | pendiente | | |
| 8 | Historia del perfil | pendiente | | |
| 9 | Adjuntos / evidencia | pendiente | | |
| 10 | Digest semanal | pendiente | | |
| 11 | Moderación básica | pendiente | | |
| 12 | Onboarding de roles | pendiente | | |
| 13 | Fase 2 | pendiente | | |
