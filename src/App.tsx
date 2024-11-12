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
import { PeerProvider } from './contexts/PeerContext';
import Collection from './pages/Collection';
import CreateProfile from './pages/CreateProfile';
import CreateWallet from './pages/CreateWallet';
import { DidList } from './pages/DidList';
import ImportWallet from './pages/ImportWallet';
import IssueToken from './pages/IssueToken';
import Login from './pages/Login';
import { MakeOffer } from './pages/MakeOffer';
import MintNft from './pages/MintNft';
import Nft from './pages/Nft';
import { NftList } from './pages/NftList';
import { Offers } from './pages/Offers';
import PeerList from './pages/PeerList';
import Receive from './pages/Receive';
import Send from './pages/Send';
import Settings from './pages/Settings';
import Token from './pages/Token';
import { TokenList } from './pages/TokenList';
import { Transactions } from './pages/Transactions';
import { ViewOffer } from './pages/ViewOffer';
import Wallet from './pages/Wallet';
import { fetchState } from './state';
import { PriceProvider } from './contexts/PriceContext';

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
        <Route path='' element={<TokenList />} />
        <Route path='token/:asset_id' element={<Token />} />
        <Route path='issue-token' element={<IssueToken />} />
        <Route path='send/:asset_id' element={<Send />} />
        <Route path='receive' element={<Receive />} />
      </Route>
      <Route path='/nfts' element={<Wallet />}>
        <Route path='' element={<NftList />} />
        <Route path=':launcher_id' element={<Nft />} />
        <Route path='mint-nft' element={<MintNft />} />
      </Route>
      <Route path='/collections' element={<Wallet />}>
        <Route path=':collection_id' element={<Collection />} />
      </Route>
      <Route path='/dids' element={<Wallet />}>
        <Route path='' element={<DidList />} />
        <Route path='create-profile' element={<CreateProfile />} />
      </Route>
      <Route path='/transactions' element={<Wallet />}>
        <Route path='' element={<Transactions />} />
      </Route>
      <Route path='/offers' element={<Wallet />}>
        <Route path='' element={<Offers />} />
        <Route path='make' element={<MakeOffer />} />
        <Route path='view/:offer' element={<ViewOffer />} />
      </Route>
      <Route path='/settings' element={<Settings />} />
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

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove(dark ? 'light' : 'dark');
    root.classList.add(dark ? 'dark' : 'light');
  }, [dark]);

  useEffect(() => {
    commands
      .initialize()
      .then((result) => {
        if (result.status === 'ok') {
          setInitialized(true);

          commands.activeWallet().then((wallet) => {
            if (wallet.status === 'ok' && wallet.data !== null) {
              fetchState();
            }
          });
        } else {
          console.error(result.error);
          setError(result.error.reason);
        }
      })
      .catch(setError);
  }, []);

  return (
    <DarkModeContext.Provider value={darkMode}>
      {initialized && (
        <PeerProvider>
          <PriceProvider>
            <RouterProvider router={router} />
          </PriceProvider>
        </PeerProvider>
      )}
      {error && (
        <Container>
          <h2 className='text-2xl font-bold mb-2'>Error</h2>
          <p className='text-red-500 mt-2'>{error}</p>
        </Container>
      )}
    </DarkModeContext.Provider>
  );
}
