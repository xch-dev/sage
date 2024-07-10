import { DarkMode, LightMode } from '@mui/icons-material';
import { IconButton } from '@mui/material';
import { useContext } from 'react';
import { DarkModeContext } from '../App';

export default function DarkToggle() {
  const { toggle, dark } = useContext(DarkModeContext);

  return (
    <IconButton onClick={toggle} color='primary'>
      {dark ? <LightMode /> : <DarkMode />}
    </IconButton>
  );
}
