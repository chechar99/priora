import { useRef, useState } from 'react';
import { api, assetUrl } from '../api/client';

const ACCEPT = 'image/jpeg,image/png,image/webp';
const MAX_IMAGES = 3;

export default function ProposalImagesField({ value = [], onChange }) {
  const inputRef = useRef(null);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState('');
  const urls = Array.isArray(value) ? value.filter(Boolean) : [];
  const canAdd = urls.length < MAX_IMAGES;

  async function handleFile(file) {
    if (!file || !canAdd) return;
    setError('');
    setUploading(true);
    try {
      const result = await api.uploadImage(file);
      onChange([...urls, result.url]);
    } catch (e) {
      setError(e.message || 'No se pudo subir la imagen');
    } finally {
      setUploading(false);
      if (inputRef.current) inputRef.current.value = '';
    }
  }

  function removeAt(index) {
    onChange(urls.filter((_, i) => i !== index));
  }

  return (
    <div className="proposal-images-field">
      <span className="proposal-images-label">Imágenes (opcional)</span>
      <p className="proposal-images-hint">
        Hasta {MAX_IMAGES} · JPEG, PNG o WebP · máx. 2 MB cada una
      </p>

      {urls.length > 0 && (
        <ul className="proposal-images-preview">
          {urls.map((url, index) => (
            <li key={`${url}-${index}`}>
              <img src={assetUrl(url)} alt={`Imagen ${index + 1}`} />
              <button type="button" className="btn-link" onClick={() => removeAt(index)}>
                Quitar
              </button>
            </li>
          ))}
        </ul>
      )}

      {canAdd && (
        <div className="proposal-images-actions">
          <input
            ref={inputRef}
            type="file"
            accept={ACCEPT}
            disabled={uploading}
            onChange={(e) => handleFile(e.target.files?.[0])}
          />
          {uploading && <span className="muted">Subiendo…</span>}
        </div>
      )}

      {error && <p className="error">{error}</p>}
    </div>
  );
}
