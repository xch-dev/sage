import {
  AccountBalance,
  Collections,
  Wallet as WalletIcon,
} from '@mui/icons-material';
import {
  Avatar,
  BottomNavigation,
  BottomNavigationAction,
  Box,
  ListItemAvatar,
  ListItemButton,
  ListItemText,
  Paper,
  Typography,
} from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import * as commands from '../commands';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';
import { NftData, SyncInfo, WalletInfo } from '../models';

export default function Wallet() {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [syncInfo, setSyncInfo] = useState<SyncInfo>({
    xch_balance: 'Syncing',
    total_coins: 0,
    synced_coins: 0,
  });
  const [tab, setTab] = useState(0);

  useEffect(() => {
    commands.activeWallet().then((wallet) => {
      if (wallet) return setWallet(wallet);
      navigate('/');
    });
  }, [navigate]);

  useEffect(() => {
    commands.syncInfo().then(setSyncInfo);

    const interval = setInterval(() => {
      commands.syncInfo().then(setSyncInfo);
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <>
      <NavBar label={wallet?.name ?? 'Loading...'} back='logout' />

      <ListContainer>
        {tab === 0 ? (
          <Grid2
            container
            spacing={2}
            alignItems='center'
            justifyContent='center'
          >
            <Grid2 xs={12} md={6}>
              <Paper sx={{ p: 1.5, px: 2 }}>
                <Typography
                  fontSize={20}
                  fontWeight='normal'
                  color='text.secondary'
                >
                  Balance
                </Typography>
                <Typography fontSize={22}>{syncInfo.xch_balance}</Typography>
              </Paper>
            </Grid2>
            <Grid2 xs={12} md={6}>
              <Paper sx={{ p: 1.5, px: 2 }}>
                <Typography
                  fontSize={20}
                  fontWeight='normal'
                  color='text.secondary'
                >
                  Sync Status
                </Typography>
                <Typography fontSize={22}>
                  {syncInfo.synced_coins}/{syncInfo.total_coins} Coins
                </Typography>
              </Paper>
            </Grid2>
          </Grid2>
        ) : tab === 1 ? (
          <TokenList />
        ) : (
          <NftList />
        )}
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

function TokenList() {
  return (
    <Box display='flex' flexDirection='column' gap={2}>
      <TokenListItem
        balance='416.17'
        ticker='SBX'
        name='Spacebux'
        icon='https://icons.dexie.space/a628c1c2c6fcb74d53746157e438e108eab5c0bb3e5c80ff9b1910b3e4832913.webp'
      />
      <TokenListItem
        balance='23.2'
        ticker='MWIF'
        name='Marmot Wif Hat'
        icon='https://icons.dexie.space/e233f9c0ebc092f083aaacf6295402ed0a0bb1f9acb1b56500d8a4f5a5e4c957.webp'
      />
    </Box>
  );
}

function TokenListItem(props: {
  icon: string;
  ticker: string;
  balance: string;
  name: string;
}) {
  return (
    <Paper>
      <ListItemButton>
        <ListItemAvatar>
          <Avatar alt={props.ticker} src={props.icon} />
        </ListItemAvatar>
        <ListItemText
          primary={props.name}
          secondary={
            <Typography
              sx={{ display: 'inline' }}
              component='span'
              variant='body2'
              color='text.primary'
            >
              {props.balance} {props.ticker}
            </Typography>
          }
        />
      </ListItemButton>
    </Paper>
  );
}

function NftList() {
  const [nftList, setNftList] = useState<NftData[]>([]);

  useEffect(() => {
    commands.nftList().then(setNftList);

    const interval = setInterval(() => {
      commands.nftList().then(setNftList);
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  return (
    <>
      {nftList.map((nft) => (
        <Paper key={nft.launcher_id}>
          <ListItemButton>
            <ListItemAvatar>
              <Avatar />
            </ListItemAvatar>
            <ListItemText primary={nft.address} secondary={nft.launcher_id} />
          </ListItemButton>
        </Paper>
      ))}
    </>
  );
}
