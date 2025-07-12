import { createRoot } from 'react-dom/client';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

import './setup.ts';

import App from './App.tsx';

const element = document.getElementById('root') as HTMLElement;

createRoot(element).render(<App />);
