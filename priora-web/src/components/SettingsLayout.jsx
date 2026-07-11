import { useEffect } from 'react';
import { Link, Outlet } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { api, getLastNamespace, saveLastNamespace } from '../api/client';
import { NamespaceProvider } from '../context/NamespaceContext';
import { FOR_PREFIX } from '../routes';

/** Carga el último espacio usado y provee contexto para /settings. */
export default function SettingsLayout() {
  const slug = getLastNamespace();

  const { data: namespace, isLoading, error } = useQuery({
    queryKey: ['namespace', slug],
    queryFn: () => api.namespace(slug),
    retry: false,
    enabled: !!slug,
  });

  useEffect(() => {
    if (namespace?.slug) {
      saveLastNamespace(namespace.slug);
    }
  }, [namespace?.slug]);

  if (isLoading) {
    return (
      <div className="auth-main">
        <p>Cargando espacio…</p>
      </div>
    );
  }

  if (error || !namespace) {
    return (
      <div className="auth-main">
        <div className="auth-card">
          <h1>Elegí un espacio</h1>
          <p>
            Para abrir la configuración necesitás haber visitado un espacio antes.
          </p>
          <Link to={FOR_PREFIX} className="btn btn-secondary" style={{ marginTop: '1rem' }}>
            Ver espacios disponibles
          </Link>
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
