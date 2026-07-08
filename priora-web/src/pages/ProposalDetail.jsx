import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';
import { api } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

export default function ProposalDetail() {
  const { id } = useParams();
  const { slug, path } = useNamespace();
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [comment, setComment] = useState('');
  const [showAll, setShowAll] = useState(false);

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

  const statusMutation = useMutation({
    mutationFn: (status) => api.updateStatus(slug, id, status),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['proposal', slug, id] });
      queryClient.invalidateQueries({ queryKey: ['proposals', slug] });
    },
  });

  if (isLoading) return <p>Cargando…</p>;
  if (error) return <p className="error">{error.message}</p>;
  if (!proposal) return null;

  const canComment =
    user?.profile_complete &&
    proposal.status !== 'rechazada';
  const isAdmin = user?.role === 'admin';
  const commentCount = commentsPage?.total ?? commentsPage?.comments?.length ?? 0;

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
            <div className="body">{proposal.description}</div>

            {isAdmin && (
              <div className="admin-panel">
                <h3>Administración</h3>
                <div className="admin-actions">
                  {['activa', 'en_analisis', 'rechazada'].map((s) => (
                    <button
                      key={s}
                      type="button"
                      className={`btn btn-small btn-secondary ${proposal.status === s ? 'active' : ''}`}
                      onClick={() => statusMutation.mutate(s)}
                    >
                      {s}
                    </button>
                  ))}
                </div>
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
            {proposal.tracker && (
              <div className="row">
                <span>Responsable</span>
                <strong>{proposal.tracker.name}</strong>
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
                  <p className="score-note">Mayor puntuación del barrio</p>
                )}
              </div>
            </div>
          )}
        </aside>
      </div>
    </div>
  );
}
