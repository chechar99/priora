import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { api } from '../api/client';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

export default function MembershipBanner() {
  const { user } = useAuth();
  const { slug } = useNamespace();
  const queryClient = useQueryClient();

  const { data: membership } = useQuery({
    queryKey: ['membership', slug],
    queryFn: () => api.membershipMe(slug),
    enabled: !!user?.profile_complete,
  });

  const request = useMutation({
    mutationFn: () => api.requestMembership(slug),
    onSuccess: (data) => {
      queryClient.setQueryData(['membership', slug], data);
    },
  });

  if (!user?.profile_complete || !membership?.require_member_approval) {
    return null;
  }

  if (membership.ranking_counts && membership.can_comment) {
    return null;
  }

  const status = membership.membership?.status;

  if (status === 'disabled') {
    return (
      <div className="banner warning" role="status">
        Tu acceso a este espacio está deshabilitado. Un administrador puede rehabilitarte.
      </div>
    );
  }

  if (status === 'pending') {
    return (
      <div className="banner warning" role="status">
        Tu solicitud de autorización está pendiente. Podés priorizar, pero no tendrá efecto
        hasta que un admin te autorice. Tampoco podés comentar todavía.
      </div>
    );
  }

  if (status === 'rejected') {
    return (
      <div className="banner warning" role="status">
        Tu solicitud fue rechazada. Podés priorizar, pero no tendrá efecto hasta que un admin
        te autorice.{' '}
        <button
          type="button"
          className="banner-link"
          disabled={request.isPending}
          onClick={() => request.mutate()}
        >
          Volver a solicitar autorización
        </button>
        {request.error && <span className="error"> — {request.error.message}</span>}
      </div>
    );
  }

  return (
    <div className="banner warning" role="status">
      Podés priorizar, pero no tendrá efecto hasta que un admin te autorice.{' '}
      <button
        type="button"
        className="banner-link"
        disabled={request.isPending}
        onClick={() => request.mutate()}
      >
        Solicitá autorización aquí
      </button>
      . Tampoco podés comentar hasta ser autorizado.
      {request.error && <span className="error"> — {request.error.message}</span>}
    </div>
  );
}
