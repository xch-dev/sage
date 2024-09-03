import {
  Add,
  ArrowBackIos,
  Collections,
  Contacts,
  Image as ImageIcon,
  Logout,
  NetworkCheck,
  Paid,
  PersonAdd,
  Settings,
} from '@mui/icons-material';
import MenuIcon from '@mui/icons-material/Menu';
import {
  AppBar,
  Box,
  Divider,
  IconButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Toolbar,
  Typography,
} from '@mui/material';
import { ReactElement, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';
import { logoutAndUpdateState } from '../state';

export interface NavBarProps {
  label: string;
  back: 'logout' | (() => void) | null;
}

export default function NavBar(props: NavBarProps) {
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const isMenuOpen = Boolean(anchorEl);
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);

  useEffect(() => {
    commands.activeWallet().then((res) => {
      if (res.status === 'error') return;
      setWallet(res.data);
    });
  }, []);

  const openMenu = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const closeMenu = () => {
    setAnchorEl(null);
  };

  const logout = () => {
    logoutAndUpdateState().then(() => {
      navigate('/');
    });
  };

  const items: Array<ReactElement> = [];

  if (wallet) {
    items.push(
      <MenuItem key='issue_cat'>
        <ListItemIcon>
          <Paid fontSize='small' />
        </ListItemIcon>
        <ListItemText>Issue CAT</ListItemText>
      </MenuItem>,

      <MenuItem key='create_did'>
        <ListItemIcon>
          <Contacts fontSize='small' />
        </ListItemIcon>
        <ListItemText>Create DID</ListItemText>
      </MenuItem>,

      <MenuItem key='mint_nft'>
        <ListItemIcon>
          <ImageIcon fontSize='small' />
        </ListItemIcon>
        <ListItemText>Mint NFT</ListItemText>
      </MenuItem>,

      <MenuItem key='bulk_mint_nfts'>
        <ListItemIcon>
          <Collections fontSize='small' />
        </ListItemIcon>
        <ListItemText>Bulk Mint NFTs</ListItemText>
      </MenuItem>,

      <Divider key='end_wallet' sx={{ my: 0.5 }} />,
    );
  } else {
    items.push(
      <MenuItem
        key='create_wallet'
        onClick={() => {
          navigate('/create');
          closeMenu();
        }}
      >
        <ListItemIcon>
          <Add fontSize='small' />
        </ListItemIcon>
        <ListItemText>Create Wallet</ListItemText>
      </MenuItem>,

      <MenuItem
        key='import_wallet'
        onClick={() => {
          navigate('/import');
          closeMenu();
        }}
      >
        <ListItemIcon>
          <PersonAdd fontSize='small' />
        </ListItemIcon>
        <ListItemText>Import Wallet</ListItemText>
      </MenuItem>,

      <Divider key='end_import' sx={{ my: 0.5 }} />,
    );
  }

  items.push(
    <MenuItem
      key='peers'
      onClick={() => {
        navigate('/peers');
        closeMenu();
      }}
    >
      <ListItemIcon>
        <NetworkCheck fontSize='small' />
      </ListItemIcon>
      <ListItemText>Peers</ListItemText>
    </MenuItem>,

    <MenuItem
      key='settings'
      onClick={() => {
        navigate('/settings');
        closeMenu();
      }}
    >
      <ListItemIcon>
        <Settings fontSize='small' />
      </ListItemIcon>
      <ListItemText>Settings</ListItemText>
    </MenuItem>,
  );

  if (wallet) {
    items.push(
      <MenuItem
        key='logout'
        onClick={() => {
          logout();
          closeMenu();
        }}
      >
        <ListItemIcon>
          <Logout fontSize='small' />
        </ListItemIcon>
        <ListItemText>Logout</ListItemText>
      </MenuItem>,
    );
  }

  return (
    <Box sx={{ flexGrow: 1 }}>
      <AppBar position='fixed'>
        <Toolbar>
          {props.back &&
            (props.back === 'logout' ? (
              <IconButton
                size='large'
                edge='start'
                color='inherit'
                sx={{ mr: 2 }}
                onClick={logout}
              >
                <Logout />
              </IconButton>
            ) : (
              <IconButton
                size='large'
                edge='start'
                color='inherit'
                sx={{ mr: 2 }}
                onClick={props.back}
              >
                <ArrowBackIos />
              </IconButton>
            ))}
          <Typography
            variant='h5'
            component='div'
            sx={{
              flexGrow: 1,
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
            }}
          >
            {props.label}
          </Typography>
          <IconButton
            size='large'
            edge='end'
            color='inherit'
            sx={{ ml: 2 }}
            onClick={openMenu}
          >
            <MenuIcon />
          </IconButton>
          <Menu anchorEl={anchorEl} open={isMenuOpen} onClose={closeMenu}>
            {items}
          </Menu>
        </Toolbar>
      </AppBar>

      <Toolbar />
    </Box>
  );
}
