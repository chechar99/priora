import { useEffect } from 'react';
import { Outlet, useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { api, saveLastNamespace } from '../api/client';
import { NamespaceProvider } from '../context/NamespaceContext';

export default function NamespaceLayout() {
  const { namespace: slug } = useParams();

  const { data: namespace, isLoading, error } = useQuery({
    queryKey: ['namespace', slug],
    queryFn: () => api.namespace(slug),
    retry: false,
  });

  useEffect(() => {
    if (namespace?.slug) {
      saveLastNamespace(namespace.slug);
    }
  }, [namespace?.slug]);

  if (isLoading) {
    return (
      <div className="auth-main">
        <p>Cargando barrio…</p>
      </div>
    );
  }

  if (error || !namespace) {
    return (
      <div className="auth-main">
        <div className="auth-card">
          <h1>Barrio no encontrado</h1>
          <p>No existe un espacio con la ruta <code>/{slug}</code>.</p>
          <a href="/" className="btn btn-secondary" style={{ marginTop: '1rem' }}>
            Ver barrios disponibles
          </a>
        </div>
      </div>
    );
  }

  return (
    <NamespaceProvider namespace={namespace}>
      <Outlet />
    </NamespaceProvider>
  );
}
