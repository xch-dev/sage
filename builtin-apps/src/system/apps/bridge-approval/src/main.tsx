import React, { Suspense } from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Suspense fallback={<div className='p-4 text-sm'>Loading approval...</div>}>
      <App />
    </Suspense>
  </React.StrictMode>,
);
