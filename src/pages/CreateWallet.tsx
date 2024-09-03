import { ContentCopy, Refresh } from '@mui/icons-material';
import {
  Alert,
  Box,
  Button,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Divider,
  FormControlLabel,
  IconButton,
  Switch,
  TextField,
  Tooltip,
} from '@mui/material';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, WalletInfo } from '../bindings';
import Container from '../components/Container';
import NavBar from '../components/NavBar';

export default function CreateWallet() {
  const navigate = useNavigate();

  const [mnemonic, setMnemonic] = useState<string | null>();
  const [name, setName] = useState('');
  const [use24Words, setUse24Words] = useState(true);
  const [saveMnemonic, setSaveMnemonic] = useState(true);
  const [isConfirmOpen, setConfirmOpen] = useState(false);

  const [nameError, setNameError] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [currentWallet, setCurrentWallet] = useState<WalletInfo | null>(null);

  useEffect(() => {
    commands.activeWallet().then((res) => {
      if (res.status === 'ok') {
        setCurrentWallet(res.data);
      }
    });
  }, []);

  const loadMnemonic = useCallback(() => {
    commands.generateMnemonic(use24Words).then((res) => {
      if (res.status === 'ok') {
        setMnemonic(res.data);
      }
    });
  }, [use24Words]);

  const copyMnemonic = useCallback(() => {
    if (!mnemonic) return;
    writeText(mnemonic);
  }, [mnemonic]);

  useEffect(() => {
    loadMnemonic();
  }, [use24Words, loadMnemonic]);

  const submit = () => {
    setNameError(!name);

    if (nameError || !name || !mnemonic) return;

    commands
      .createWallet(name, mnemonic, saveMnemonic)
      .then((res) => {
        if (res.status === 'ok') {
          navigate('/wallet');
        }
      })
      .catch(setError);
  };

  return (
    <>
      <NavBar
        label='Create Wallet'
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
          required
          autoFocus
          value={name}
          error={nameError}
          onChange={(event) => setName(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              submit();
            }
          }}
        />
        <FormControlLabel
          sx={{ mt: 2 }}
          control={
            <Switch
              checked={use24Words}
              onChange={(event) => setUse24Words(event.target.checked)}
            />
          }
          label={
            <Tooltip
              title='While 12 word mnemonics are sufficiently hard to crack, you can choose to use 24 instead if you want to.'
              placement='bottom-start'
              enterDelay={750}
            >
              <span>Use 24 words</span>
            </Tooltip>
          }
        />
        <Box display='flex' alignItems='center' justifyContent='space-between'>
          <FormControlLabel
            control={
              <Switch
                checked={saveMnemonic}
                onChange={(event) => setSaveMnemonic(event.target.checked)}
              />
            }
            label={
              <Tooltip
                title='By disabling this you are creating a cold wallet, with no ability to sign transactions. The mnemonic will need to be saved elsewhere.'
                placement='bottom-start'
                enterDelay={750}
              >
                <span>Save mnemonic</span>
              </Tooltip>
            }
          />
          <Box display='flex' alignItems='center'>
            <IconButton size='medium' onClick={loadMnemonic}>
              <Refresh />
            </IconButton>
            <IconButton size='medium' onClick={copyMnemonic}>
              <ContentCopy />
            </IconButton>
          </Box>
        </Box>

        <Divider sx={{ mt: 2 }} />

        <Box mt={3} textAlign='center'>
          {mnemonic
            ?.split(' ')
            .map((word, i) => (
              <Chip key={i} label={word} variant='outlined' sx={{ m: '2px' }} />
            ))}
        </Box>

        <Button
          variant='contained'
          fullWidth
          sx={{ mt: 3 }}
          disabled={!mnemonic || !name}
          onClick={() => {
            if (saveMnemonic) submit();
            else setConfirmOpen(true);
          }}
        >
          Create Wallet
        </Button>

        {error && (
          <Alert variant='outlined' severity='error' sx={{ mt: 2 }}>
            {error}
          </Alert>
        )}
      </Container>

      <Dialog open={isConfirmOpen} onClose={() => setConfirmOpen(false)}>
        <DialogTitle>Did you save your mnemonic?</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Make sure you have saved your mnemonic. You will not be able to
            access it later, since it will not be saved in the wallet. You will
            also not be able to make transactions with this wallet.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfirmOpen(false)}>Cancel</Button>
          <Button
            onClick={() => {
              setConfirmOpen(false);
              submit();
            }}
            autoFocus
          >
            Confirm
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
}
