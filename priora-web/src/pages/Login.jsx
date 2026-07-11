import { useState } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { api, getLastNamespace } from '../api/client';
import { defaultNamespacePath, FOR_PREFIX, namespacePath } from '../routes';
import { useAuth } from '../context/AuthContext';

const SHOW_DEV_LOGIN = import.meta.env.DEV;

const DEV_USERS = [
  { email: 'carlos.mendez@priora.local', name: 'Carlos Méndez', role: 'regular' },
  { email: 'ana.rios@priora.local', name: 'Ana Ríos', role: 'regular' },
  { email: 'luis.torres@priora.local', name: 'Luis Torres', role: 'regular' },
  { email: 'proponente@priora.local', name: 'María Proponente', role: 'proponent' },
  { email: 'sofia.navarro@priora.local', name: 'Sofía Navarro', role: 'proponent' },
  { email: 'admin@priora.local', name: 'Administrador', role: 'admin' },
];

export default function Login() {
  const { loginWithToken } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const returnTo = location.state?.returnTo || defaultNamespacePath();

  const afterLogin = (u) => {
    navigate(u.profile_complete ? returnTo : '/completar-perfil', {
      state: u.profile_complete ? undefined : { returnTo },
    });
  };

  const handleDevLogin = async (user) => {
    setLoading(true);
    setError('');
    try {
      const res = await api.devLogin(user);
      const u = await loginWithToken(res.token);
      afterLogin(u);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="auth-main">
      <div className="auth-card">
        <Link to={FOR_PREFIX} className="auth-logo">
          <span className="auth-logo-icon">P</span>
          Priora
        </Link>
        <h1>Iniciar sesión</h1>
        <p className="subtitle" style={{ marginBottom: '1.5rem' }}>
          {SHOW_DEV_LOGIN
            ? 'Accede con tu cuenta de Google o usa un usuario de prueba para el prototipo.'
            : 'Accede con tu cuenta de Google.'}
        </p>

        <a href={api.googleLoginUrl()} className="btn btn-google">
          Continuar con Google
        </a>

        {SHOW_DEV_LOGIN && (
          <>
            <div className="divider">o usuarios de prueba</div>

            <div className="dev-users">
              {DEV_USERS.map((u) => (
                <button
                  key={u.email}
                  type="button"
                  className="btn btn-secondary"
                  disabled={loading}
                  onClick={() => handleDevLogin(u)}
                >
                  {u.name} ({u.role})
                </button>
              ))}
            </div>

            <div className="hint-box">
              <p><strong>Impersonación (admin):</strong></p>
              <p>
                Como administrador, visita{' '}
                <code>{namespacePath(getLastNamespace())}?priora_as=carlos.mendez@priora.local</code> para actuar como otro usuario.
              </p>
              <p>
                Con <code>DEV_IMPERSONATION=true</code> en el backend, puedes usar esa URL sin iniciar
                sesión. En producción solo los administradores pueden impersonar.
              </p>
            </div>
          </>
        )}

        {error && <p className="error">{error}</p>}
      </div>
    </div>
  );
}
