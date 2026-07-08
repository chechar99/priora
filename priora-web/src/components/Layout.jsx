import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

function getInitials(name) {
  return name
    .split(' ')
    .filter(Boolean)
    .map((part) => part[0])
    .slice(0, 2)
    .join('')
    .toUpperCase();
}

function isProposalsRoute(pathname, basePath) {
  return (
    pathname === basePath
    || pathname.startsWith(`${basePath}/propuestas/`)
  );
}

export default function Layout() {
  const { user, impersonator, logout, stopImpersonating } = useAuth();
  const { slug, name, path } = useNamespace();
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const basePath = path();

  const handleLogout = () => {
    logout();
    navigate(path());
  };

  const handleStopImpersonating = async () => {
    await stopImpersonating();
    navigate(path());
  };

  return (
    <div className="app">
      {impersonator && user && (
        <div className="impersonation-banner">
          <span>
            Actuando como <strong>{user.name}</strong> ({user.email})
          </span>
          <button type="button" className="btn btn-small" onClick={handleStopImpersonating}>
            Volver a {impersonator.name}
          </button>
        </div>
      )}

      <div className="app-shell">
        <aside className="sidebar">
          <Link to={path()} className="sidebar-logo">
            <span className="sidebar-logo-icon">P</span>
            Priora
          </Link>
          <div className="sidebar-namespace">
            <Link to="/" className="sidebar-namespace-switch" title="Cambiar barrio">
              {name}
            </Link>
          </div>
          <nav className="sidebar-nav">
            <NavLink
              to={path()}
              end
              className={({ isActive }) =>
                (isActive || isProposalsRoute(pathname, basePath) ? 'active' : '')
              }
            >
              <span className="icon">📋</span>
              Propuestas
            </NavLink>
            <NavLink to={path('priorizar')}>
              <span className="icon">↕</span>
              Priorizar
            </NavLink>
            <NavLink to={user ? path('perfil') : '/login'}>
              <span className="icon">👤</span>
              Perfil
            </NavLink>
          </nav>
          <div className="sidebar-footer">
            {user ? (
              <>
                <div className="sidebar-user">
                  <span className="avatar">{getInitials(user.name)}</span>
                  <span>{user.name}</span>
                </div>
                <button type="button" className="sidebar-logout" onClick={handleLogout}>
                  Cerrar sesión
                </button>
              </>
            ) : (
              <Link to="/login" className="sidebar-login" state={{ returnTo: path() }}>
                Iniciar sesión
              </Link>
            )}
          </div>
        </aside>
        <main className="content">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
