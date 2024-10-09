import { useEffect, useState } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';

import { logoutAndUpdateState } from '@/state';
import Layout from '@/components/Layout';

export default function Wallet() {
  const navigate = useNavigate();
  const location = useLocation();

  let tab = null;

  if (location.pathname === '/wallet') {
    tab = 0;
  } else if (location.pathname === '/wallet/tokens') {
    tab = 1;
  } else if (location.pathname === '/wallet/nfts') {
    tab = 2;
  }

  const [wallet, setWallet] = useState<WalletInfo | null>(null);

  useEffect(() => {
    commands.activeWallet().then((wallet) => {
      if (wallet.status === 'error') {
        return;
      }
      if (wallet.data) return setWallet(wallet.data);
      navigate('/');
    });
  }, [navigate]);

  const logout = () => {
    logoutAndUpdateState().then(() => {
      navigate('/');
    });
  };

  return (
    <>
      <Layout>
        <Outlet />
      </Layout>
    </>
  );
}
