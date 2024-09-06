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
import { useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';
import { MainWallet } from '../tabs/Main';
import { WalletNfts } from '../tabs/Nfts';
import { WalletTokens } from '../tabs/Tokens';

export default function Wallet() {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [tab, setTab] = useState(0);

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
      <NavBar label={wallet?.name ?? 'Loading...'} back='logout' />

      <ListContainer>
        <Box pb={11}>
          {tab === 0 ? (
            <MainWallet />
          ) : tab === 1 ? (
            <WalletTokens />
          ) : (
            <WalletNfts />
          )}
        </Box>
      </ListContainer>

      <Paper
        sx={{ position: 'fixed', bottom: 0, left: 0, right: 0 }}
        elevation={3}
      >
        <BottomNavigation
          showLabels
          value={tab}
          onChange={(_event, newValue) => {
            setTab(newValue);
          }}
        >
          <BottomNavigationAction label='Wallet' icon={<WalletIcon />} />
          <BottomNavigationAction label='Tokens' icon={<AccountBalance />} />
          <BottomNavigationAction label='NFTs' icon={<Collections />} />
        </BottomNavigation>
      </Paper>
    </>
  );
}
