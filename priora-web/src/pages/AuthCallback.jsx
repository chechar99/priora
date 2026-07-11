import { useEffect, useState } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { defaultNamespacePath } from '../routes';
import { useAuth } from '../context/AuthContext';

export default function AuthCallback() {
  const { loginWithToken } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [error, setError] = useState('');

  useEffect(() => {
    const params = new URLSearchParams(location.search);
    const token = params.get('token');
    if (!token) {
      setError('Token no recibido');
      return;
    }
    const returnTo = location.state?.returnTo || defaultNamespacePath();
    loginWithToken(token)
      .then((u) =>
        navigate(u?.profile_complete ? returnTo : '/completar-perfil', {
          state: u?.profile_complete ? undefined : { returnTo },
        }),
      )
      .catch(() => setError('Error al autenticar'));
  }, [location, loginWithToken, navigate]);

  if (error) {
    return (
      <div className="auth-main">
        <div className="auth-card">
          <p className="error">{error}</p>
          <Link to="/login" className="btn btn-secondary" style={{ marginTop: '1rem' }}>
            Volver al login
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="auth-main">
      <div className="auth-card">
        <p>Cargando…</p>
      </div>
    </div>
  );
}
