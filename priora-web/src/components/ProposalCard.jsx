import { Link } from 'react-router-dom';
import { assetUrl } from '../api/client';
import { useNamespace } from '../context/NamespaceContext';
import StatusBadge from './StatusBadge';

const AGREEMENT_LABELS = {
  consensus: 'Consenso',
  polarized: 'Divide',
};

export default function ProposalCard({ proposal, showRank = true, maxScore = 1 }) {
  const { path } = useNamespace();
  const excerpt =
    proposal.description.length > 120
      ? `${proposal.description.slice(0, 120)}…`
      : proposal.description;

  const score = proposal.score || 0;
  const scorePct = maxScore > 0 ? Math.round((score / maxScore) * 100) : 0;
  const isTop = showRank && proposal.rank_position === 1;
  const agreement = proposal.agreement;
  const thumb = Array.isArray(proposal.image_urls) ? proposal.image_urls[0] : null;

  return (
    <article className={`proposal-card${thumb ? ' has-thumb' : ''}`}>
      {showRank && proposal.rank_position > 0 ? (
        <div className={`rank-circle${isTop ? ' top' : ''}`}>
          #{proposal.rank_position}
        </div>
      ) : (
        <div className="rank-circle rank-circle-empty" aria-hidden="true" />
      )}
      {thumb && (
        <Link to={path(`propuestas/${proposal.id}`)} className="card-thumb" tabIndex={-1}>
          <img src={assetUrl(thumb)} alt="" />
        </Link>
      )}
      <div className="card-main">
        <h2>
          <Link to={path(`propuestas/${proposal.id}`)}>{proposal.title}</Link>
        </h2>
        <p className="excerpt">{excerpt}</p>
        <div className="card-meta">
          <span>{proposal.author.name}</span>
          {proposal.category && (
            <span className="badge badge-category">{proposal.category.name}</span>
          )}
          <StatusBadge status={proposal.status} />
          {agreement && AGREEMENT_LABELS[agreement] && (
            <span
              className={`badge badge-agreement badge-${agreement}`}
              title={
                agreement === 'consensus'
                  ? 'Los vecinos coinciden en la prioridad de esta propuesta'
                  : 'Las opiniones sobre esta propuesta están divididas'
              }
            >
              {AGREEMENT_LABELS[agreement]}
            </span>
          )}
          {proposal.tracker && (
            <span className="card-tracker" title={`Responsable: ${proposal.tracker.name}`}>
              · {proposal.tracker.name}
            </span>
          )}
        </div>
      </div>
      {showRank && score > 0 && (
        <div className="score-bar">
          <div className="score">{score}</div>
          <div className="bar">
            <div className="bar-fill" style={{ width: `${scorePct}%` }} />
          </div>
        </div>
      )}
    </article>
  );
}
