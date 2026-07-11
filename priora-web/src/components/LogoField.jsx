import { useRef, useState } from 'react';
import { api, assetUrl } from '../api/client';

const ACCEPT = 'image/jpeg,image/png,image/webp';

export default function LogoField({ value, onChange }) {
  const inputRef = useRef(null);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState('');
  const preview = assetUrl(value);

  async function handleFile(file) {
    if (!file) return;
    setError('');
    setUploading(true);
    try {
      const result = await api.uploadLogo(file);
      onChange(result.url);
    } catch (e) {
      setError(e.message || 'No se pudo subir la imagen');
    } finally {
      setUploading(false);
      if (inputRef.current) inputRef.current.value = '';
    }
  }

  return (
    <div className="logo-field">
      <span className="logo-field-label">Logo (opcional)</span>
      <p className="logo-field-hint">JPEG, PNG o WebP · máx. 2 MB</p>

      {preview && (
        <div className="logo-preview">
          <img src={preview} alt="Vista previa del logo" />
          <button
            type="button"
            className="btn-link"
            onClick={() => onChange('')}
          >
            Quitar
          </button>
        </div>
      )}

      <div className="logo-field-actions">
        <input
          ref={inputRef}
          type="file"
          accept={ACCEPT}
          disabled={uploading}
          onChange={(e) => handleFile(e.target.files?.[0])}
        />
        {uploading && <span className="muted">Subiendo…</span>}
      </div>

      <label className="logo-url-fallback">
        O pegá una URL
        <input
          type="url"
          placeholder="https://…"
          value={value?.startsWith('http') ? value : ''}
          onChange={(e) => onChange(e.target.value)}
        />
      </label>

      {error && <p className="error">{error}</p>}
    </div>
  );
}
