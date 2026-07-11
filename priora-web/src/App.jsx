import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import Layout from './components/Layout';
import NamespaceLayout from './components/NamespaceLayout';
import SettingsLayout from './components/SettingsLayout';
import ImpersonationHandler from './components/ImpersonationHandler';
import { api } from './api/client';
import { AuthProvider, useAuth } from './context/AuthContext';
import { useOptionalNamespace } from './context/NamespaceContext';
import { defaultNamespacePath, FOR_PREFIX } from './routes';
import AdminSettings from './pages/AdminSettings';
import AuthCallback from './pages/AuthCallback';
import CompleteProfile from './pages/CompleteProfile';
import CreateProposal from './pages/CreateProposal';
import Home from './pages/Home';
import Login from './pages/Login';
import NamespacePicker from './pages/NamespacePicker';
import Prioritize from './pages/Prioritize';
import Profile from './pages/Profile';
import ProposalDetail from './pages/ProposalDetail';
import './index.css';

const queryClient = new QueryClient();

function ProtectedRoute({ children, requireProfile = false, requireSpaceAdmin = false }) {
  const { user, loading } = useAuth();
  const ns = useOptionalNamespace();
  const slug = ns?.slug;

  const { data: membership, isLoading: membershipLoading } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user && requireSpaceAdmin && !!slug,
  });

  if (loading || (requireSpaceAdmin && user && membershipLoading)) return <p>Cargando…</p>;
  if (!user) return <Navigate to="/login" replace />;
  if (requireProfile && !user.profile_complete) {
    return <Navigate to="/completar-perfil" replace state={{ returnTo: ns?.path() }} />;
  }
  if (requireSpaceAdmin) {
    const allowed = user.role === 'admin' || membership?.can_manage_space;
    if (!allowed) {
      return <Navigate to={ns?.path() || '/'} replace />;
    }
  }
  return children;
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <BrowserRouter>
          <ImpersonationHandler />
          <Routes>
            <Route path="/" element={<Navigate to={FOR_PREFIX} replace />} />
            <Route path={FOR_PREFIX} element={<NamespacePicker />} />
            <Route path="/login" element={<Login />} />
            <Route path="/auth/callback" element={<AuthCallback />} />
            <Route
              path="/completar-perfil"
              element={
                <ProtectedRoute>
                  <CompleteProfile />
                </ProtectedRoute>
              }
            />
            <Route path="/settings" element={<SettingsLayout />}>
              <Route element={<Layout />}>
                <Route
                  index
                  element={
                    <ProtectedRoute requireSpaceAdmin>
                      <AdminSettings />
                    </ProtectedRoute>
                  }
                />
              </Route>
            </Route>
            <Route path={`${FOR_PREFIX}/:namespace`} element={<NamespaceLayout />}>
              <Route element={<Layout />}>
                <Route index element={<Home />} />
                <Route path="propuestas/:id" element={<ProposalDetail />} />
                <Route
                  path="propuestas/nueva"
                  element={
                    <ProtectedRoute requireProfile>
                      <CreateProposal />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="priorizar"
                  element={
                    <ProtectedRoute requireProfile>
                      <Prioritize />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="perfil"
                  element={
                    <ProtectedRoute>
                      <Profile />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="configuracion"
                  element={<Navigate to="/settings" replace />}
                />
              </Route>
            </Route>
            <Route
              path="*"
              element={<Navigate to={defaultNamespacePath()} replace />}
            />
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  );
}
