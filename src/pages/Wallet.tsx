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
  useMediaQuery,
  useTheme,
} from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { GridRowSelectionModel } from '@mui/x-data-grid';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  CatRecord,
  commands,
  events,
  NftRecord,
  WalletInfo,
} from '../bindings';
import CoinList from '../components/CoinList';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';
import { useWalletState } from '../state';

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
          {walletState.sync.balance} {walletState.sync.unit.ticker}
        </Typography>

        <LinearProgress
          variant='determinate'
          value={Math.ceil(
            (walletState.sync.synced_coins / walletState.sync.total_coins) *
              100,
          )}
          sx={{ mt: 2 }}
        />

        <Box mt={1} textAlign='center'>
          {walletState.sync.synced_coins}
          {walletState.sync.synced_coins === walletState.sync.total_coins
            ? ''
            : `/${walletState.sync.total_coins}`}{' '}
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
  const [cats, setCats] = useState<CatRecord[]>([]);

  const updateCats = () => {
    commands.getCats().then((result) => {
      if (result.status === 'ok') {
        setCats(result.data);
      }
    });
  };

  useEffect(() => {
    updateCats();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'cat_update') {
        updateCats();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  return (
    <Box display='flex' flexDirection='column' gap={2}>
      {cats
        .sort((a, b) => {
          if (a.name && b.name) {
            return a.name.localeCompare(b.name);
          } else if (a.name) {
            return -1;
          } else {
            return 1;
          }
        })
        .map((cat, i) => (
          <TokenListItem key={i} cat={cat} />
        ))}
    </Box>
  );
}

function TokenListItem(props: { cat: CatRecord }) {
  const theme = useTheme();
  const md = useMediaQuery(theme.breakpoints.up('md'));
  const sm = useMediaQuery(theme.breakpoints.up('sm'));

  const chars = sm ? 16 : 8;

  return (
    <Paper>
      <ListItemButton>
        <ListItemAvatar>
          {props.cat.icon_url && (
            <Avatar
              alt={props.cat.ticker ?? undefined}
              src={props.cat.icon_url}
            />
          )}
        </ListItemAvatar>
        <ListItemText
          primary={props.cat.name ?? 'Unknown CAT'}
          secondary={
            <Typography
              sx={{ display: 'inline' }}
              component='span'
              variant='body2'
              color='text.primary'
            >
              {md
                ? props.cat.asset_id
                : `${props.cat.asset_id.slice(0, chars)}..${props.cat.asset_id.slice(-chars)}`}
            </Typography>
          }
        />
      </ListItemButton>
    </Paper>
  );
}

function NftList() {
  const [page, setPage] = useState(0);
  const [totalPages, setTotalPages] = useState(1);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const updateNfts = async (page: number) => {
    return await commands
      .getNfts({ offset: page * 60, limit: 60 })
      .then((result) => {
        if (result.status === 'ok') {
          setNfts(result.data.items);
          setTotalPages(Math.max(1, Math.ceil(result.data.total / 60)));
        } else {
          throw new Error('Failed to get NFTs');
        }
      });
  };

  const nextPage = () => {
    if (loading) return;
    setLoading(true);
    updateNfts(page + 1)
      .then(() => setPage(page + 1))
      .finally(() => {
        setLoading(false);
      });
  };

  const previousPage = () => {
    if (loading) return;
    setLoading(true);
    updateNfts(page - 1)
      .then(() => setPage(page - 1))
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    updateNfts(page);

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'nft_update') {
        updateNfts(page);
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [page]);

  return (
    <>
      <Box display='flex' justifyContent='center' alignItems='center' gap={2}>
        <Button
          variant='outlined'
          onClick={() => previousPage()}
          disabled={page === 0}
        >
          Previous
        </Button>
        <Typography variant='body1'>
          Page {page + 1} of {totalPages}
        </Typography>
        <Button
          variant='outlined'
          onClick={() => nextPage()}
          disabled={page >= totalPages - 1}
        >
          Next
        </Button>
      </Box>
      <Grid2 mt={3} container spacing={2}>
        {nfts.map((nft, i) => (
          <Nft nft={nft} key={i} />
        ))}
      </Grid2>
    </>
  );
}

function Nft({ nft }: { nft: NftRecord }) {
  const theme = useTheme();

  if (!nft.metadata_json) console.log(nft);

  let json: any = {};

  if (nft.metadata_json) {
    try {
      json = JSON.parse(nft.metadata_json);
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <Grid2 xs={6} sm={4} md={4}>
      <Box position='relative' width='100%' height='100%'>
        <Button sx={{ padding: 0, width: '100%', height: '100%' }}>
          <img
            src={nft.data_uris[0]}
            style={{
              width: '100%',
              height: '100%',
              borderRadius: theme.shape.borderRadius,
            }}
          />

          <Paper
            sx={{
              position: 'absolute',
              bottom: 0,
              width: '100%',
              height: '40px',
              p: 1,
              borderTopLeftRadius: '0px',
              borderTopRightRadius: '0px',
              borderBottomLeftRadius: theme.shape.borderRadius,
              borderBottomRightRadius: theme.shape.borderRadius,
              textAlign: 'center',
              textTransform: 'none',
              overflow: 'none',
            }}
          >
            <Typography variant='body1' width='100%' height='100%'>
              {json.name ?? 'Unknown NFT'}
            </Typography>
          </Paper>
        </Button>
      </Box>
    </Grid2>
  );
}
