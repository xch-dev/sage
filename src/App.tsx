import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { createContext, useEffect, useMemo, useState } from 'react';
import {
  createHashRouter,
  createRoutesFromElements,
  Route,
  RouterProvider,
} from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { ErrorProvider } from './contexts/ErrorContext';
import {
  getBrowserLanguage,
  LanguageProvider,
  SupportedLanguage,
  useLanguage,
} from './contexts/LanguageContext';
import { PeerProvider } from './contexts/PeerContext';
import { PriceProvider } from './contexts/PriceContext';
import { SafeAreaProvider } from './contexts/SafeAreaContext';
import { WalletConnectProvider } from './contexts/WalletConnectContext';
import useInitialization from './hooks/useInitialization';
import { WalletProvider } from './contexts/WalletContext';
import { loadCatalog } from './i18n';
import Addresses from './pages/Addresses';
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
import Transaction from './pages/Transaction';
import { Transactions } from './pages/Transactions';
import { ViewOffer } from './pages/ViewOffer';
import { ViewSavedOffer } from './pages/ViewSavedOffer';
import Wallet from './pages/Wallet';
import QRScanner from './pages/QrScanner';
import { ToastContainer, Slide } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import CollectionMetaData from './pages/CollectionMetaData';

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
        <Route path='collections/:collection_id' element={<NftList />} />
        <Route
          path='collections/:collection_id/metadata'
          element={<CollectionMetaData />}
        />
        <Route path='owners/:owner_did' element={<NftList />} />
        <Route path='minters/:minter_did' element={<NftList />} />
        <Route path='mint' element={<MintNft />} />
      </Route>
      <Route path='/dids' element={<Wallet />}>
        <Route path='' element={<DidList />} />
        <Route path='create' element={<CreateProfile />} />
      </Route>
      <Route path='/transactions' element={<Wallet />}>
        <Route path='' element={<Transactions />} />
        <Route path=':height' element={<Transaction />} />
      </Route>
      <Route path='/offers' element={<Wallet />}>
        <Route path='' element={<Offers />} />
        <Route path='make' element={<MakeOffer />} />
        <Route path='view/:offer' element={<ViewOffer />} />
        <Route path='view_saved/:offer_id' element={<ViewSavedOffer />} />
      </Route>
      <Route path='/settings' element={<Settings />} />
      <Route path='/scan' element={<QRScanner />} />
      <Route path='/peers' element={<PeerList />} />
    </>,
  ),
);

export default function App() {
  const [dark, setDark] = useLocalStorage('dark', false);
  const [locale, setLocale] = useLocalStorage<SupportedLanguage>(
    'locale',
    getBrowserLanguage,
  );

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
    <LanguageProvider locale={locale} setLocale={setLocale}>
      <DarkModeContext.Provider value={darkMode}>
        <SafeAreaProvider>
          <ErrorProvider>
            <AppInner />
          </ErrorProvider>
        </SafeAreaProvider>
      </DarkModeContext.Provider>
      <ToastContainer
        position='bottom-right'
        autoClose={5000}
        hideProgressBar={false}
        newestOnTop={false}
        closeOnClick
        rtl={false}
        pauseOnFocusLoss
        draggable
        pauseOnHover
        theme={dark ? 'dark' : 'light'}
        transition={Slide}
        style={
          {
            '--toastify-toast-transition-timing': 'ease',
            '--toastify-toast-transition-duration': '750ms',
          } as React.CSSProperties
        }
      />
    </LanguageProvider>
  );
}

function AppInner() {
  const initialized = useInitialization();
  const { locale } = useLanguage();
  const [isLocaleInitialized, setIsLocaleInitialized] = useState(false);

  useEffect(() => {
    const initLocale = async () => {
      await loadCatalog(locale);
      setIsLocaleInitialized(true);
    };
    initLocale();
  }, [locale]);

  return (
    initialized &&
    isLocaleInitialized && (
      <I18nProvider i18n={i18n}>
        <WalletProvider>
          <PeerProvider>
            <WalletConnectProvider>
              <PriceProvider>
                <RouterProvider router={router} />
              </PriceProvider>
            </WalletConnectProvider>
          </PeerProvider>
        </WalletProvider>
      </I18nProvider>
    )
  );
}
