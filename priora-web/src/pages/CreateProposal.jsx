import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Link, useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

export default function CreateProposal() {
  const { user } = useAuth();
  const { slug, path } = useNamespace();
  const navigate = useNavigate();
  const [form, setForm] = useState({ title: '', description: '', logo_url: '', category_id: '' });
  const [error, setError] = useState('');

  const { data: categories = [], isLoading: loadingCategories } = useQuery({
    queryKey: ['categories'],
    queryFn: () => api.categories(),
  });

  const { data: membership, isLoading: loadingMembership } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user,
  });

  const mutation = useMutation({
    mutationFn: () =>
      api.createProposal(slug, {
        title: form.title,
        description: form.description,
        logo_url: form.logo_url || null,
        category_id: form.category_id,
      }),
    onSuccess: (data) => navigate(path(`propuestas/${data.id}`)),
    onError: (e) => setError(e.message),
  });

  if (loadingMembership) {
    return <p>Cargando…</p>;
  }

  if (!user || !membership?.can_create_proposal) {
    return (
      <div className="panel page-narrow">
        <p>No tienes permiso para crear propuestas.</p>
        <Link to={path()}>Volver</Link>
      </div>
    );
  }

  return (
    <div>
      <div className="content-header">
        <div>
          <h1>Nueva propuesta</h1>
          <p>Compartí una idea de mejora para el espacio</p>
        </div>
      </div>

      <div className="panel">
        <form
          className="form"
          onSubmit={(e) => {
            e.preventDefault();
            mutation.mutate();
          }}
        >
          <label>
            Título
            <input
              required
              maxLength={200}
              value={form.title}
              onChange={(e) => setForm({ ...form, title: e.target.value })}
            />
          </label>
          <label>
            Categoría
            <select
              required
              value={form.category_id}
              onChange={(e) => setForm({ ...form, category_id: e.target.value })}
              disabled={loadingCategories}
            >
              <option value="">Seleccioná una categoría</option>
              {categories.map((category) => (
                <option key={category.id} value={category.id}>
                  {category.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Descripción
            <textarea
              required
              rows={6}
              maxLength={5000}
              value={form.description}
              onChange={(e) => setForm({ ...form, description: e.target.value })}
            />
          </label>
          <label>
            URL del logo (opcional)
            <input
              type="url"
              value={form.logo_url}
              onChange={(e) => setForm({ ...form, logo_url: e.target.value })}
            />
          </label>
          {error && <p className="error">{error}</p>}
          <div className="actions">
            <Link to={path()} className="btn btn-secondary">Cancelar</Link>
            <button type="submit" className="btn btn-primary" disabled={mutation.isPending}>
              Crear propuesta
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
