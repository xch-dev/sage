import { ContentCopy, Refresh } from '@mui/icons-material';
import {
  Box,
  Button,
  Chip,
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
import { generateMnemonic } from '../commands';
import Container from '../components/Container';
import NavBar from '../components/NavBar';

export default function CreateWallet() {
  const navigate = useNavigate();

  const [mnemonic, setMnemonic] = useState<string | null>();
  const [name, setName] = useState('');
  const [use24Words, setUse24Words] = useState(true);
  const [nameError, setNameError] = useState(false);

  const loadMnemonic = useCallback(() => {
    generateMnemonic(use24Words).then(setMnemonic);
  }, [use24Words]);

  const copyMnemonic = useCallback(() => {
    if (!mnemonic) return;
    writeText(mnemonic);
  }, [mnemonic]);

  useEffect(() => {
    loadMnemonic();
  }, [use24Words, loadMnemonic]);

  const submit = () => {
    let valid = true;
    if (!name) {
      setNameError(true);
      valid = false;
    }

    if (!valid) return;

    navigate('/');
  };

  return (
    <>
      <NavBar label='Create Wallet' back={() => navigate('/')} />
      <Container>
        <TextField
          label='Wallet Name'
          variant='outlined'
          fullWidth
          required
          value={name}
          error={nameError}
          onChange={(event) => setName(event.target.value)}
        />
        <Box
          display='flex'
          alignItems='center'
          justifyContent='space-between'
          mt={2}
        >
          <FormControlLabel
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

        <Button variant='contained' fullWidth sx={{ mt: 3 }} onClick={submit}>
          Create Wallet
        </Button>
      </Container>
    </>
  );
}
