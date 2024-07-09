import { DarkMode, LightMode } from '@mui/icons-material';
import { IconButton } from '@mui/material';
import { useContext } from 'react';
import { DarkModeContext } from '../App';

export default function DarkToggle() {
  const { toggle, isDark } = useContext(DarkModeContext);

  return (
    <IconButton onClick={toggle} color="primary">
      {isDark ? <LightMode /> : <DarkMode />}
    </IconButton>
  );
}
