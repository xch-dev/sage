import { ContentCopy } from '@mui/icons-material';
import { IconButton } from '@mui/material';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { activeWallet } from '../commands';
import NavBar from '../components/NavBar';
import { WalletInfo } from '../models';

export default function Wallet() {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);

  useEffect(() => {
    activeWallet().then((wallet) => {
      if (wallet) return setWallet(wallet);
      navigate('/');
    });
  }, [navigate]);

  return (
    <>
      <NavBar label={wallet?.name ?? 'Loading...'} back='logout' />
      <IconButton>
        <ContentCopy />
      </IconButton>
    </>
  );
}
