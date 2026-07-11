import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { FOR_PREFIX, namespacePath } from '../routes';

export default function NamespacePicker() {
  const { data: namespaces = [], isLoading, error } = useQuery({
    queryKey: ['namespaces'],
    queryFn: () => api.namespaces(),
  });

  return (
    <div className="auth-main">
      <div className="auth-card" style={{ maxWidth: '480px' }}>
        <div className="auth-logo">
          <span className="auth-logo-icon">P</span>
          Priora
        </div>
        <h1>Elegí tu espacio</h1>
        <p className="subtitle">
          Cada espacio tiene sus propias propuestas y ranking de priorización.
        </p>

        {isLoading && <p>Cargando espacios…</p>}
        {error && <p className="error">{error.message}</p>}

        <div className="namespace-list">
          {namespaces.map((ns) => (
            <Link key={ns.id} to={namespacePath(ns.slug)} className="namespace-card">
              <strong>{ns.name}</strong>
              <span>{FOR_PREFIX}/{ns.slug}</span>
            </Link>
          ))}
        </div>

        <p className="hint-box" style={{ marginTop: '1.5rem' }}>
          Accedé directamente con <code>{FOR_PREFIX}/barrio-test</code>.
        </p>
      </div>
    </div>
  );
}
