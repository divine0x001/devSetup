import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import App from '@/App';

// Global stylesheet order matters: reset -> tokens -> base -> effects.
import '@/theme/reset.css';
import '@/theme/tokens.css';
import '@/theme/base.css';
import '@/theme/effects.css';

const container = document.getElementById('root');

if (!container) {
  throw new Error('Prism: root element #root was not found in index.html');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
