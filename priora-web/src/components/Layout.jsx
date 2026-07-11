import { useEffect, useState } from 'react';
import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import MembershipBanner from './MembershipBanner';
import InviteBanner from './InviteBanner';
import TutorialOverlay from './TutorialOverlay';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';
import { FOR_PREFIX } from '../routes';

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
  const { name, path, slug } = useNamespace();
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const basePath = path();
  const [menuOpen, setMenuOpen] = useState(false);

  const { data: membership } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user,
  });

  const canManageSpace = user?.role === 'admin' || membership?.can_manage_space;

  useEffect(() => {
    setMenuOpen(false);
  }, [pathname]);

  useEffect(() => {
    if (!menuOpen) return undefined;
    const onKey = (e) => {
      if (e.key === 'Escape') setMenuOpen(false);
    };
    document.body.style.overflow = 'hidden';
    window.addEventListener('keydown', onKey);
    return () => {
      document.body.style.overflow = '';
      window.removeEventListener('keydown', onKey);
    };
  }, [menuOpen]);

  const handleLogout = () => {
    logout();
    setMenuOpen(false);
    navigate(path());
  };

  const handleStopImpersonating = async () => {
    await stopImpersonating();
    navigate(path());
  };

  const navClass = ({ isActive }) => (isActive ? 'active' : '');

  return (
    <div className={`app${menuOpen ? ' menu-open' : ''}`}>
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

      <header className="mobile-topbar">
        <Link to={path()} className="sidebar-logo mobile-topbar-logo">
          <span className="sidebar-logo-icon">P</span>
          Priora
        </Link>
        <Link to={FOR_PREFIX} className="mobile-namespace" title="Cambiar espacio">
          {name}
        </Link>
        <button
          type="button"
          className="mobile-menu-btn"
          aria-label={menuOpen ? 'Cerrar menú' : 'Abrir menú'}
          aria-expanded={menuOpen}
          onClick={() => setMenuOpen((open) => !open)}
        >
          <span className="mobile-menu-icon" aria-hidden="true" />
        </button>
      </header>

      {menuOpen && (
        <button
          type="button"
          className="mobile-overlay"
          aria-label="Cerrar menú"
          onClick={() => setMenuOpen(false)}
        />
      )}

      <div className="app-shell">
        <aside className="sidebar">
          <div className="sidebar-drawer-header">
            <span className="sidebar-drawer-title">Menú</span>
            <button
              type="button"
              className="mobile-menu-btn sidebar-drawer-close"
              aria-label="Cerrar menú"
              onClick={() => setMenuOpen(false)}
            >
              <span className="mobile-menu-icon" aria-hidden="true" />
            </button>
          </div>
          <Link to={path()} className="sidebar-logo desktop-only">
            <span className="sidebar-logo-icon">P</span>
            Priora
          </Link>
          <div className="sidebar-namespace desktop-only">
            <Link to={FOR_PREFIX} className="sidebar-namespace-switch" title="Cambiar espacio">
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
            <NavLink to={path('priorizar')} className={navClass}>
              <span className="icon">↕</span>
              Priorizar
            </NavLink>
            <NavLink to={user ? path('perfil') : '/login'} className={navClass}>
              <span className="icon">👤</span>
              Perfil
            </NavLink>
            {canManageSpace && (
              <NavLink to="/settings" className={navClass}>
                <span className="icon">⚙</span>
                Configuración
              </NavLink>
            )}
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
          <InviteBanner />
          <MembershipBanner />
          <Outlet />
        </main>
      </div>

      <TutorialOverlay />
    </div>
  );
}
