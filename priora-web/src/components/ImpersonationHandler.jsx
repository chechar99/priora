import { useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { api, IMPERSONATE_QUERY_KEY } from '../api/client';
import { useAuth } from '../context/AuthContext';

export default function ImpersonationHandler() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { loginWithToken, user } = useAuth();

  useEffect(() => {
    const target = searchParams.get(IMPERSONATE_QUERY_KEY);
    if (!target) return;

    let cancelled = false;

    (async () => {
      try {
        const hadSession = !!user;
        const res = await api.impersonate(target);
        if (cancelled) return;
        await loginWithToken(res.token, { saveReturnToken: hadSession && !!res.impersonator });
      } catch (err) {
        console.error('Impersonation failed:', err.message);
      } finally {
        if (!cancelled) {
          const next = new URLSearchParams(searchParams);
          next.delete(IMPERSONATE_QUERY_KEY);
          setSearchParams(next, { replace: true });
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [searchParams, setSearchParams, loginWithToken, user]);

  return null;
}
