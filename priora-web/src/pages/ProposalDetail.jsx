import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';
import { api, assetUrl } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

const STATUS_LABELS = {
  activa: 'Activa',
  en_analisis: 'En análisis',
  rechazada: 'Rechazada',
};

const STATUS_TRANSITIONS = {
  activa: ['activa', 'en_analisis', 'rechazada'],
  en_analisis: ['en_analisis', 'activa', 'rechazada'],
  rechazada: ['rechazada', 'activa'],
};

function formatDate(value) {
  return new Date(value).toLocaleString('es-AR', {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
}

function timelineLabel(event) {
  if (event.event_type === 'created') {
    return 'Propuesta creada';
  }
  if (event.event_type === 'status_changed') {
    const from = STATUS_LABELS[event.from_value] || event.from_value;
    const to = STATUS_LABELS[event.to_value] || event.to_value;
    return `Estado: ${from} → ${to}`;
  }
  if (event.event_type === 'tracker_changed') {
    if (!event.to_user) {
      return 'Responsable de seguimiento quitado';
    }
    if (!event.from_user) {
      return `Responsable asignado: ${event.to_user.name}`;
    }
    return `Responsable: ${event.from_user.name} → ${event.to_user.name}`;
  }
  return event.event_type;
}

export default function ProposalDetail() {
  const { id } = useParams();
  const { slug, path } = useNamespace();
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [comment, setComment] = useState('');
  const [showAll, setShowAll] = useState(false);
  const [lightboxUrl, setLightboxUrl] = useState(null);

  const { data: proposal, isLoading, error } = useQuery({
    queryKey: ['proposal', slug, id],
    queryFn: () => api.proposal(slug, id),
  });

  const { data: activeProposals = [] } = useQuery({
    queryKey: ['proposals', slug, 'active'],
    queryFn: () => api.proposals(slug, 'active'),
  });

  const { data: commentsPage } = useQuery({
    queryKey: ['comments', slug, id, showAll],
    queryFn: () => api.comments(slug, id, showAll ? 100 : 10, 0),
    enabled: !!id,
  });

  const { data: membership } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user,
  });

  const isAdmin = user?.role === 'admin';

  const { data: users = [] } = useQuery({
    queryKey: ['admin-users'],
    queryFn: () => api.users(),
    enabled: isAdmin,
  });

  const maxScore = useMemo(
    () => Math.max(...activeProposals.map((p) => p.score || 0), 1),
    [activeProposals],
  );

  const scorePct = proposal ? Math.round(((proposal.score || 0) / maxScore) * 100) : 0;
  const isTopScore = proposal?.rank_position === 1;

  const commentMutation = useMutation({
    mutationFn: (content) => api.addComment(slug, id, content),
    onSuccess: () => {
      setComment('');
      queryClient.invalidateQueries({ queryKey: ['comments', slug, id] });
    },
  });

  const invalidateProposal = () => {
    queryClient.invalidateQueries({ queryKey: ['proposal', slug, id] });
    queryClient.invalidateQueries({ queryKey: ['proposals', slug] });
  };

  const statusMutation = useMutation({
    mutationFn: (status) => api.updateStatus(slug, id, status),
    onSuccess: invalidateProposal,
  });

  const trackerMutation = useMutation({
    mutationFn: (trackerId) => api.updateTracker(slug, id, trackerId),
    onSuccess: invalidateProposal,
  });

  if (isLoading) return <p>Cargando…</p>;
  if (error) return <p className="error">{error.message}</p>;
  if (!proposal) return null;

  const canComment =
    user?.profile_complete &&
    proposal.status !== 'rechazada' &&
    membership?.can_comment;
  const commentCount = commentsPage?.total ?? commentsPage?.comments?.length ?? 0;
  const needsApproval =
    membership?.require_member_approval && user?.profile_complete && !membership?.can_comment;
  const allowedStatuses = STATUS_TRANSITIONS[proposal.status] || [proposal.status];
  const timeline = proposal.timeline || [];
  const imageUrls = Array.isArray(proposal.image_urls) ? proposal.image_urls : [];

  return (
    <div>
      <div className="breadcrumb">
        <Link to={path()}>Propuestas</Link> / {proposal.title}
      </div>

      <div className="detail-layout">
        <div>
          <article className="detail-main">
            <div className="detail-badges">
              <StatusBadge status={proposal.status} />
              {proposal.category && (
                <span className="badge badge-category">{proposal.category.name}</span>
              )}
            </div>
            <h1>{proposal.title}</h1>

            {imageUrls.length > 0 && (
              <ul className="proposal-gallery">
                {imageUrls.map((url, index) => (
                  <li key={`${url}-${index}`}>
                    <button
                      type="button"
                      className="proposal-gallery-thumb"
                      onClick={() => setLightboxUrl(url)}
                      aria-label={`Ver imagen ${index + 1} en grande`}
                    >
                      <img src={assetUrl(url)} alt={`Imagen ${index + 1} de la propuesta`} />
                    </button>
                  </li>
                ))}
              </ul>
            )}

            <div className="body">{proposal.description}</div>

            {lightboxUrl && (
              <div
                className="proposal-lightbox"
                role="dialog"
                aria-modal="true"
                aria-label="Vista ampliada de la imagen"
                onClick={() => setLightboxUrl(null)}
                onKeyDown={(e) => {
                  if (e.key === 'Escape') setLightboxUrl(null);
                }}
              >
                <button
                  type="button"
                  className="proposal-lightbox-close"
                  onClick={() => setLightboxUrl(null)}
                >
                  Cerrar
                </button>
                <img
                  src={assetUrl(lightboxUrl)}
                  alt="Imagen ampliada"
                  onClick={(e) => e.stopPropagation()}
                />
              </div>
            )}

            {isAdmin && (
              <div className="admin-panel">
                <h3>Administración</h3>
                <div className="admin-field">
                  <label>Estado</label>
                  <div className="admin-actions">
                    {allowedStatuses.map((s) => (
                      <button
                        key={s}
                        type="button"
                        className={`btn btn-small btn-secondary ${proposal.status === s ? 'active' : ''}`}
                        disabled={statusMutation.isPending || proposal.status === s}
                        onClick={() => statusMutation.mutate(s)}
                      >
                        {STATUS_LABELS[s] || s}
                      </button>
                    ))}
                  </div>
                </div>
                <div className="admin-field">
                  <label htmlFor="tracker-select">Responsable de seguimiento</label>
                  <select
                    id="tracker-select"
                    className="tracker-select"
                    value={proposal.tracker?.id || ''}
                    disabled={trackerMutation.isPending}
                    onChange={(e) => {
                      const value = e.target.value;
                      trackerMutation.mutate(value || null);
                    }}
                  >
                    <option value="">Sin asignar</option>
                    {users.map((u) => (
                      <option key={u.id} value={u.id}>
                        {u.name}
                        {u.email ? ` (${u.email})` : ''}
                      </option>
                    ))}
                  </select>
                </div>
                {(statusMutation.isError || trackerMutation.isError) && (
                  <p className="error">
                    {statusMutation.error?.message || trackerMutation.error?.message}
                  </p>
                )}
              </div>
            )}
          </article>

          <section className="comments-panel">
            <h2>Comentarios ({commentCount})</h2>

            {canComment && (
              <form
                className="comment-form"
                onSubmit={(e) => {
                  e.preventDefault();
                  if (comment.trim()) commentMutation.mutate(comment.trim());
                }}
              >
                <textarea
                  rows={3}
                  placeholder="Escribí tu comentario…"
                  value={comment}
                  onChange={(e) => setComment(e.target.value)}
                />
                <button type="submit" className="btn btn-primary" disabled={commentMutation.isPending}>
                  Publicar
                </button>
              </form>
            )}

            {needsApproval && proposal.status !== 'rechazada' && (
              <p className="subtitle">
                Necesitás autorización de un admin del espacio para comentar.
              </p>
            )}

            {!user && (
              <p className="subtitle">
                <Link to="/login" state={{ returnTo: path(`propuestas/${id}`) }}>Inicia sesión</Link> para comentar.
              </p>
            )}

            <div className="comments-list">
              {commentsPage?.comments?.map((c) => (
                <article key={c.id} className="comment">
                  <div className="comment-author">{c.author.name}</div>
                  <div className="comment-content">{c.content}</div>
                  <time className="comment-date">
                    {new Date(c.created_at).toLocaleString('es-AR')}
                  </time>
                </article>
              ))}
            </div>

            {!showAll && commentsPage && commentsPage.total > 10 && (
              <button type="button" className="btn-link" onClick={() => setShowAll(true)}>
                Ver todos los comentarios ({commentsPage.total})
              </button>
            )}
          </section>
        </div>

        <aside className="detail-sidebar">
          <div className="panel tracking-panel">
            <h3>Seguimiento</h3>
            <div className="tracking-status">
              <StatusBadge status={proposal.status} />
              <p className="tracking-status-hint">
                {proposal.status === 'activa' && 'Abierta a priorización y comentarios.'}
                {proposal.status === 'en_analisis' && 'En evaluación por la administración.'}
                {proposal.status === 'rechazada' && 'Descartada; visible solo con filtro.'}
              </p>
            </div>

            <div className="tracker-card">
              <span className="tracker-label">Responsable</span>
              {proposal.tracker ? (
                <div className="tracker-person">
                  {proposal.tracker.picture_url ? (
                    <img
                      src={proposal.tracker.picture_url}
                      alt=""
                      className="tracker-avatar"
                    />
                  ) : (
                    <span className="tracker-avatar tracker-avatar-fallback" aria-hidden="true">
                      {proposal.tracker.name.slice(0, 1).toUpperCase()}
                    </span>
                  )}
                  <strong>{proposal.tracker.name}</strong>
                </div>
              ) : (
                <p className="tracker-empty">Todavía no hay responsable asignado.</p>
              )}
            </div>

            <div className="timeline">
              <h4>Historial</h4>
              {timeline.length === 0 ? (
                <p className="tracker-empty">Sin eventos aún.</p>
              ) : (
                <ol className="timeline-list">
                  {[...timeline].reverse().map((event) => (
                    <li key={event.id} className={`timeline-item timeline-${event.event_type}`}>
                      <div className="timeline-dot" aria-hidden="true" />
                      <div className="timeline-body">
                        <div className="timeline-title">{timelineLabel(event)}</div>
                        <div className="timeline-meta">
                          <time dateTime={event.created_at}>{formatDate(event.created_at)}</time>
                          {event.actor && <span> · {event.actor.name}</span>}
                        </div>
                      </div>
                    </li>
                  ))}
                </ol>
              )}
            </div>
          </div>

          <div className="panel">
            <h3>Información</h3>
            <div className="row">
              <span>Autor/a</span>
              <strong>{proposal.author.name}</strong>
            </div>
            {proposal.category && (
              <div className="row">
                <span>Categoría</span>
                <strong>{proposal.category.name}</strong>
              </div>
            )}
            <div className="row">
              <span>Creada</span>
              <strong>{formatDate(proposal.created_at)}</strong>
            </div>
            {proposal.rank_position > 0 && (
              <div className="row">
                <span>Ranking</span>
                <strong>#{proposal.rank_position}</strong>
              </div>
            )}
            {proposal.score > 0 && (
              <div className="row">
                <span>Puntos</span>
                <strong>{proposal.score}</strong>
              </div>
            )}
            {proposal.agreement === 'consensus' && (
              <div className="row">
                <span>Acuerdo</span>
                <strong className="text-consensus">Consenso</strong>
              </div>
            )}
            {proposal.agreement === 'polarized' && (
              <div className="row">
                <span>Acuerdo</span>
                <strong className="text-polarized">Divide opiniones</strong>
              </div>
            )}
          </div>

          {proposal.score > 0 && (
            <div className="panel">
              <h3>Progreso de apoyo</h3>
              <div className="score-bar" style={{ textAlign: 'left' }}>
                <div className="score" style={{ fontSize: '2rem' }}>{proposal.score}</div>
                <div className="bar" style={{ height: '8px', marginTop: '0.5rem' }}>
                  <div className="bar-fill" style={{ width: `${scorePct}%` }} />
                </div>
                {isTopScore && (
                  <p className="score-note">Mayor puntuación del espacio</p>
                )}
              </div>
            </div>
          )}

          {proposal.ranking_insight && (
            <div className="panel ranking-insight-panel">
              <h3>Cómo se calcula</h3>
              <p className="ranking-insight-summary">{proposal.ranking_insight.summary}</p>
              {proposal.agreement === 'consensus' && (
                <p className="agreement-note consensus">Hay consenso entre quienes priorizaron.</p>
              )}
              {proposal.agreement === 'polarized' && (
                <p className="agreement-note polarized">Esta propuesta divide opiniones.</p>
              )}
              <details className="borda-explainer">
                <summary>Método Borda (en simple)</summary>
                <p>
                  Si hay {proposal.ranking_insight.points_for_first} propuestas en tu lista,
                  tu #1 suma {proposal.ranking_insight.points_for_first} puntos, tu #2 suma{' '}
                  {Math.max(proposal.ranking_insight.points_for_first - 1, 0)}, y así
                  sucesivamente. El ranking global suma los puntos de todos los vecinos.
                </p>
                {proposal.ranking_insight.your_points != null && (
                  <p>
                    En tu priorización, esta propuesta es #{' '}
                    {(proposal.ranking_insight.your_position ?? 0) + 1} y aporta{' '}
                    <strong>{proposal.ranking_insight.your_points}</strong> puntos.
                  </p>
                )}
              </details>
            </div>
          )}
        </aside>
      </div>
    </div>
  );
}
