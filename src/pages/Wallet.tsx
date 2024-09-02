import {
  AccountBalance,
  ArrowForward,
  Collections,
  CompareArrows,
  ForkRight,
  MoreVert,
  Wallet as WalletIcon,
} from '@mui/icons-material';
import {
  Avatar,
  BottomNavigation,
  BottomNavigationAction,
  Box,
  Button,
  IconButton,
  LinearProgress,
  ListItemAvatar,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Paper,
  Typography,
} from '@mui/material';
import { GridRowSelectionModel } from '@mui/x-data-grid';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import * as commands from '../commands';
import CoinList from '../components/CoinList';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';
import { NftData, WalletInfo } from '../models';
import { useWalletState } from '../state';

export default function Wallet() {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [tab, setTab] = useState(0);

  useEffect(() => {
    commands.activeWallet().then((wallet) => {
      if (wallet) return setWallet(wallet);
      navigate('/');
    });
  }, [navigate]);

  return (
    <>
      <NavBar label={wallet?.name ?? 'Loading...'} back='logout' />

      <ListContainer>
        <Box pb={11}>
          {tab === 0 ? <MainWallet /> : tab === 1 ? <TokenList /> : <NftList />}
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

function MainWallet() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const [selectedCoins, setSelectedCoins] = useState<GridRowSelectionModel>([]);

  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };

  return (
    <>
      <Box mt={1}>
        <Typography variant='h5' fontSize={30} textAlign='center'>
          {walletState.syncInfo.xch_balance} XCH
        </Typography>

        <LinearProgress
          variant='determinate'
          value={Math.ceil(
            (walletState.syncInfo.synced_coins /
              walletState.syncInfo.total_coins) *
              100,
          )}
          sx={{ mt: 2 }}
        />

        <Box mt={1} textAlign='center'>
          {walletState.syncInfo.synced_coins}
          {walletState.syncInfo.synced_coins ===
          walletState.syncInfo.total_coins
            ? ''
            : `/${walletState.syncInfo.total_coins}`}{' '}
          coins synced
        </Box>

        <Box display='flex' gap={2} mt={2}>
          <Button
            variant='outlined'
            size='large'
            sx={{ flexGrow: 1 }}
            onClick={() => navigate('/send')}
          >
            Send
          </Button>
          <Button
            variant='outlined'
            size='large'
            sx={{ flexGrow: 1 }}
            onClick={() => navigate('/receive')}
          >
            Receive
          </Button>
        </Box>

        <Box height={350} position='relative' mt={2}>
          <CoinList
            coins={walletState.coins}
            selectedCoins={selectedCoins}
            setSelectedCoins={setSelectedCoins}
          />

          <Box
            position='absolute'
            top={9}
            right={5}
            display={selectedCoins.length === 0 ? 'none' : 'block'}
          >
            <IconButton onClick={handleClick}>
              <MoreVert />
            </IconButton>

            <Menu
              anchorEl={anchorEl}
              open={open}
              onClose={handleClose}
              MenuListProps={{
                'aria-labelledby': 'basic-button',
              }}
            >
              <MenuItem disabled={selectedCoins.length < 2}>
                <ListItemIcon>
                  <CompareArrows fontSize='small' />
                </ListItemIcon>
                <ListItemText>Combine</ListItemText>
              </MenuItem>

              <MenuItem>
                <ListItemIcon>
                  <ForkRight fontSize='small' />
                </ListItemIcon>
                <ListItemText>Split</ListItemText>
              </MenuItem>

              <MenuItem>
                <ListItemIcon>
                  <ArrowForward fontSize='small' />
                </ListItemIcon>
                <ListItemText>Transfer</ListItemText>
              </MenuItem>
            </Menu>
          </Box>
        </Box>
      </Box>
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
