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
import { commands } from './bindings';
import Container from './components/Container';
import CreateWallet from './pages/CreateWallet';
import ImportWallet from './pages/ImportWallet';
import IssueCat from './pages/IssueCat';
import Login from './pages/Login';
import NetworkList from './pages/NetworkList';
import Nft from './pages/Nft';
import PeerList from './pages/PeerList';
import Receive from './pages/Receive';
import Send from './pages/Send';
import SendCat from './pages/SendCat';
import Settings from './pages/Settings';
import Token from './pages/Token';
import { MainWallet } from './pages/WalletMain';
import { WalletNfts } from './pages/WalletNfts';
import Wallet from './pages/WalletTabs';
import { WalletTokens } from './pages/WalletTokens';

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
      <Route path='/wallet' element={<Wallet />}>
        <Route path='' element={<MainWallet />} />
        <Route path='tokens' element={<WalletTokens />} />
        <Route path='nfts' element={<WalletNfts />} />
        <Route path='token/:asset_id' element={<Token />} />
        <Route path='nft/:launcher_id' element={<Nft />} />
      </Route>
      <Route path='/issue-cat' element={<IssueCat />} />
      <Route path='/send' element={<Send />} />
      <Route path='/send-cat/:asset_id' element={<SendCat />} />
      <Route path='/receive' element={<Receive />} />
      <Route path='/settings' element={<Settings />} />
      <Route path='/networks' element={<NetworkList />} />
      <Route path='/peers' element={<PeerList />} />
    </>,
  ),
);

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

  const theme = useMemo(() => {
    const root = window.document.documentElement;
    root.classList.remove(dark ? 'light' : 'dark');
    root.classList.add(dark ? 'dark' : 'light');

    return createTheme({
      palette: {
        mode: dark ? 'dark' : 'light',
      },
    });
  }, [dark]);

  useEffect(() => {
    commands
      .initialize()
      .then((result) => {
        if (result.status === 'ok') {
          setInitialized(true);
        } else {
          setError(result.error.reason);
        }
      })
      .catch(setError);
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
