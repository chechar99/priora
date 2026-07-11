import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const API_ORIGIN = process.env.PRIORA_API_ORIGIN || 'http://127.0.0.1:3100';

export default defineConfig({
  plugins: [react()],
  server: {
    host: '127.0.0.1',
    port: 5190,
    strictPort: true,
    proxy: {
      '/api': API_ORIGIN,
      '/uploads': API_ORIGIN,
    },
  },
});
