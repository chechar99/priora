import { createContext, useContext, useMemo } from 'react';
import { useParams } from 'react-router-dom';

const NamespaceContext = createContext(null);

export function NamespaceProvider({ namespace, children }) {
  const value = useMemo(
    () => ({
      slug: namespace.slug,
      name: namespace.name,
      path: (subpath = '') => {
        const suffix = subpath.startsWith('/') ? subpath : subpath ? `/${subpath}` : '';
        return `/${namespace.slug}${suffix}`;
      },
    }),
    [namespace],
  );

  return (
    <NamespaceContext.Provider value={value}>
      {children}
    </NamespaceContext.Provider>
  );
}

export function useNamespace() {
  const ctx = useContext(NamespaceContext);
  if (!ctx) {
    throw new Error('useNamespace must be used within NamespaceProvider');
  }
  return ctx;
}

export function useOptionalNamespace() {
  return useContext(NamespaceContext);
}

export function useNsParams() {
  const { namespace: slug } = useParams();
  return slug;
}
