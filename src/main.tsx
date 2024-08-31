import { createRoot } from 'react-dom/client';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

import App from './App.tsx';

import { appDataDir } from '@tauri-apps/api/path';
import './index.css';

const element = document.getElementById('root')!;

createRoot(element).render(<App />);

appDataDir().then(console.log.bind(console.log));
