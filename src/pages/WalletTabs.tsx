import {
  AccountBalance,
  Collections,
  Wallet as WalletIcon,
} from '@mui/icons-material';
import {
  BottomNavigation,
  BottomNavigationAction,
  Box,
  Paper,
} from '@mui/material';
import { useEffect, useState } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';

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

  return (
    <>
      <NavBar
        label={wallet?.name ?? 'Loading...'}
        back={tab === null ? () => navigate(-1) : 'logout'}
      />

      <ListContainer>
        <Box pb={11}>
          <Outlet />
        </Box>
      </ListContainer>

      <Paper
        sx={{ position: 'fixed', bottom: 0, left: 0, right: 0 }}
        elevation={3}
      >
        <BottomNavigation showLabels value={tab}>
          <BottomNavigationAction
            label='Wallet'
            icon={<WalletIcon />}
            onClick={() => navigate('')}
          />
          <BottomNavigationAction
            label='Tokens'
            icon={<AccountBalance />}
            onClick={() => navigate('tokens')}
          />
          <BottomNavigationAction
            label='NFTs'
            icon={<Collections />}
            onClick={() => navigate('nfts')}
          />
        </BottomNavigation>
      </Paper>
    </>
  );
}
