import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { api } from '../api/client';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';
import { namespacePath } from '../routes';

const ROLE_OPTIONS = [
  { value: 'regular', label: 'Usuario regular' },
  { value: 'proponent', label: 'Proponente' },
  { value: 'admin', label: 'Administrador' },
];

const SPACE_ROLE_OPTIONS = [
  { value: 'regular', label: 'Regular' },
  { value: 'proponent', label: 'Proponente' },
  { value: 'space_admin', label: 'Admin de espacio' },
];

const roleLabels = Object.fromEntries(ROLE_OPTIONS.map((r) => [r.value, r.label]));
const spaceRoleLabels = Object.fromEntries(SPACE_ROLE_OPTIONS.map((r) => [r.value, r.label]));
const statusLabels = {
  pending: 'Pendiente',
  active: 'Activo',
  rejected: 'Rechazado',
  disabled: 'Deshabilitado',
};

const USERS_PAGE_SIZE = 20;

function DashboardTab() {
  const { slug, path } = useNamespace();

  const { data, isLoading, error } = useQuery({
    queryKey: ['spaceDashboard', slug],
    queryFn: () => api.spaceDashboard(slug),
  });

  if (isLoading) return <p>Cargando métricas…</p>;
  if (error) return <p className="error">{error.message}</p>;
  if (!data) return null;

  const pct = data.prioritization_pct;
  const pctLabel =
    pct == null
      ? '—'
      : `${pct % 1 === 0 ? pct.toFixed(0) : pct.toFixed(1)}%`;

  return (
    <section className="panel dashboard-panel">
      <h2>Dashboard del espacio</h2>
      <p className="section-hint">
        Resumen para la reunión del barrio: participación, pendientes y propuestas
        con consenso o conflicto.
      </p>

      <div className="stats-row">
        <div className="stat-card">
          <div className="label">Priorizaron</div>
          <div className="value teal">{pctLabel}</div>
          <p className="stat-sub">
            {data.members_who_prioritized} de {data.active_members || '—'} miembros
            activos
          </p>
        </div>
        <div className="stat-card">
          <div className="label">Pendientes de aprobación</div>
          <div className={`value${data.pending_approvals > 0 ? ' blue' : ''}`}>
            {data.pending_approvals}
          </div>
          {data.pending_approvals > 0 && (
            <p className="stat-sub">
              Revisá la pestaña Autorizaciones
            </p>
          )}
        </div>
        <div className="stat-card">
          <div className="label">Propuestas en ranking</div>
          <div className="value">{data.active_proposals}</div>
        </div>
      </div>

      <div className="dashboard-columns">
        <div>
          <h3>Más consensuadas</h3>
          {data.most_consensual.length === 0 ? (
            <p className="muted">
              Todavía no hay suficientes priorizaciones (≥3) con acuerdo claro.
            </p>
          ) : (
            <ul className="dashboard-proposal-list">
              {data.most_consensual.map((p) => (
                <li key={p.id}>
                  <span className="badge badge-agreement badge-consensus">Consenso</span>
                  <Link to={path(`propuestas/${p.id}`)}>
                    #{p.rank_position} {p.title}
                  </Link>
                  <span className="muted">{p.rankers_count} vecinos</span>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div>
          <h3>Más polarizadas</h3>
          {data.most_polarized.length === 0 ? (
            <p className="muted">
              No hay propuestas que dividan opiniones de forma marcada.
            </p>
          ) : (
            <ul className="dashboard-proposal-list">
              {data.most_polarized.map((p) => (
                <li key={p.id}>
                  <span className="badge badge-agreement badge-polarized">Divide</span>
                  <Link to={path(`propuestas/${p.id}`)}>
                    #{p.rank_position} {p.title}
                  </Link>
                  <span className="muted">{p.rankers_count} vecinos</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </section>
  );
}

function slugify(name) {
  return name
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64);
}

function formatDate(value) {
  if (!value) return '—';
  try {
    return new Date(value).toLocaleString('es-AR', {
      dateStyle: 'short',
      timeStyle: 'short',
    });
  } catch {
    return value;
  }
}

function SpaceSettingsTab({ isGlobalAdmin }) {
  const { slug, name } = useNamespace();
  const queryClient = useQueryClient();
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [copied, setCopied] = useState(false);
  const [description, setDescription] = useState('');

  const { data: namespace, isLoading } = useQuery({
    queryKey: ['namespace', slug],
    queryFn: () => api.namespace(slug),
  });

  const descriptionValue = description || namespace?.description || '';

  const { data: invite, isLoading: loadingInvite } = useQuery({
    queryKey: ['invite', slug],
    queryFn: () => api.getInvite(slug),
  });

  const updateNs = useMutation({
    mutationFn: (data) => api.updateNamespace(slug, data),
    onSuccess: (data) => {
      queryClient.setQueryData(['namespace', slug], data);
      queryClient.invalidateQueries({ queryKey: ['namespaces'] });
      queryClient.invalidateQueries({ queryKey: ['namespaces', 'admin'] });
      queryClient.invalidateQueries({ queryKey: ['membership', slug] });
      queryClient.invalidateQueries({ queryKey: ['proposals', slug] });
      setDescription('');
      setError('');
      setSuccess('Configuración del espacio actualizada.');
    },
    onError: (e) => {
      setSuccess('');
      setError(e.message);
    },
  });

  const regenerate = useMutation({
    mutationFn: () => api.regenerateInvite(slug),
    onSuccess: (data) => {
      queryClient.setQueryData(['invite', slug], data);
      setCopied(false);
      setSuccess('Se generó un nuevo link de invitación. El anterior ya no sirve.');
      setError('');
    },
    onError: (e) => {
      setSuccess('');
      setError(e.message);
    },
  });

  const inviteUrl = invite
    ? `${window.location.origin}${invite.invite_path}`
    : '';
  const shareText = inviteUrl
    ? `Te invito a Priora — propuestas de ${name}:\n${inviteUrl}`
    : '';

  async function copyInvite() {
    if (!shareText) return;
    try {
      await navigator.clipboard.writeText(shareText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setError('No se pudo copiar al portapapeles');
    }
  }

  if (isLoading || !namespace) {
    return (
      <section className="panel admin-section">
        <p>Cargando…</p>
      </section>
    );
  }

  return (
    <section className="panel admin-section">
      <h2>Espacio: {name}</h2>
      <p className="section-hint">
        Configuración de este espacio. La descripción se muestra en el listado público de espacios.
      </p>

      <form
        className="form"
        onSubmit={(e) => {
          e.preventDefault();
          updateNs.mutate({ description: descriptionValue.trim() });
        }}
      >
        <label>
          Descripción pública
          <textarea
            maxLength={500}
            rows={3}
            placeholder="En este espacio debatimos, valoramos y priorizamos las propuestas para mejorar nuestro barrio."
            value={descriptionValue}
            onChange={(e) => setDescription(e.target.value)}
          />
        </label>
        <button
          type="submit"
          className="btn btn-secondary btn-small"
          disabled={updateNs.isPending || descriptionValue.trim() === (namespace.description || '')}
        >
          Guardar descripción
        </button>
      </form>

      {isGlobalAdmin && (
        <label className="admin-toggle" style={{ marginTop: '1.25rem' }}>
          <input
            type="checkbox"
            checked={!!namespace.is_hidden}
            disabled={updateNs.isPending}
            onChange={(e) => updateNs.mutate({ is_hidden: e.target.checked })}
          />
          <span>
            <strong>Espacio oculto</strong>
            <span className="muted">
              No aparece en el listado público; solo se accede conociendo la URL.
            </span>
          </span>
        </label>
      )}

      <label className="admin-toggle" style={{ marginTop: '1.25rem' }}>
        <input
          type="checkbox"
          checked={!!namespace.require_member_approval}
          disabled={updateNs.isPending}
          onChange={(e) => updateNs.mutate({ require_member_approval: e.target.checked })}
        />
        <span>
          <strong>Aprobación de usuarios requerida</strong>
          <span className="muted">
            Si está activo, la priorización y los comentarios solo valen para usuarios
            autorizados por un admin del espacio.
          </span>
        </span>
      </label>

      <div className="invite-share">
        <h3>Invitar vecinos</h3>
        <p className="section-hint">
          Compartí este link. Quien lo abra e inicie sesión entra al espacio como miembro
          activo (sin pasar por la cola de aprobación).
        </p>
        {loadingInvite ? (
          <p className="muted">Cargando link…</p>
        ) : (
          <>
            <div className="invite-url-row">
              <input
                type="text"
                readOnly
                className="invite-url-input"
                value={inviteUrl}
                onFocus={(e) => e.target.select()}
              />
              <button type="button" className="btn btn-primary btn-small" onClick={copyInvite}>
                {copied ? 'Copiado' : 'Copiar'}
              </button>
            </div>
            <button
              type="button"
              className="btn-link"
              disabled={regenerate.isPending}
              onClick={() => {
                if (
                  window.confirm(
                    '¿Regenerar el link? El link actual dejará de funcionar.',
                  )
                ) {
                  regenerate.mutate();
                }
              }}
            >
              Regenerar link
            </button>
          </>
        )}
      </div>

      {error && <p className="error">{error}</p>}
      {success && <p className="success">{success}</p>}

      {isGlobalAdmin && (
        <p className="section-hint" style={{ marginTop: '1.5rem' }}>
          Los administradores de plataforma también pueden crear espacios y asignar roles
          globales en las otras pestañas.
        </p>
      )}
    </section>
  );
}

function AuthorizationsTab() {
  const { slug } = useNamespace();
  const queryClient = useQueryClient();
  const [error, setError] = useState('');

  const { data: pending = [], isLoading } = useQuery({
    queryKey: ['members', slug, 'pending'],
    queryFn: () => api.members(slug, 'pending'),
  });

  const updateMember = useMutation({
    mutationFn: ({ userId, status }) => api.updateMember(slug, userId, { status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['members', slug] });
      queryClient.invalidateQueries({ queryKey: ['membership', slug] });
      setError('');
    },
    onError: (e) => setError(e.message),
  });

  return (
    <section className="panel admin-section">
      <h2>Autorizaciones pendientes</h2>
      <p className="section-hint">
        Usuarios que solicitaron participar en este espacio. Al aprobar, su priorización y
        comentarios pasan a tener efecto.
      </p>

      {error && <p className="error">{error}</p>}

      {isLoading ? (
        <p>Cargando…</p>
      ) : pending.length === 0 ? (
        <p className="muted">No hay solicitudes pendientes.</p>
      ) : (
        <div className="admin-table-wrap">
          <table className="admin-table">
            <thead>
              <tr>
                <th>Nombre</th>
                <th>Email</th>
                <th>Dirección</th>
                <th>Solicitado</th>
                <th>Acciones</th>
              </tr>
            </thead>
            <tbody>
              {pending.map((m) => (
                <tr key={m.user_id}>
                  <td>{m.name}</td>
                  <td className="muted">{m.email}</td>
                  <td className="muted">
                    {[m.street, m.city].filter(Boolean).join(', ') || '—'}
                  </td>
                  <td className="muted">{formatDate(m.requested_at)}</td>
                  <td>
                    <div className="admin-actions">
                      <button
                        type="button"
                        className="btn btn-small btn-primary"
                        disabled={updateMember.isPending}
                        onClick={() =>
                          updateMember.mutate({ userId: m.user_id, status: 'active' })
                        }
                      >
                        Aprobar
                      </button>
                      <button
                        type="button"
                        className="btn btn-small btn-secondary"
                        disabled={updateMember.isPending}
                        onClick={() =>
                          updateMember.mutate({ userId: m.user_id, status: 'rejected' })
                        }
                      >
                        Rechazar
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

function MembersTab({ isGlobalAdmin }) {
  const { slug } = useNamespace();
  const queryClient = useQueryClient();
  const [error, setError] = useState('');
  const [filter, setFilter] = useState('all');
  const [addUserId, setAddUserId] = useState('');
  const [addRole, setAddRole] = useState('space_admin');

  const { data: members = [], isLoading } = useQuery({
    queryKey: ['members', slug, filter],
    queryFn: () => api.members(slug, filter === 'all' ? undefined : filter),
  });

  const { data: allMembers = [] } = useQuery({
    queryKey: ['members', slug, 'all'],
    queryFn: () => api.members(slug),
    enabled: isGlobalAdmin,
  });

  const { data: users = [] } = useQuery({
    queryKey: ['admin-users'],
    queryFn: () => api.users(),
    enabled: isGlobalAdmin,
  });

  const updateMember = useMutation({
    mutationFn: ({ userId, data }) => api.updateMember(slug, userId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['members', slug] });
      queryClient.invalidateQueries({ queryKey: ['membership', slug] });
      setError('');
      setAddUserId('');
    },
    onError: (e) => setError(e.message),
  });

  const memberIds = new Set(allMembers.map((m) => m.user_id));
  const addableUsers = users.filter((u) => !memberIds.has(u.id) && u.role !== 'admin');

  return (
    <section className="panel admin-section">
      <h2>Miembros del espacio</h2>
      <p className="section-hint">
        Deshabilitá usuarios regulares o gestioná roles de espacio. El admin de espacio puede
        crear propuestas y gestionar autorizaciones.
      </p>

      {error && <p className="error">{error}</p>}

      {isGlobalAdmin && (
        <form
          className="form form-inline-admin"
          onSubmit={(e) => {
            e.preventDefault();
            if (!addUserId) return;
            updateMember.mutate({
              userId: addUserId,
              data: { status: 'active', role: addRole },
            });
          }}
        >
          <label>
            Agregar miembro
            <select
              required
              value={addUserId}
              onChange={(e) => setAddUserId(e.target.value)}
            >
              <option value="">Elegir usuario…</option>
              {addableUsers.map((u) => (
                <option key={u.id} value={u.id}>
                  {u.name} ({u.email})
                </option>
              ))}
            </select>
          </label>
          <label>
            Rol
            <select value={addRole} onChange={(e) => setAddRole(e.target.value)}>
              {SPACE_ROLE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </label>
          <button type="submit" className="btn btn-primary" disabled={updateMember.isPending}>
            Agregar
          </button>
        </form>
      )}

      <div className="admin-toolbar">
        <label className="admin-search">
          <span className="sr-only">Filtrar por estado</span>
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            aria-label="Filtrar por estado"
          >
            <option value="all">Todos</option>
            <option value="active">Activos</option>
            <option value="pending">Pendientes</option>
            <option value="rejected">Rechazados</option>
            <option value="disabled">Deshabilitados</option>
          </select>
        </label>
        {!isLoading && (
          <span className="muted admin-count">
            {members.length} miembro{members.length === 1 ? '' : 's'}
          </span>
        )}
      </div>

      {isLoading ? (
        <p>Cargando…</p>
      ) : members.length === 0 ? (
        <p className="muted">No hay miembros con ese filtro.</p>
      ) : (
        <div className="admin-table-wrap">
          <table className="admin-table">
            <thead>
              <tr>
                <th>Nombre</th>
                <th>Email</th>
                <th>Estado</th>
                <th>Rol en espacio</th>
                <th>Acciones</th>
              </tr>
            </thead>
            <tbody>
              {members.map((m) => (
                <tr key={m.user_id}>
                  <td>{m.name}</td>
                  <td className="muted">{m.email}</td>
                  <td>
                    <span
                      className={
                        m.status === 'active'
                          ? 'badge-ok'
                          : m.status === 'pending'
                            ? 'badge-warn'
                            : 'badge-warn'
                      }
                    >
                      {statusLabels[m.status] || m.status}
                    </span>
                  </td>
                  <td>
                    {isGlobalAdmin ? (
                      <select
                        className="role-select"
                        value={m.role}
                        disabled={updateMember.isPending}
                        onChange={(e) =>
                          updateMember.mutate({
                            userId: m.user_id,
                            data: { role: e.target.value },
                          })
                        }
                      >
                        {SPACE_ROLE_OPTIONS.map((opt) => (
                          <option key={opt.value} value={opt.value}>
                            {opt.label}
                          </option>
                        ))}
                      </select>
                    ) : (
                      spaceRoleLabels[m.role] || m.role
                    )}
                  </td>
                  <td>
                    <div className="admin-actions">
                      {m.status === 'active' && m.role !== 'space_admin' && (
                        <button
                          type="button"
                          className="btn btn-small btn-secondary"
                          disabled={updateMember.isPending}
                          onClick={() =>
                            updateMember.mutate({
                              userId: m.user_id,
                              data: { status: 'disabled' },
                            })
                          }
                        >
                          Deshabilitar
                        </button>
                      )}
                      {m.status === 'disabled' && (
                        <button
                          type="button"
                          className="btn btn-small btn-primary"
                          disabled={updateMember.isPending}
                          onClick={() =>
                            updateMember.mutate({
                              userId: m.user_id,
                              data: { status: 'active' },
                            })
                          }
                        >
                          Rehabilitar
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

function emptyNsForm() {
  return {
    name: '',
    slug: '',
    description: '',
    isHidden: false,
    requireMemberApproval: false,
    slugTouched: false,
  };
}

function nsToForm(ns) {
  return {
    name: ns.name,
    slug: ns.slug,
    description: ns.description || '',
    isHidden: !!ns.is_hidden,
    requireMemberApproval: !!ns.require_member_approval,
    slugTouched: true,
  };
}

function NamespaceForm({ form, setForm, isEdit, onSubmit, onCancel, isPending, error, success }) {
  return (
    <form
      className="form namespace-form"
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit();
      }}
    >
      <label>
        Nombre
        <input
          required
          maxLength={100}
          placeholder="Priora"
          value={form.name}
          onChange={(e) => {
            const name = e.target.value;
            setForm((prev) => ({
              ...prev,
              name,
              slug: prev.slugTouched || isEdit ? prev.slug : slugify(name),
              slugTouched: prev.slugTouched,
            }));
          }}
        />
      </label>
      <label>
        Slug (URL)
        <input
          required
          readOnly={isEdit}
          pattern="[a-z0-9][a-z0-9-]{0,62}[a-z0-9]"
          title="Minúsculas, números y guiones (mín. 2 caracteres)"
          placeholder="priora"
          value={form.slug}
          onChange={(e) =>
            setForm((prev) => ({
              ...prev,
              slug: e.target.value.toLowerCase(),
              slugTouched: true,
            }))
          }
        />
        {isEdit && (
          <span className="muted" style={{ fontSize: '0.85rem' }}>
            El slug no se puede cambiar; forma parte de la URL del espacio.
          </span>
        )}
      </label>
      <label>
        Descripción
        <textarea
          maxLength={500}
          rows={3}
          placeholder="Espacio para experimentar y mejorar Priora: probamos ideas, priorizamos mejoras y damos feedback sobre la plataforma."
          value={form.description}
          onChange={(e) => setForm((prev) => ({ ...prev, description: e.target.value }))}
        />
      </label>
      <label className="admin-toggle">
        <input
          type="checkbox"
          checked={form.isHidden}
          onChange={(e) => setForm((prev) => ({ ...prev, isHidden: e.target.checked }))}
        />
        <span>
          <strong>Espacio oculto</strong>
          <span className="muted">
            No aparece en el listado público; solo se accede conociendo la URL.
          </span>
        </span>
      </label>
      {isEdit && (
        <label className="admin-toggle">
          <input
            type="checkbox"
            checked={form.requireMemberApproval}
            onChange={(e) =>
              setForm((prev) => ({ ...prev, requireMemberApproval: e.target.checked }))
            }
          />
          <span>
            <strong>Aprobación de usuarios requerida</strong>
            <span className="muted">
              Si está activo, la priorización y los comentarios solo valen para usuarios
              autorizados.
            </span>
          </span>
        </label>
      )}
      <div className="namespace-form-actions">
        <button type="submit" className="btn btn-primary" disabled={isPending}>
          {isEdit ? 'Guardar cambios' : 'Crear espacio'}
        </button>
        <button type="button" className="btn btn-secondary" onClick={onCancel} disabled={isPending}>
          Cancelar
        </button>
      </div>
      {error && <p className="error">{error}</p>}
      {success && <p className="success">{success}</p>}
    </form>
  );
}

function NamespacesTab() {
  const queryClient = useQueryClient();
  const [view, setView] = useState('list');
  const [editingSlug, setEditingSlug] = useState(null);
  const [search, setSearch] = useState('');
  const [nsForm, setNsForm] = useState(emptyNsForm);
  const [nsError, setNsError] = useState('');
  const [nsSuccess, setNsSuccess] = useState('');

  const { data: namespaces = [], isLoading: loadingNs } = useQuery({
    queryKey: ['namespaces', 'admin'],
    queryFn: () => api.namespaces({ includeHidden: true }),
  });

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return namespaces;
    return namespaces.filter(
      (ns) =>
        ns.name?.toLowerCase().includes(q) ||
        ns.slug?.toLowerCase().includes(q) ||
        ns.description?.toLowerCase().includes(q)
    );
  }, [namespaces, search]);

  const resetFormState = () => {
    setNsForm(emptyNsForm());
    setEditingSlug(null);
    setNsError('');
    setNsSuccess('');
  };

  const openCreate = () => {
    resetFormState();
    setView('form');
  };

  const openEdit = (ns) => {
    setNsForm(nsToForm(ns));
    setEditingSlug(ns.slug);
    setNsError('');
    setNsSuccess('');
    setView('form');
  };

  const backToList = () => {
    setView('list');
    resetFormState();
  };

  const createNs = useMutation({
    mutationFn: () =>
      api.createNamespace({
        name: nsForm.name.trim(),
        slug: nsForm.slug.trim(),
        description: nsForm.description.trim() || undefined,
        is_hidden: nsForm.isHidden,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['namespaces'] });
      queryClient.invalidateQueries({ queryKey: ['namespaces', 'admin'] });
      setNsSuccess('Espacio creado correctamente');
      setNsError('');
      setTimeout(() => backToList(), 600);
    },
    onError: (e) => {
      setNsSuccess('');
      setNsError(e.message);
    },
  });

  const updateNs = useMutation({
    mutationFn: () =>
      api.updateNamespace(editingSlug, {
        name: nsForm.name.trim(),
        description: nsForm.description.trim(),
        is_hidden: nsForm.isHidden,
        require_member_approval: nsForm.requireMemberApproval,
      }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['namespaces'] });
      queryClient.invalidateQueries({ queryKey: ['namespaces', 'admin'] });
      queryClient.setQueryData(['namespace', editingSlug], data);
      setNsSuccess('Espacio actualizado correctamente');
      setNsError('');
      setTimeout(() => backToList(), 600);
    },
    onError: (e) => {
      setNsSuccess('');
      setNsError(e.message);
    },
  });

  const isEdit = editingSlug != null;
  const isPending = createNs.isPending || updateNs.isPending;

  if (view === 'form') {
    return (
      <section className="panel admin-section">
        <div className="namespace-panel-header">
          <div>
            <h2>{isEdit ? 'Editar espacio' : 'Agregar espacio'}</h2>
            <p className="section-hint">
              {isEdit
                ? 'Modificá los datos del espacio. El slug no se puede cambiar.'
                : 'Cada espacio tiene sus propias propuestas y ranking. El slug forma parte de la URL.'}
            </p>
          </div>
        </div>
        <NamespaceForm
          form={nsForm}
          setForm={setNsForm}
          isEdit={isEdit}
          isPending={isPending}
          error={nsError}
          success={nsSuccess}
          onCancel={backToList}
          onSubmit={() => (isEdit ? updateNs.mutate() : createNs.mutate())}
        />
      </section>
    );
  }

  return (
    <section className="panel admin-section">
      <div className="namespace-panel-header">
        <div>
          <h2>Espacios</h2>
          <p className="section-hint">
            Cada espacio tiene sus propias propuestas y ranking. El slug forma parte de la URL.
          </p>
        </div>
        <button type="button" className="btn btn-primary btn-small" onClick={openCreate}>
          Agregar espacio
        </button>
      </div>

      <div className="admin-toolbar">
        <label className="admin-search">
          <span className="sr-only">Buscar espacios</span>
          <input
            type="search"
            placeholder="Buscar por nombre, slug o descripción…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </label>
        {!loadingNs && (
          <span className="muted admin-count">
            {filtered.length === namespaces.length
              ? `${namespaces.length} espacio${namespaces.length === 1 ? '' : 's'}`
              : `${filtered.length} de ${namespaces.length}`}
          </span>
        )}
      </div>

      {loadingNs ? (
        <p>Cargando espacios…</p>
      ) : filtered.length === 0 ? (
        <p className="muted">
          {namespaces.length === 0
            ? 'No hay espacios creados.'
            : 'Ningún espacio coincide con la búsqueda.'}
        </p>
      ) : (
        <ul className="admin-list">
          {filtered.map((ns) => (
            <li key={ns.id} className="admin-list-item">
              <div>
                <strong>{ns.name}</strong>
                <span className="muted mono">{ns.slug}</span>
                {ns.description && <p className="admin-list-description">{ns.description}</p>}
                <div className="admin-list-badges">
                  {ns.is_hidden && <span className="badge-warn">Oculto</span>}
                  {ns.require_member_approval && (
                    <span className="badge-warn">Aprobación requerida</span>
                  )}
                </div>
              </div>
              <div className="admin-actions">
                <button
                  type="button"
                  className="btn btn-small btn-secondary"
                  onClick={() => openEdit(ns)}
                >
                  Editar
                </button>
                <Link to={namespacePath(ns.slug)} className="btn btn-small btn-secondary">
                  Abrir
                </Link>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function UsersTab({ currentUser }) {
  const queryClient = useQueryClient();
  const [roleError, setRoleError] = useState('');
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(1);

  const { data: users = [], isLoading: loadingUsers } = useQuery({
    queryKey: ['admin-users'],
    queryFn: () => api.users(),
  });

  const updateRole = useMutation({
    mutationFn: ({ id, role }) => api.updateUserRole(id, role),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin-users'] });
      setRoleError('');
    },
    onError: (e) => setRoleError(e.message),
  });

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return users;
    return users.filter(
      (u) =>
        u.name?.toLowerCase().includes(q) ||
        u.email?.toLowerCase().includes(q) ||
        (roleLabels[u.role] || u.role).toLowerCase().includes(q)
    );
  }, [users, search]);

  const totalPages = Math.max(1, Math.ceil(filtered.length / USERS_PAGE_SIZE));
  const currentPage = Math.min(page, totalPages);
  const pageUsers = filtered.slice(
    (currentPage - 1) * USERS_PAGE_SIZE,
    currentPage * USERS_PAGE_SIZE
  );

  const from = filtered.length === 0 ? 0 : (currentPage - 1) * USERS_PAGE_SIZE + 1;
  const to = Math.min(currentPage * USERS_PAGE_SIZE, filtered.length);

  return (
    <section className="panel admin-section">
      <h2>Usuarios y roles</h2>
      <p className="section-hint">
        Roles globales de plataforma. Para roles por espacio usá la pestaña Miembros.
      </p>

      {roleError && <p className="error">{roleError}</p>}

      <div className="admin-toolbar">
        <label className="admin-search">
          <span className="sr-only">Buscar usuarios</span>
          <input
            type="search"
            placeholder="Buscar por nombre, email o rol…"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(1);
            }}
          />
        </label>
        {!loadingUsers && (
          <span className="muted admin-count">
            {filtered.length === users.length
              ? `${users.length} usuario${users.length === 1 ? '' : 's'}`
              : `${filtered.length} de ${users.length}`}
          </span>
        )}
      </div>

      {loadingUsers ? (
        <p>Cargando usuarios…</p>
      ) : filtered.length === 0 ? (
        <p className="muted">
          {users.length === 0 ? 'No hay usuarios.' : 'Ningún usuario coincide con la búsqueda.'}
        </p>
      ) : (
        <>
          <div className="admin-table-wrap">
            <table className="admin-table">
              <thead>
                <tr>
                  <th>Nombre</th>
                  <th>Email</th>
                  <th>Perfil</th>
                  <th>Rol</th>
                </tr>
              </thead>
              <tbody>
                {pageUsers.map((u) => {
                  const isSelf = u.id === currentUser.id;
                  return (
                    <tr key={u.id}>
                      <td>{u.name}</td>
                      <td className="muted">{u.email}</td>
                      <td>
                        <span className={u.profile_complete ? 'badge-ok' : 'badge-warn'}>
                          {u.profile_complete ? 'Completo' : 'Incompleto'}
                        </span>
                      </td>
                      <td>
                        {isSelf ? (
                          <span className="role-self">{roleLabels[u.role] || u.role}</span>
                        ) : (
                          <select
                            className="role-select"
                            value={u.role}
                            disabled={updateRole.isPending}
                            onChange={(e) =>
                              updateRole.mutate({ id: u.id, role: e.target.value })
                            }
                          >
                            {ROLE_OPTIONS.map((opt) => (
                              <option key={opt.value} value={opt.value}>
                                {opt.label}
                              </option>
                            ))}
                          </select>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          <div className="admin-pagination">
            <span className="muted">
              {from}–{to} de {filtered.length}
            </span>
            <div className="admin-pagination-actions">
              <button
                type="button"
                className="btn btn-small btn-secondary"
                disabled={currentPage <= 1}
                onClick={() => setPage((p) => Math.max(1, p - 1))}
              >
                Anterior
              </button>
              <span className="admin-page-indicator">
                Página {currentPage} de {totalPages}
              </span>
              <button
                type="button"
                className="btn btn-small btn-secondary"
                disabled={currentPage >= totalPages}
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              >
                Siguiente
              </button>
            </div>
          </div>
        </>
      )}
    </section>
  );
}

export default function AdminSettings() {
  const { user } = useAuth();
  const { path, slug } = useNamespace();
  const isGlobalAdmin = user?.role === 'admin';

  const { data: membership, isLoading } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user,
  });

  const canManage = isGlobalAdmin || membership?.can_manage_space;
  const [tab, setTab] = useState('dashboard');

  if (isLoading) {
    return <p>Cargando…</p>;
  }

  if (!user || !canManage) {
    return (
      <div className="panel page-narrow">
        <p>No tienes permiso para ver esta sección.</p>
        <Link to={path()}>Volver</Link>
      </div>
    );
  }

  const tabs = [
    { id: 'dashboard', label: 'Dashboard' },
    { id: 'space', label: 'Este espacio' },
    { id: 'authorizations', label: 'Autorizaciones' },
    { id: 'members', label: 'Miembros' },
    ...(isGlobalAdmin
      ? [
          { id: 'users', label: 'Roles globales' },
          { id: 'namespaces', label: 'Espacios' },
        ]
      : []),
  ];

  return (
    <div>
      <div className="content-header">
        <div>
          <h1>Configuración</h1>
          <p>Dashboard, administración del espacio, autorizaciones y roles</p>
        </div>
      </div>

      <div className="admin-tabs" role="tablist" aria-label="Secciones de configuración">
        {tabs.map((t) => (
          <button
            key={t.id}
            type="button"
            role="tab"
            id={`tab-${t.id}`}
            aria-selected={tab === t.id}
            aria-controls={`panel-${t.id}`}
            className={`admin-tab${tab === t.id ? ' active' : ''}`}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div role="tabpanel" id={`panel-${tab}`} aria-labelledby={`tab-${tab}`}>
        {tab === 'dashboard' && <DashboardTab />}
        {tab === 'space' && <SpaceSettingsTab isGlobalAdmin={isGlobalAdmin} />}
        {tab === 'authorizations' && <AuthorizationsTab />}
        {tab === 'members' && <MembersTab isGlobalAdmin={isGlobalAdmin} />}
        {tab === 'users' && isGlobalAdmin && <UsersTab currentUser={user} />}
        {tab === 'namespaces' && isGlobalAdmin && <NamespacesTab />}
      </div>
    </div>
  );
}
