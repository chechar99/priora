import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { api } from '../api/client';
import ProposalCard from '../components/ProposalCard';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

export default function Home() {
  const { user } = useAuth();
  const { slug, name, path } = useNamespace();
  const [filter, setFilter] = useState('active');
  const [category, setCategory] = useState('');

  const { data: categories = [] } = useQuery({
    queryKey: ['categories'],
    queryFn: () => api.categories(),
  });

  const { data: proposals = [], isLoading, error } = useQuery({
    queryKey: ['proposals', slug, filter, category],
    queryFn: () => api.proposals(slug, filter, category || undefined),
  });

  const { data: activeProposals = [] } = useQuery({
    queryKey: ['proposals', slug, 'active', ''],
    queryFn: () => api.proposals(slug, 'active'),
  });

  const { data: membership } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user,
  });

  const canCreate = membership?.can_create_proposal;

  const stats = useMemo(() => ({
    activas: activeProposals.filter((p) => p.status === 'activa').length,
    analisis: activeProposals.filter((p) => p.status === 'en_analisis').length,
    total: activeProposals.length,
  }), [activeProposals]);

  const maxScore = useMemo(
    () => Math.max(...proposals.map((p) => p.score || 0), 1),
    [proposals],
  );

  return (
    <div>
      <div className="content-header">
        <div>
          <h1>Propuestas de {name}</h1>
          <p>Ranking comunitario · actualizado hoy</p>
        </div>
        <div className="actions">
          {user?.profile_complete && (
            <Link to={path('priorizar')} className="btn btn-secondary">
              ↕ Priorizar
            </Link>
          )}
          {canCreate && (
            <Link to={path('propuestas/nueva')} className="btn btn-primary">
              + Nueva
            </Link>
          )}
        </div>
      </div>

      {filter === 'active' && (
        <div className="stats-row">
          <div className="stat-card">
            <div className="label">Propuestas activas</div>
            <div className="value teal">{stats.activas}</div>
          </div>
          <div className="stat-card">
            <div className="label">En análisis</div>
            <div className="value blue">{stats.analisis}</div>
          </div>
          <div className="stat-card">
            <div className="label">En ranking</div>
            <div className="value">{stats.total}</div>
          </div>
        </div>
      )}

      <div className="toolbar">
        <div className="filters">
          <button
            type="button"
            className={filter === 'active' ? 'active' : ''}
            onClick={() => setFilter('active')}
          >
            Activas
          </button>
          <button
            type="button"
            className={filter === 'rejected' ? 'active' : ''}
            onClick={() => setFilter('rejected')}
          >
            Rechazadas
          </button>
        </div>
        <label className="category-filter">
          <span>Categoría</span>
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
          >
            <option value="">Todas</option>
            {categories.map((cat) => (
              <option key={cat.id} value={cat.id}>
                {cat.name}
              </option>
            ))}
          </select>
        </label>
      </div>

      {!user && (
        <div className="banner">
          <Link to="/login" state={{ returnTo: path() }}>Inicia sesión</Link> para priorizar y comentar propuestas.
        </div>
      )}

      {user && !user.profile_complete && (
        <div className="banner warning">
          <Link to="/completar-perfil" state={{ returnTo: path() }}>Completa tu dirección</Link> para participar.
        </div>
      )}

      {isLoading && <p>Cargando propuestas…</p>}
      {error && <p className="error">{error.message}</p>}

      <div className="proposal-grid">
        {proposals.map((p) => (
          <ProposalCard
            key={p.id}
            proposal={p}
            showRank={filter === 'active'}
            maxScore={maxScore}
          />
        ))}
        {!isLoading && proposals.length === 0 && (
          <p>No hay propuestas con estos filtros.</p>
        )}
      </div>
    </div>
  );
}
