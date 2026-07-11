import { useEffect, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Link, useNavigate, useParams } from 'react-router-dom';
import { api } from '../api/client';
import LogoField from '../components/LogoField';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

export default function EditProposal() {
  const { id } = useParams();
  const { user } = useAuth();
  const { slug, path } = useNamespace();
  const navigate = useNavigate();
  const [form, setForm] = useState({
    title: '',
    description: '',
    logo_url: '',
    category_id: '',
  });
  const [error, setError] = useState('');
  const [ready, setReady] = useState(false);

  const { data: categories = [], isLoading: loadingCategories } = useQuery({
    queryKey: ['categories'],
    queryFn: () => api.categories(),
  });

  const { data: proposal, isLoading, error: loadError } = useQuery({
    queryKey: ['proposal', slug, id],
    queryFn: () => api.proposal(slug, id),
  });

  useEffect(() => {
    if (!proposal || ready) return;
    setForm({
      title: proposal.title || '',
      description: proposal.description || '',
      logo_url: proposal.logo_url || '',
      category_id: proposal.category?.id || '',
    });
    setReady(true);
  }, [proposal, ready]);

  const mutation = useMutation({
    mutationFn: () =>
      api.updateProposal(slug, id, {
        title: form.title,
        description: form.description,
        logo_url: form.logo_url || '',
        category_id: form.category_id,
      }),
    onSuccess: (data) => navigate(path(`propuestas/${data.id}`)),
    onError: (e) => setError(e.message),
  });

  if (isLoading || !ready) return <p>Cargando…</p>;
  if (loadError) return <p className="error">{loadError.message}</p>;
  if (!proposal) return null;

  const isAuthor = user?.id === proposal.author?.id;
  const canEdit =
    user?.role === 'admin' || (isAuthor && proposal.status === 'activa');

  if (!canEdit) {
    return (
      <div className="panel page-narrow">
        <p>No tenés permiso para editar esta propuesta.</p>
        <Link to={path(`propuestas/${id}`)}>Volver</Link>
      </div>
    );
  }

  return (
    <div>
      <div className="content-header">
        <div>
          <h1>Editar propuesta</h1>
          <p>Actualizá el contenido de la propuesta</p>
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
          <LogoField
            value={form.logo_url}
            onChange={(logo_url) => setForm({ ...form, logo_url })}
          />
          {error && <p className="error">{error}</p>}
          <div className="actions">
            <Link to={path(`propuestas/${id}`)} className="btn btn-secondary">
              Cancelar
            </Link>
            <button type="submit" className="btn btn-primary" disabled={mutation.isPending}>
              Guardar cambios
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
