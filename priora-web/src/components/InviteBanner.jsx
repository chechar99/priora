import { useEffect, useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '../api/client';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

const INVITE_STORAGE_PREFIX = 'priora_invite:';

export function peekStoredInvite(slug) {
  try {
    return sessionStorage.getItem(`${INVITE_STORAGE_PREFIX}${slug}`) || '';
  } catch {
    return '';
  }
}

export function clearStoredInvite(slug) {
  try {
    sessionStorage.removeItem(`${INVITE_STORAGE_PREFIX}${slug}`);
  } catch {
    /* ignore */
  }
}

export default function InviteBanner() {
  const { user } = useAuth();
  const { slug, name, path } = useNamespace();
  const [searchParams, setSearchParams] = useSearchParams();
  const queryClient = useQueryClient();
  const [inviteCode, setInviteCode] = useState('');
  const [status, setStatus] = useState('idle'); // idle | success | error
  const [message, setMessage] = useState('');
  const attempted = useRef(false);

  useEffect(() => {
    const fromQuery = searchParams.get('invite')?.trim() || '';
    if (fromQuery) {
      try {
        sessionStorage.setItem(`${INVITE_STORAGE_PREFIX}${slug}`, fromQuery);
      } catch {
        /* ignore */
      }
      setInviteCode(fromQuery);
      const next = new URLSearchParams(searchParams);
      next.delete('invite');
      setSearchParams(next, { replace: true });
      return;
    }
    setInviteCode(peekStoredInvite(slug));
  }, [slug, searchParams, setSearchParams]);

  const redeem = useMutation({
    mutationFn: (code) => api.acceptInvite(slug, code),
    onSuccess: (data) => {
      clearStoredInvite(slug);
      setInviteCode('');
      setStatus('success');
      setMessage('¡Listo! Ya formás parte de este espacio.');
      queryClient.setQueryData(['membership', slug], data);
      queryClient.invalidateQueries({ queryKey: ['membership', slug] });
    },
    onError: (e) => {
      setStatus('error');
      setMessage(e.message || 'No se pudo aceptar la invitación');
    },
  });

  useEffect(() => {
    if (!inviteCode || !user?.profile_complete || attempted.current) return;
    attempted.current = true;
    redeem.mutate(inviteCode);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- redeem once per mount/code
  }, [inviteCode, user?.profile_complete]);

  if (status === 'success') {
    return (
      <div className="banner success-banner" role="status">
        {message}
      </div>
    );
  }

  if (!inviteCode) {
    if (status === 'error' && message) {
      return (
        <div className="banner warning" role="status">
          {message}
        </div>
      );
    }
    return null;
  }

  if (!user) {
    return (
      <div className="banner invite-banner" role="status">
        Te invitaron a <strong>{name}</strong>.{' '}
        <Link to="/login" state={{ returnTo: path() }}>
          Iniciá sesión
        </Link>{' '}
        para unirte al espacio.
      </div>
    );
  }

  if (!user.profile_complete) {
    return (
      <div className="banner invite-banner" role="status">
        Te invitaron a <strong>{name}</strong>. Completá tu perfil para unirte.
      </div>
    );
  }

  if (redeem.isPending || status === 'idle') {
    return (
      <div className="banner invite-banner" role="status">
        Aceptando invitación a <strong>{name}</strong>…
      </div>
    );
  }

  if (status === 'error') {
    return (
      <div className="banner warning" role="status">
        {message}{' '}
        <button
          type="button"
          className="banner-link"
          disabled={redeem.isPending}
          onClick={() => {
            attempted.current = true;
            redeem.mutate(inviteCode);
          }}
        >
          Reintentar
        </button>
      </div>
    );
  }

  return null;
}
