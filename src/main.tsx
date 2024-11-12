import BigNumber from 'bignumber.js';

import { createRoot } from 'react-dom/client';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

import App from './App.tsx';

BigNumber.config({ EXPONENTIAL_AT: 1e9 });

const element = document.getElementById('root')!;

createRoot(element).render(<App />);
