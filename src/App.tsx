import { createContext, useEffect, useMemo } from 'react';
import {
  createHashRouter,
  createRoutesFromElements,
  Route,
  RouterProvider,
} from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { ErrorProvider } from './contexts/ErrorContext';
import { PeerProvider } from './contexts/PeerContext';
import { PriceProvider } from './contexts/PriceContext';
import { WalletConnectProvider } from './contexts/WalletConnectContext';
import useInitialization from './hooks/useInitialization';
import { useWallet } from './hooks/useWallet';
import Addresses from './pages/Addresses';
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
import Send from './pages/Send';
import Settings from './pages/Settings';
import Token from './pages/Token';
import { TokenList } from './pages/TokenList';
import { Transactions } from './pages/Transactions';
import { ViewOffer } from './pages/ViewOffer';
import { ViewSavedOffer } from './pages/ViewSavedOffer';
import Wallet from './pages/Wallet';
import { fetchState } from './state';
import { getInsets } from 'tauri-plugin-safe-area-insets';
import { SafeAreaProvider } from './contexts/SafeAreaContext';

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
        <Route path='addresses' element={<Addresses />} />
      </Route>
      <Route path='/nfts' element={<Wallet />}>
        <Route path='' element={<NftList />} />
        <Route path=':launcher_id' element={<Nft />} />
        <Route path='mint' element={<MintNft />} />
      </Route>
      <Route path='/collections' element={<Wallet />}>
        <Route path=':collection_id' element={<Collection />} />
      </Route>
      <Route path='/dids' element={<Wallet />}>
        <Route path='' element={<DidList />} />
        <Route path='create' element={<CreateProfile />} />
      </Route>
      <Route path='/transactions' element={<Wallet />}>
        <Route path='' element={<Transactions />} />
      </Route>
      <Route path='/offers' element={<Wallet />}>
        <Route path='' element={<Offers />} />
        <Route path='make' element={<MakeOffer />} />
        <Route path='view/:offer' element={<ViewOffer />} />
        <Route path='view_saved/:offer_id' element={<ViewSavedOffer />} />
      </Route>
      <Route path='/settings' element={<Settings />} />
      <Route path='/peers' element={<PeerList />} />
    </>,
  ),
);

export default function App() {
  const [dark, setDark] = useLocalStorage('dark', false);

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

  return (
    <DarkModeContext.Provider value={darkMode}>
      <SafeAreaProvider>
        <ErrorProvider>
          <AppInner />
        </ErrorProvider>
      </SafeAreaProvider>
    </DarkModeContext.Provider>
  );
}

function AppInner() {
  const initialized = useInitialization();
  const wallet = useWallet(initialized);

  useEffect(() => {
    if (wallet !== null) {
      fetchState();
    }
  }, [wallet]);

  return (
    initialized && (
      <PeerProvider>
        <WalletConnectProvider>
          <PriceProvider>
            <RouterProvider router={router} />
          </PriceProvider>
        </WalletConnectProvider>
      </PeerProvider>
    )
  );
}
