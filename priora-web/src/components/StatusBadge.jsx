const labels = {
  activa: 'Activa',
  en_analisis: 'En análisis',
  rechazada: 'Rechazada',
};

export default function StatusBadge({ status }) {
  return <span className={`badge badge-${status}`}>{labels[status] || status}</span>;
}
