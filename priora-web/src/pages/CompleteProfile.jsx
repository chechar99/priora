import { useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { api, getLastNamespace } from '../api/client';
import { useAuth } from '../context/AuthContext';

export default function CompleteProfile() {
  const { refresh } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [form, setForm] = useState({
    street: '',
    floor_apt: '',
    city: '',
    postal_code: '',
  });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const returnTo = location.state?.returnTo || `/${getLastNamespace()}`;

  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      await api.updateProfile(form);
      await refresh();
      navigate(returnTo);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="auth-main">
      <div className="panel" style={{ maxWidth: '480px', width: '100%' }}>
        <div className="content-header">
          <div>
            <h1>Completa tu perfil</h1>
            <p>Indica tu dirección para identificarte dentro del barrio</p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="form">
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
          {error && <p className="error">{error}</p>}
          <button type="submit" className="btn btn-primary" disabled={loading}>
            Guardar y continuar
          </button>
        </form>
      </div>
    </div>
  );
}
