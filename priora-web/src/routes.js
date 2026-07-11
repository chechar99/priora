import { getLastNamespace } from './api/client';

/** Prefijo de rutas del frontend en producción: /for/{espacio} */
export const FOR_PREFIX = '/for';

export function namespacePath(slug, subpath = '') {
  const suffix = subpath.startsWith('/')
    ? subpath
    : subpath
      ? `/${subpath}`
      : '';
  return `${FOR_PREFIX}/${slug}${suffix}`;
}

export function defaultNamespacePath() {
  return namespacePath(getLastNamespace());
}
