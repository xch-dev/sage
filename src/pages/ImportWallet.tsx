import { Alert, Button, TextField, Typography } from '@mui/material';
import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { activeWallet, importWallet } from '../commands';
import Container from '../components/Container';
import NavBar from '../components/NavBar';
import { WalletInfo } from '../models';

export default function ImportWallet() {
  const navigate = useNavigate();

  const [currentWallet, setCurrentWallet] = useState<WalletInfo | null>(null);
  const [name, setName] = useState('');
  const [key, setKey] = useState('');

  const [nameError, setNameError] = useState(false);
  const [keyError, setKeyError] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const keyRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    activeWallet().then(setCurrentWallet);
  }, []);

  const submit = () => {
    setNameError(!name);
    setKeyError(!key);

    if (nameError || keyError || !name || !key) return;

    importWallet(name, key)
      .then(() => navigate('/wallet'))
      .catch(setError);
  };

  return (
    <>
      <NavBar
        label='Import Wallet'
        back={() => {
          if (currentWallet) {
            navigate('/wallet');
          } else {
            navigate('/');
          }
        }}
      />

      <Container>
        <TextField
          label='Wallet Name'
          fullWidth
          autoFocus
          error={nameError}
          value={name}
          onChange={(event) => setName(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              keyRef.current?.focus();
            }
          }}
        />

        <Typography sx={{ mt: 2 }}>
          Enter your mnemonic, private key, or public key below. If it's a
          public key, it will be imported as a read-only cold wallet.
        </Typography>

        <TextField
          label='Wallet Key'
          rows={2}
          inputRef={keyRef}
          fullWidth
          multiline
          error={keyError}
          value={key}
          onChange={(event) => setKey(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              submit();
            }
          }}
          sx={{ mt: 2 }}
        />

        <Button
          variant='contained'
          fullWidth
          sx={{ mt: 3 }}
          disabled={!key || !name}
          onClick={submit}
        >
          Import Wallet
        </Button>

        {error && (
          <Alert variant='outlined' severity='error' sx={{ mt: 2 }}>
            {error}
          </Alert>
        )}
      </Container>
    </>
  );
}
