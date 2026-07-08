import { createContext, useContext, useEffect, useState } from 'react';
import { api, setToken } from '../api/client';

const AuthContext = createContext(null);

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [impersonator, setImpersonator] = useState(null);
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    try {
      const data = await api.me();
      const { impersonator: imp, ...profile } = data;
      setUser(profile);
      setImpersonator(imp || null);
      return profile;
    } catch {
      setUser(null);
      setImpersonator(null);
      setToken(null);
      return null;
    }
  };

  useEffect(() => {
    refresh().finally(() => setLoading(false));
  }, []);

  const loginWithToken = async (token, { saveReturnToken = false } = {}) => {
    if (saveReturnToken) {
      const current = localStorage.getItem('priora_token');
      if (current) localStorage.setItem('priora_return_token', current);
    }
    setToken(token);
    return refresh();
  };

  const stopImpersonating = async () => {
    try {
      if (impersonator) {
        const res = await api.stopImpersonate();
        setToken(res.token);
        localStorage.removeItem('priora_return_token');
        return refresh();
      }
      const returnToken = localStorage.getItem('priora_return_token');
      if (returnToken) {
        localStorage.removeItem('priora_return_token');
        setToken(returnToken);
        return refresh();
      }
    } catch {
      setToken(null);
      setUser(null);
      setImpersonator(null);
    }
    return null;
  };

  const logout = () => {
    setToken(null);
    setUser(null);
    setImpersonator(null);
    localStorage.removeItem('priora_return_token');
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        impersonator,
        loading,
        refresh,
        loginWithToken,
        stopImpersonating,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
