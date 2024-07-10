import {
  Add,
  ArrowBackIos,
  Logout,
  PersonAdd,
  Settings,
} from '@mui/icons-material';
import MenuIcon from '@mui/icons-material/Menu';
import {
  AppBar,
  Box,
  IconButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Toolbar,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { activeWallet, logoutWallet } from '../commands';
import { WalletInfo } from '../models';

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
    activeWallet().then(setWallet);
  }, []);

  const openMenu = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const closeMenu = () => {
    setAnchorEl(null);
  };

  const logout = () => {
    logoutWallet().then(() => {
      navigate('/');
    });
  };

  return (
    <Box sx={{ flexGrow: 1 }}>
      <AppBar position='static'>
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
            <MenuItem
              onClick={() => {
                navigate('/settings');
                closeMenu();
              }}
            >
              <ListItemIcon>
                <Settings fontSize='small' />
              </ListItemIcon>
              <ListItemText>Settings</ListItemText>
            </MenuItem>
            <MenuItem
              onClick={() => {
                navigate('/create');
                closeMenu();
              }}
            >
              <ListItemIcon>
                <Add fontSize='small' />
              </ListItemIcon>
              <ListItemText>Create Wallet</ListItemText>
            </MenuItem>
            <MenuItem
              onClick={() => {
                navigate('/import');
                closeMenu();
              }}
            >
              <ListItemIcon>
                <PersonAdd fontSize='small' />
              </ListItemIcon>
              <ListItemText>Import Wallet</ListItemText>
            </MenuItem>
            {wallet && (
              <MenuItem
                onClick={() => {
                  logout();
                  closeMenu();
                }}
              >
                <ListItemIcon>
                  <Logout fontSize='small' />
                </ListItemIcon>
                <ListItemText>Logout</ListItemText>
              </MenuItem>
            )}
          </Menu>
        </Toolbar>
      </AppBar>
    </Box>
  );
}
