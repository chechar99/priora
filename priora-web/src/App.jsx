import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import Layout from './components/Layout';
import NamespaceLayout from './components/NamespaceLayout';
import ImpersonationHandler from './components/ImpersonationHandler';
import { AuthProvider, useAuth } from './context/AuthContext';
import { useOptionalNamespace } from './context/NamespaceContext';
import { getLastNamespace } from './api/client';
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

function ProtectedRoute({ children, requireProfile = false }) {
  const { user, loading } = useAuth();
  const ns = useOptionalNamespace();
  if (loading) return <p>Cargando…</p>;
  if (!user) return <Navigate to="/login" replace />;
  if (requireProfile && !user.profile_complete) {
    return <Navigate to="/completar-perfil" replace state={{ returnTo: ns?.path() }} />;
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
            <Route path="/" element={<NamespacePicker />} />
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
            <Route path="/:namespace" element={<NamespaceLayout />}>
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
              </Route>
            </Route>
            <Route
              path="*"
              element={<Navigate to={`/${getLastNamespace()}`} replace />}
            />
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  );
}
