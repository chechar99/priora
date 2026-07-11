const API_BASE = import.meta.env.VITE_API_URL || '';
export const IMPERSONATE_QUERY_KEY =
  import.meta.env.VITE_IMPERSONATE_QUERY_KEY || 'priora_as';
export const LAST_NAMESPACE_KEY = 'priora_last_namespace';

function getToken() {
  return localStorage.getItem('priora_token');
}

export function setToken(token) {
  if (token) localStorage.setItem('priora_token', token);
  else localStorage.removeItem('priora_token');
}

async function request(path, options = {}) {
  const headers = { ...(options.headers || {}) };
  const token = getToken();
  if (token) headers.Authorization = `Bearer ${token}`;
  if (options.body && !(options.body instanceof FormData)) {
    headers['Content-Type'] = 'application/json';
  }

  const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || 'request failed');
  }
  if (res.status === 204) return null;
  return res.json();
}

function nsPrefix(namespace) {
  return `/api/${namespace}`;
}

export const api = {
  devLogin: (data) =>
    request('/api/auth/dev-login', { method: 'POST', body: JSON.stringify(data) }),
  me: () => request('/api/auth/me'),
  impersonate: (reference) =>
    request(
      `/api/auth/impersonate?${IMPERSONATE_QUERY_KEY}=${encodeURIComponent(reference)}`,
    ),
  stopImpersonate: () =>
    request('/api/auth/stop-impersonate', { method: 'POST' }),
  updateProfile: (data) =>
    request('/api/users/me', { method: 'PATCH', body: JSON.stringify(data) }),
  users: () => request('/api/users'),
  updateUserRole: (id, role) =>
    request(`/api/users/${id}/role`, {
      method: 'PATCH',
      body: JSON.stringify({ role }),
    }),
  namespaces: () => request('/api/namespaces'),
  namespace: (slug) => request(`/api/namespaces/${slug}`),
  createNamespace: (data) =>
    request('/api/namespaces', { method: 'POST', body: JSON.stringify(data) }),
  updateNamespace: (slug, data) =>
    request(`/api/namespaces/${slug}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),
  membershipMe: (namespace) => request(`${nsPrefix(namespace)}/membership/me`),
  requestMembership: (namespace) =>
    request(`${nsPrefix(namespace)}/membership/request`, { method: 'POST' }),
  members: (namespace, status) => {
    const params = new URLSearchParams();
    if (status) params.set('status', status);
    const q = params.toString();
    return request(`${nsPrefix(namespace)}/members${q ? `?${q}` : ''}`);
  },
  updateMember: (namespace, userId, data) =>
    request(`${nsPrefix(namespace)}/members/${userId}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),
  proposals: (namespace, filter = 'active', category) => {
    const params = new URLSearchParams({ filter });
    if (category) params.set('category', category);
    return request(`${nsPrefix(namespace)}/proposals?${params}`);
  },
  categories: () => request('/api/categories'),
  proposal: (namespace, id) => request(`${nsPrefix(namespace)}/proposals/${id}`),
  createProposal: (namespace, data) =>
    request(`${nsPrefix(namespace)}/proposals`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  updateProposal: (namespace, id, data) =>
    request(`${nsPrefix(namespace)}/proposals/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),
  updateStatus: (namespace, id, status) =>
    request(`${nsPrefix(namespace)}/proposals/${id}/status`, {
      method: 'PATCH',
      body: JSON.stringify({ status }),
    }),
  comments: (namespace, id, limit = 10, offset = 0) =>
    request(`${nsPrefix(namespace)}/proposals/${id}/comments?limit=${limit}&offset=${offset}`),
  addComment: (namespace, id, content) =>
    request(`${nsPrefix(namespace)}/proposals/${id}/comments`, {
      method: 'POST',
      body: JSON.stringify({ content }),
    }),
  myRanking: (namespace) => request(`${nsPrefix(namespace)}/rankings/me`),
  saveRanking: (namespace, proposal_ids) =>
    request(`${nsPrefix(namespace)}/rankings/me`, {
      method: 'PUT',
      body: JSON.stringify({ proposal_ids }),
    }),
  googleLoginUrl: () => `${API_BASE}/api/auth/google`,
};

export function saveLastNamespace(slug) {
  if (slug) localStorage.setItem(LAST_NAMESPACE_KEY, slug);
}

export function getLastNamespace() {
  return localStorage.getItem(LAST_NAMESPACE_KEY) || 'barrio-test';
}
