import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { api } from '../api/client';
import { openTutorial } from '../components/TutorialOverlay';
import { useAuth } from '../context/AuthContext';

const roleLabels = {
  regular: 'Usuario regular',
  proponent: 'Proponente',
  admin: 'Administrador',
};

export default function Profile() {
  const { user, refresh } = useAuth();
  const [form, setForm] = useState({
    street: user?.street || '',
    floor_apt: user?.floor_apt || '',
    city: user?.city || '',
    postal_code: user?.postal_code || '',
  });
  const [message, setMessage] = useState('');

  const mutation = useMutation({
    mutationFn: () => api.updateProfile(form),
    onSuccess: async () => {
      await refresh();
      setMessage('Perfil actualizado');
    },
  });

  if (!user) return <p>Debes iniciar sesión.</p>;

  return (
    <div>
      <div className="content-header">
        <div>
          <h1>Mi perfil</h1>
          <p>Datos de tu cuenta y dirección en el espacio</p>
        </div>
      </div>

      <div className="panel">
        <div className="profile-info">
          <p><strong>Nombre:</strong> {user.name}</p>
          <p><strong>Email:</strong> {user.email}</p>
          <p><strong>Rol:</strong> {roleLabels[user.role] || user.role}</p>
        </div>

        <h2>Dirección</h2>
        <form
          className="form"
          onSubmit={(e) => {
            e.preventDefault();
            mutation.mutate();
          }}
        >
          <label>
            Calle y número
            <input
              required
              minLength={5}
              value={form.street}
              onChange={(e) => setForm({ ...form, street: e.target.value })}
            />
          </label>
          <label>
            Piso / Depto
            <input
              value={form.floor_apt}
              onChange={(e) => setForm({ ...form, floor_apt: e.target.value })}
            />
          </label>
          <label>
            Ciudad / Barrio
            <input
              required
              value={form.city}
              onChange={(e) => setForm({ ...form, city: e.target.value })}
            />
          </label>
          <label>
            Código postal
            <input
              value={form.postal_code}
              onChange={(e) => setForm({ ...form, postal_code: e.target.value })}
            />
          </label>
          <button type="submit" className="btn btn-primary" disabled={mutation.isPending}>
            Guardar
          </button>
        </form>
        {message && <p className="success">{message}</p>}

        <div className="hint-box">
          <p>¿Querés repasar cómo funciona Priora?</p>
          <button type="button" className="btn btn-secondary" onClick={openTutorial}>
            Ver tutorial
          </button>
        </div>
      </div>
    </div>
  );
}
