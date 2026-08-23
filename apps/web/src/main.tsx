import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './api';
import { App } from './App';
import './index.css';

const root = document.getElementById('root');

if (!root) {
  throw new Error('Root element #root was not found');
}

// OAuth return can restore a frozen pre-login document from bfcache (blank until refresh).
window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    window.location.reload();
  }
});

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
