import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { api } from '../api/client';
import { openTutorial } from '../components/TutorialOverlay';
import StatusBadge from '../components/StatusBadge';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

const roleLabels = {
  regular: 'Usuario regular',
  proponent: 'Proponente',
  admin: 'Administrador',
};

const roleHelp = {
  regular:
    'Podés priorizar propuestas y comentar. Para crear propuestas necesitás rol de proponente o admin.',
  proponent: 'Podés crear propuestas, priorizar y comentar en este espacio.',
  admin:
    'Administrás la plataforma: roles globales, espacios y cualquier configuración.',
};

function formatDate(value) {
  try {
    return new Date(value).toLocaleString('es-AR', {
      dateStyle: 'short',
      timeStyle: 'short',
    });
  } catch {
    return value;
  }
}

export default function Profile() {
  const { user, refresh } = useAuth();
  const { slug, path, name } = useNamespace();
  const [form, setForm] = useState({
    street: user?.street || '',
    floor_apt: user?.floor_apt || '',
    city: user?.city || '',
    postal_code: user?.postal_code || '',
  });
  const [message, setMessage] = useState('');

  const { data: activity, isLoading: loadingActivity } = useQuery({
    queryKey: ['myActivity', slug],
    queryFn: () => api.myActivity(slug),
    enabled: !!user && !!slug,
  });

  const { data: membership } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user && !!slug,
  });

  const mutation = useMutation({
    mutationFn: () => api.updateProfile(form),
    onSuccess: async () => {
      await refresh();
      setMessage('Perfil actualizado');
    },
  });

  if (!user) return <p>Debes iniciar sesión.</p>;

  const spaceRole = membership?.membership?.role;
  const spaceRoleLabel =
    spaceRole === 'space_admin'
      ? 'Admin de espacio'
      : spaceRole === 'proponent'
        ? 'Proponente del espacio'
        : spaceRole === 'regular'
          ? 'Miembro'
          : null;

  return (
    <div>
      <div className="content-header">
        <div>
          <h1>Mi perfil</h1>
          <p>Datos de tu cuenta, dirección e historial en {name}</p>
        </div>
      </div>

      <div className="panel">
        <div className="profile-info">
          <p><strong>Nombre:</strong> {user.name}</p>
          <p><strong>Email:</strong> {user.email}</p>
          <p><strong>Rol global:</strong> {roleLabels[user.role] || user.role}</p>
          {spaceRoleLabel && (
            <p><strong>En este espacio:</strong> {spaceRoleLabel}</p>
          )}
        </div>
        <p className="section-hint">{roleHelp[user.role] || roleHelp.regular}</p>

        <h2>Dirección</h2>
        <form
          className="form"
          onSubmit={(e) => {
            e.preventDefault();
            mutation.mutate();
          }}
        >
          <label>
            Calle y número
            <input
              required
              minLength={5}
              value={form.street}
              onChange={(e) => setForm({ ...form, street: e.target.value })}
            />
          </label>
          <label>
            Piso / Depto
            <input
              value={form.floor_apt}
              onChange={(e) => setForm({ ...form, floor_apt: e.target.value })}
            />
          </label>
          <label>
            Ciudad / Barrio
            <input
              required
              value={form.city}
              onChange={(e) => setForm({ ...form, city: e.target.value })}
            />
          </label>
          <label>
            Código postal
            <input
              value={form.postal_code}
              onChange={(e) => setForm({ ...form, postal_code: e.target.value })}
            />
          </label>
          <button type="submit" className="btn btn-primary" disabled={mutation.isPending}>
            Guardar
          </button>
        </form>
        {message && <p className="success">{message}</p>}

        <div className="hint-box">
          <p>¿Querés repasar quién propone, quién prioriza y qué hace un admin?</p>
          <button type="button" className="btn btn-secondary" onClick={openTutorial}>
            Ver tutorial
          </button>
        </div>
      </div>

      <div className="profile-activity">
        <section className="panel">
          <h2>Mis propuestas</h2>
          {loadingActivity && <p className="muted">Cargando…</p>}
          {!loadingActivity && (!activity?.proposals || activity.proposals.length === 0) && (
            <p className="muted">Todavía no creaste propuestas en este espacio.</p>
          )}
          {activity?.proposals?.length > 0 && (
            <ul className="activity-list">
              {activity.proposals.map((p) => (
                <li key={p.id}>
                  <Link to={path(`propuestas/${p.id}`)}>{p.title}</Link>
                  <div className="activity-meta">
                    <StatusBadge status={p.status} />
                    {p.rank_position != null && <span>#{p.rank_position}</span>}
                    {p.score > 0 && <span>{p.score} pts</span>}
                    <time dateTime={p.created_at}>{formatDate(p.created_at)}</time>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="panel">
          <h2>Mi ranking actual</h2>
          {loadingActivity && <p className="muted">Cargando…</p>}
          {!loadingActivity && (!activity?.ranking || activity.ranking.length === 0) && (
            <p className="muted">
              Aún no priorizaste.{' '}
              <Link to={path('priorizar')}>Ir a priorizar</Link>
            </p>
          )}
          {activity?.ranking?.length > 0 && (
            <>
              <p className="section-hint">
                Tu #1 aporta {activity.ranking[0]?.points ?? '—'} puntos al ranking global.
              </p>
              <ol className="activity-ranking">
                {activity.ranking.map((item) => (
                  <li key={item.proposal_id}>
                    <span className="activity-pos">#{item.position + 1}</span>
                    <Link to={path(`propuestas/${item.proposal_id}`)}>{item.title}</Link>
                    <span className="activity-points">{item.points} pts</span>
                  </li>
                ))}
              </ol>
              <Link to={path('priorizar')} className="btn btn-secondary btn-small">
                Editar priorización
              </Link>
            </>
          )}
        </section>

        <section className="panel">
          <h2>Comentarios recientes</h2>
          {loadingActivity && <p className="muted">Cargando…</p>}
          {!loadingActivity && (!activity?.comments || activity.comments.length === 0) && (
            <p className="muted">No hay comentarios tuyos en este espacio.</p>
          )}
          {activity?.comments?.length > 0 && (
            <ul className="activity-list">
              {activity.comments.map((c) => (
                <li key={c.id}>
                  <Link to={path(`propuestas/${c.proposal_id}`)}>{c.proposal_title}</Link>
                  <p className="activity-comment-body">{c.content}</p>
                  <div className="activity-meta">
                    <time dateTime={c.created_at}>{formatDate(c.created_at)}</time>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}
