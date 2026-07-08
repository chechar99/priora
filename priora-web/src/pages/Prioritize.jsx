import { useEffect, useState } from 'react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { useAuth } from '../context/AuthContext';
import { useNamespace } from '../context/NamespaceContext';

function SortableItem({ item, position }) {
  const { attributes, listeners, setNodeRef, transform, transition } = useSortable({
    id: item.id,
  });
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div ref={setNodeRef} style={style} className="sortable-item" {...attributes} {...listeners}>
      <span className="drag-handle">⠿</span>
      <span className="pos">{position}</span>
      <strong>{item.title}</strong>
      <StatusBadge status={item.status} />
    </div>
  );
}

export default function Prioritize() {
  const { user } = useAuth();
  const { slug, path } = useNamespace();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [items, setItems] = useState([]);
  const [saved, setSaved] = useState(false);

  const { data: proposals = [], isLoading } = useQuery({
    queryKey: ['proposals', slug, 'active'],
    queryFn: () => api.proposals(slug, 'active'),
  });

  const { data: myRanking } = useQuery({
    queryKey: ['myRanking', slug],
    queryFn: () => api.myRanking(slug),
    enabled: !!user,
  });

  useEffect(() => {
    if (!proposals.length) return;
    if (myRanking?.proposal_ids?.length) {
      const ordered = myRanking.proposal_ids
        .map((id) => proposals.find((p) => p.id === id))
        .filter(Boolean);
      const rest = proposals.filter((p) => !myRanking.proposal_ids.includes(p.id));
      setItems([...ordered, ...rest]);
    } else {
      setItems(proposals);
    }
  }, [proposals, myRanking]);

  const saveMutation = useMutation({
    mutationFn: () => api.saveRanking(slug, items.map((i) => i.id)),
    onSuccess: () => {
      setSaved(true);
      queryClient.invalidateQueries({ queryKey: ['proposals', slug] });
      queryClient.invalidateQueries({ queryKey: ['myRanking', slug] });
    },
  });

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  if (!user) {
    navigate('/login', { state: { returnTo: path('priorizar') } });
    return null;
  }

  if (!user.profile_complete) {
    navigate('/completar-perfil', { state: { returnTo: path('priorizar') } });
    return null;
  }

  return (
    <div>
      <div className="content-header">
        <div>
          <h1>Priorizar propuestas</h1>
          <p>Arrastrá para ordenar de mayor a menor prioridad</p>
        </div>
      </div>

      <div className="prioritize-panel">
        <div className="hint">
          Tu orden personal contribuye al ranking global. Los cambios se reflejan al guardar.
        </div>

        {isLoading && <p>Cargando…</p>}

        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={({ active, over }) => {
            if (!over || active.id === over.id) return;
            setItems((prev) => {
              const oldIndex = prev.findIndex((i) => i.id === active.id);
              const newIndex = prev.findIndex((i) => i.id === over.id);
              return arrayMove(prev, oldIndex, newIndex);
            });
            setSaved(false);
          }}
        >
          <SortableContext items={items.map((i) => i.id)} strategy={verticalListSortingStrategy}>
            <div className="sortable-list">
              {items.map((item, index) => (
                <SortableItem key={item.id} item={item} position={index + 1} />
              ))}
            </div>
          </SortableContext>
        </DndContext>

        <button
          type="button"
          className="btn btn-primary"
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending || !items.length}
        >
          Guardar priorización
        </button>

        {saved && <p className="success">¡Priorización guardada!</p>}
        {saveMutation.error && <p className="error">{saveMutation.error.message}</p>}
      </div>
    </div>
  );
}
