import { Link } from 'react-router-dom';
import { useNamespace } from '../context/NamespaceContext';
import StatusBadge from './StatusBadge';

export default function ProposalCard({ proposal, showRank = true, maxScore = 1 }) {
  const { path } = useNamespace();
  const excerpt =
    proposal.description.length > 120
      ? `${proposal.description.slice(0, 120)}…`
      : proposal.description;

  const score = proposal.score || 0;
  const scorePct = maxScore > 0 ? Math.round((score / maxScore) * 100) : 0;
  const isTop = showRank && proposal.rank_position === 1;

  return (
    <article className="proposal-card">
      {showRank && proposal.rank_position > 0 ? (
        <div className={`rank-circle${isTop ? ' top' : ''}`}>
          #{proposal.rank_position}
        </div>
      ) : (
        <div className="rank-circle rank-circle-empty" aria-hidden="true" />
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
