import {
  AcUnit,
  Delete,
  Edit,
  LocalFireDepartment,
  Login as LoginIcon,
  MoreVert,
} from '@mui/icons-material';
import {
  Box,
  Button,
  Card,
  CardActionArea,
  CardContent,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  IconButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Skeleton,
  TextField,
  Typography,
} from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  activeWallet,
  deleteWallet,
  loginWallet,
  renameWallet,
  WalletInfo,
  walletList,
} from '../commands';
import Container from '../components/Container';
import NavBar from '../components/NavBar';

export default function Login() {
  const [wallets, setWallets] = useState<WalletInfo[] | null>(null);

  const navigate = useNavigate();

  useEffect(() => {
    walletList().then(setWallets);
  }, []);

  useEffect(() => {
    activeWallet().then((fingerprint) => {
      if (!fingerprint) return;
      navigate('/wallet');
    });
  }, [navigate]);

  return (
    <>
      <NavBar label='Wallet Login' back={null} />
      {wallets ? (
        wallets.length ? (
          <Grid2 container spacing={2} margin={2} justifyContent={'center'}>
            {wallets.map((wallet, i) => (
              <WalletItem
                key={i}
                wallet={wallet}
                wallets={wallets}
                setWallets={setWallets}
              />
            ))}
          </Grid2>
        ) : (
          <Welcome />
        )
      ) : (
        <SkeletonWalletList />
      )}
    </>
  );
}

function SkeletonWalletList() {
  return (
    <Grid2 container spacing={2} margin={2} justifyContent={'center'}>
      {Array.from({ length: 3 }).map((_, i) => (
        <Grid2 key={i} xs={12} sm={6} md={4}>
          <Skeleton variant='rounded' height={100} />
        </Grid2>
      ))}
    </Grid2>
  );
}

function WalletItem(props: {
  wallet: WalletInfo;
  wallets: WalletInfo[];
  setWallets: (wallets: WalletInfo[]) => void;
}) {
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const isMenuOpen = Boolean(anchorEl);

  const [isDeleteOpen, setDeleteOpen] = useState(false);
  const [isRenameOpen, setRenameOpen] = useState(false);
  const [newName, setNewName] = useState('');

  const navigate = useNavigate();

  const openMenu = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
    event.stopPropagation();
    event.preventDefault();
  };

  const closeMenu = () => {
    setAnchorEl(null);
  };

  const deleteSelf = () => {
    deleteWallet(props.wallet.fingerprint).then(() => {
      props.setWallets(
        props.wallets.filter(
          (wallet) => wallet.fingerprint !== props.wallet.fingerprint,
        ),
      );
      setDeleteOpen(false);
    });
  };

  const renameSelf = () => {
    if (!newName) return;

    renameWallet(props.wallet.fingerprint, newName).then(() => {
      props.setWallets(
        props.wallets.map((wallet) =>
          wallet.fingerprint === props.wallet.fingerprint
            ? { ...wallet, name: newName }
            : wallet,
        ),
      );
      setRenameOpen(false);
    });

    setNewName('');
  };

  const loginSelf = (explicit: boolean) => {
    if (isMenuOpen && !explicit) return;

    loginWallet(props.wallet.fingerprint).then(() => {
      navigate('/wallet');
    });
  };

  return (
    <Grid2 xs={12} sm={6} md={4}>
      <Card>
        <CardActionArea onClick={() => loginSelf(false)}>
          <CardContent>
            <Box
              display='flex'
              alignItems='center'
              justifyContent='space-between'
              mt={-0.9}
            >
              <Typography variant='h5' component='div'>
                {props.wallet.name}
              </Typography>
              <IconButton
                sx={{ mr: -0.9 }}
                color='inherit'
                onClick={openMenu}
                onMouseDown={(e) => e.stopPropagation()}
              >
                <MoreVert />
              </IconButton>
            </Box>
            <Menu
              anchorEl={anchorEl}
              open={isMenuOpen}
              onClose={closeMenu}
              onMouseDown={(e) => e.stopPropagation()}
            >
              <MenuItem
                onClick={() => {
                  setRenameOpen(true);
                  closeMenu();
                }}
              >
                <ListItemIcon>
                  <Edit fontSize='small' />
                </ListItemIcon>
                <ListItemText>Rename</ListItemText>
              </MenuItem>
              <MenuItem
                onClick={() => {
                  setDeleteOpen(true);
                  closeMenu();
                }}
              >
                <ListItemIcon>
                  <Delete fontSize='small' />
                </ListItemIcon>
                <ListItemText>Delete</ListItemText>
              </MenuItem>
              <MenuItem
                onClick={() => {
                  loginSelf(true);
                  closeMenu();
                }}
              >
                <ListItemIcon>
                  <LoginIcon fontSize='small' />
                </ListItemIcon>
                <ListItemText>Login</ListItemText>
              </MenuItem>
            </Menu>
            <Box
              display='flex'
              alignItems='center'
              justifyContent='space-between'
              mt={1}
            >
              <Typography color='text.secondary'>
                {props.wallet.fingerprint}
              </Typography>
              {props.wallet.kind == 'hot' ? (
                <Chip
                  icon={<LocalFireDepartment />}
                  label='Hot Wallet'
                  size='small'
                />
              ) : (
                <Chip icon={<AcUnit />} label='Cold Wallet' size='small' />
              )}
            </Box>
          </CardContent>
        </CardActionArea>
      </Card>

      <Dialog open={isDeleteOpen} onClose={() => setDeleteOpen(false)}>
        <DialogTitle>Permanently Delete</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Are you sure you want to delete this wallet? This cannot be undone,
            and all funds will be lost unless you have saved your mnemonic
            phrase.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteOpen(false)}>Cancel</Button>
          <Button onClick={deleteSelf} autoFocus>
            Delete
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog
        open={isRenameOpen}
        onClose={() => {
          setNewName('');
          setRenameOpen(false);
        }}
      >
        <DialogTitle>Rename Wallet</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Enter the new display name for this wallet.
          </DialogContentText>
          <TextField
            label='Wallet Name'
            variant='standard'
            margin='dense'
            required
            fullWidth
            autoFocus
            value={newName}
            onChange={(event) => setNewName(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                renameSelf();
              }
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button
            onClick={() => {
              setNewName('');
              setRenameOpen(false);
            }}
          >
            Cancel
          </Button>
          <Button onClick={renameSelf} autoFocus disabled={!newName}>
            Rename
          </Button>
        </DialogActions>
      </Dialog>
    </Grid2>
  );
}

function Welcome() {
  const navigate = useNavigate();

  return (
    <Container>
      <Typography variant='h3' component='div' textAlign='center'>
        Sage Wallet
      </Typography>

      <Typography variant='body1' component='div' textAlign='center' mt={3}>
        There aren't any wallets to log into yet. To get started, create a new
        wallet or import an existing one.
      </Typography>

      <Button
        variant='contained'
        color='primary'
        fullWidth
        sx={{ mt: 4 }}
        onClick={() => navigate('/create')}
      >
        Create Wallet
      </Button>
      <Button
        variant='outlined'
        color='secondary'
        fullWidth
        sx={{ mt: 2 }}
        onClick={() => navigate('/import')}
      >
        Import Wallet
      </Button>
    </Container>
  );
}
