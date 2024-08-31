import {
  createTheme,
  CssBaseline,
  ThemeProvider,
  Typography,
} from '@mui/material';
import { createContext, useEffect, useMemo, useState } from 'react';
import {
  createHashRouter,
  createRoutesFromElements,
  Route,
  RouterProvider,
} from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { initialize } from './commands';
import Container from './components/Container';
import CreateWallet from './pages/CreateWallet';
import ImportWallet from './pages/ImportWallet';
import Login from './pages/Login';
import NetworkList from './pages/NetworkList';
import PeerList from './pages/PeerList';
import Settings from './pages/Settings';
import Wallet from './pages/Wallet';

export interface DarkModeContext {
  toggle: () => void;
  dark: boolean;
  setDark: (dark: boolean) => void;
}

export const DarkModeContext = createContext<DarkModeContext>({
  toggle: () => {},
  dark: false,
  setDark: () => {},
});

const router = createHashRouter(
  createRoutesFromElements(
    <>
      <Route path='/' element={<Login />} />
      <Route path='/create' element={<CreateWallet />} />
      <Route path='/import' element={<ImportWallet />} />
      <Route path='/wallet' element={<Wallet />} />
      <Route path='/settings' element={<Settings />} />
      <Route path='/networks' element={<NetworkList />} />
      <Route path='/peers' element={<PeerList />} />
    </>,
  ),
);

let initializedGlobal = false;

export default function App() {
  const [dark, setDark] = useLocalStorage('dark', false);

  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const darkMode = useMemo<DarkModeContext>(
    () => ({
      toggle: () => setDark((dark) => !dark),
      dark,
      setDark,
    }),
    [dark, setDark],
  );

  const theme = useMemo(
    () =>
      createTheme({
        palette: {
          mode: dark ? 'dark' : 'light',
        },
      }),
    [dark],
  );

  useEffect(() => {
    if (initializedGlobal) return;

    initialize()
      .then(() => setInitialized(true))
      .catch(setError);

    initializedGlobal = true;
  }, []);

  return (
    <DarkModeContext.Provider value={darkMode}>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        {initialized && <RouterProvider router={router} />}
        {error && (
          <Container>
            <Typography variant='h4'>Error</Typography>
            <Typography color='error' mt={2}>
              {error}
            </Typography>
          </Container>
        )}
      </ThemeProvider>
    </DarkModeContext.Provider>
  );
}
