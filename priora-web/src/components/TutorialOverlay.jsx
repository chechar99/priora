import { useEffect, useState } from 'react';

const STORAGE_KEY = 'priora_tutorial_dismissed';

const STEPS = [
  {
    icon: '👋',
    title: 'Bienvenido a Priora',
    body: 'Priora te ayuda a proponer mejoras y decidir en comunidad qué es más importante. Este recorrido rápido te muestra las funciones principales.',
  },
  {
    icon: '🏠',
    title: 'Tu espacio',
    body: 'Cada espacio (barrio, edificio, organización) tiene sus propias propuestas y ranking. Tocá el nombre del espacio en el menú para cambiar a otro.',
  },
  {
    icon: '📋',
    title: 'Propuestas',
    body: 'En Propuestas ves el ranking comunitario: las ideas ordenadas por el apoyo de todos. Filtrá por estado o categoría y abrí una para ver el detalle.',
  },
  {
    icon: '↕',
    title: 'Priorizar',
    body: 'En Priorizar ordenás las propuestas según lo que más te importa. Arrastrá para reordenar y guardá: tu orden suma al ranking global del espacio.',
  },
  {
    icon: '＋',
    title: 'Nueva propuesta',
    body: 'Con el botón + Nueva podés compartir una idea de mejora. Completá título, categoría y descripción para que el resto del espacio la priorice.',
  },
  {
    icon: '💬',
    title: 'Comentarios',
    body: 'Dentro de cada propuesta podés leer y publicar comentarios. Si el espacio pide autorización, un admin debe aprobarte antes de comentar.',
  },
  {
    icon: '👤',
    title: 'Perfil y participación',
    body: 'En Perfil completás tu dirección para participar. Iniciá sesión para priorizar, comentar y crear propuestas. ¡Listo para empezar!',
  },
];

export function isTutorialDismissed() {
  try {
    return localStorage.getItem(STORAGE_KEY) === '1';
  } catch {
    return false;
  }
}

export function dismissTutorial() {
  try {
    localStorage.setItem(STORAGE_KEY, '1');
  } catch {
    /* ignore quota / private mode */
  }
}

export function resetTutorial() {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    /* ignore */
  }
}

export const TUTORIAL_OPEN_EVENT = 'priora-tutorial-open';

export function openTutorial() {
  window.dispatchEvent(new Event(TUTORIAL_OPEN_EVENT));
}

export default function TutorialOverlay({ forceOpen = false, onClose }) {
  const [open, setOpen] = useState(() => forceOpen || !isTutorialDismissed());
  const [step, setStep] = useState(0);

  useEffect(() => {
    if (forceOpen) {
      setOpen(true);
      setStep(0);
    }
  }, [forceOpen]);

  useEffect(() => {
    const onOpen = () => {
      setStep(0);
      setOpen(true);
    };
    window.addEventListener(TUTORIAL_OPEN_EVENT, onOpen);
    return () => window.removeEventListener(TUTORIAL_OPEN_EVENT, onOpen);
  }, []);

  useEffect(() => {
    if (!open) return undefined;
    const onKey = (e) => {
      if (e.key === 'Escape') {
        setOpen(false);
        onClose?.();
      }
    };
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    window.addEventListener('keydown', onKey);
    return () => {
      document.body.style.overflow = prevOverflow;
      window.removeEventListener('keydown', onKey);
    };
  }, [open, onClose]);

  const finish = (permanent) => {
    if (permanent) dismissTutorial();
    setOpen(false);
    onClose?.();
  };

  if (!open) return null;

  const current = STEPS[step];
  const isLast = step === STEPS.length - 1;
  const progress = ((step + 1) / STEPS.length) * 100;

  return (
    <div className="tutorial-overlay" role="dialog" aria-modal="true" aria-labelledby="tutorial-title">
      <div className="tutorial-backdrop" aria-hidden="true" />
      <div className="tutorial-card">
        <div className="tutorial-progress" aria-hidden="true">
          <div className="tutorial-progress-bar" style={{ width: `${progress}%` }} />
        </div>

        <p className="tutorial-step-label">
          Paso {step + 1} de {STEPS.length}
        </p>

        <div className="tutorial-icon" aria-hidden="true">{current.icon}</div>
        <h2 id="tutorial-title">{current.title}</h2>
        <p className="tutorial-body">{current.body}</p>

        <div className="tutorial-dots" role="tablist" aria-label="Pasos del tutorial">
          {STEPS.map((s, i) => (
            <button
              key={s.title}
              type="button"
              role="tab"
              aria-selected={i === step}
              aria-label={`Ir al paso ${i + 1}: ${s.title}`}
              className={`tutorial-dot${i === step ? ' active' : ''}${i < step ? ' done' : ''}`}
              onClick={() => setStep(i)}
            />
          ))}
        </div>

        <div className="tutorial-actions">
          {step > 0 ? (
            <button type="button" className="btn btn-secondary" onClick={() => setStep((s) => s - 1)}>
              Anterior
            </button>
          ) : (
            <span />
          )}
          {isLast ? (
            <button type="button" className="btn btn-primary" onClick={() => finish(true)}>
              Empezar
            </button>
          ) : (
            <button type="button" className="btn btn-primary" onClick={() => setStep((s) => s + 1)}>
              Siguiente
            </button>
          )}
        </div>

        <button type="button" className="tutorial-dismiss" onClick={() => finish(true)}>
          No mostrar más el tutorial
        </button>
      </div>
    </div>
  );
}
