import { useEffect, useState } from 'react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  arrayMove,
  defaultAnimateLayoutChanges,
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

/** dnd-kit only animates after a drag by default; enable it for arrow reorders too. */
function animateLayoutChanges(args) {
  if (typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    return false;
  }
  const { isSorting, wasDragging } = args;
  if (isSorting || wasDragging) {
    return defaultAnimateLayoutChanges(args);
  }
  return true;
}

const sortableTransition = {
  duration: 220,
  easing: 'cubic-bezier(0.25, 1, 0.5, 1)',
};

function SortableItem({ item, position, isFirst, isLast, onMove }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: item.id,
    animateLayoutChanges,
    transition: sortableTransition,
  });
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.85 : undefined,
  };

  return (
    <div ref={setNodeRef} style={style} className="sortable-item">
      <span className="drag-handle" {...attributes} {...listeners} aria-label="Arrastrar para reordenar">
        ⠿
      </span>
      <span className="pos">{position}</span>
      <strong>{item.title}</strong>
      <StatusBadge status={item.status} />
      <div className="priority-arrows">
        <button
          type="button"
          className="priority-arrow"
          aria-label="Subir prioridad"
          disabled={isFirst}
          onClick={() => onMove('up')}
        >
          ▲
        </button>
        <button
          type="button"
          className="priority-arrow"
          aria-label="Bajar prioridad"
          disabled={isLast}
          onClick={() => onMove('down')}
        >
          ▼
        </button>
      </div>
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
    useSensor(MouseSensor, {
      activationConstraint: { distance: 5 },
    }),
    useSensor(TouchSensor, {
      activationConstraint: { delay: 250, tolerance: 5 },
    }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  function moveItem(id, direction) {
    setItems((prev) => {
      const index = prev.findIndex((i) => i.id === id);
      if (index < 0) return prev;
      const target = direction === 'up' ? index - 1 : index + 1;
      if (target < 0 || target >= prev.length) return prev;
      return arrayMove(prev, index, target);
    });
    setSaved(false);
  }

  if (!user) {
    navigate('/login', { state: { returnTo: path('priorizar') } });
    return null;
  }

  if (!user.profile_complete) {
    navigate('/completar-perfil', { state: { returnTo: path('priorizar') } });
    return null;
  }

  return (
    <div className="content-narrow">
      <div className="content-header">
        <div>
          <h1>Priorizar propuestas</h1>
          <p>Ordená de mayor a menor prioridad</p>
        </div>
      </div>

      <div className="prioritize-panel">
        <div className="hint">
          Tu orden personal contribuye al ranking global (método Borda). Si hay{' '}
          {items.length || 'N'} propuestas, tu #1 suma {items.length || 'N'} puntos, tu #2 suma{' '}
          {Math.max((items.length || 1) - 1, 0)}, y así hasta 1 punto para la última. Los
          cambios se reflejan al guardar.
        </div>

        <div className="banner prioritize-mobile-hint" role="note">
          En el celular, mantené presionado el ícono ⠿ para arrastrar. Si solo deslizás, la
          pantalla hace scroll. También podés usar las flechas ▲ ▼.
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
                <SortableItem
                  key={item.id}
                  item={item}
                  position={index + 1}
                  isFirst={index === 0}
                  isLast={index === items.length - 1}
                  onMove={(direction) => moveItem(item.id, direction)}
                />
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
