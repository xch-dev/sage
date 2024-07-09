import { ArrowBackIos } from '@mui/icons-material';
import MenuIcon from '@mui/icons-material/Menu';
import {
  AppBar,
  Box,
  IconButton,
  Menu,
  MenuItem,
  Toolbar,
  Typography,
} from '@mui/material';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

export interface NavBarProps {
  label: string;
  back: (() => void) | null;
}

export default function NavBar(props: NavBarProps) {
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const isMenuOpen = Boolean(anchorEl);
  const navigate = useNavigate();

  const openMenu = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const closeMenu = () => {
    setAnchorEl(null);
  };

  return (
    <Box sx={{ flexGrow: 1 }}>
      <AppBar position='static'>
        <Toolbar>
          {props.back && (
            <IconButton
              size='large'
              edge='start'
              color='inherit'
              sx={{ mr: 2 }}
              onClick={props.back}
            >
              <ArrowBackIos />
            </IconButton>
          )}
          <Typography variant='h5' component='div' sx={{ flexGrow: 1 }}>
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
                navigate('/create');
                closeMenu();
              }}
            >
              Create Wallet
            </MenuItem>
            <MenuItem
              onClick={() => {
                navigate('/import');
                closeMenu();
              }}
            >
              Import Wallet
            </MenuItem>
            <MenuItem
              onClick={() => {
                navigate('/');
                closeMenu();
              }}
            >
              Logout
            </MenuItem>
          </Menu>
        </Toolbar>
      </AppBar>
    </Box>
  );
}
