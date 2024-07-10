import {
  Box,
  Divider,
  FormControl,
  FormControlLabel,
  InputLabel,
  MenuItem,
  Select,
  Switch,
  TextField,
  Typography,
} from '@mui/material';
import { useContext, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { DarkModeContext } from '../App';
import * as commands from '../commands';
import Container from '../components/Container';
import NavBar from '../components/NavBar';
import { DerivationMode, WalletConfig, WalletInfo } from '../models';

export default function Settings() {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);

  const { dark, setDark } = useContext(DarkModeContext);

  useEffect(() => {
    commands.activeWallet().then(setWallet);
  }, []);

  return (
    <>
      <NavBar
        label='Settings'
        back={() => {
          if (wallet) {
            navigate('/wallet');
          } else {
            navigate('/');
          }
        }}
      />
      <Container>
        <Typography variant='h5'>App</Typography>
        <FormControlLabel
          sx={{ mt: 2 }}
          control={
            <Switch
              checked={dark}
              onChange={(event) => setDark(event.target.checked)}
            />
          }
          label='Dark mode'
        />
        {wallet && <WalletSettings wallet={wallet} />}
      </Container>
    </>
  );
}

function WalletSettings(props: { wallet: WalletInfo }) {
  const [name, setName] = useState(props.wallet.name);
  const [derivationMode, setDerivationMode] = useState<DerivationMode | null>(
    null,
  );

  const [config, setConfig] = useState<WalletConfig | null>(null);

  useEffect(() => {
    commands.walletConfig(props.wallet.fingerprint).then(setConfig);
  }, [props.wallet.fingerprint]);

  console.log(config);

  return (
    <>
      <Divider sx={{ mt: 3 }} />
      <Box
        display='flex'
        justifyContent='space-between'
        alignItems='center'
        mt={3}
      >
        <Typography variant='h5'>Wallet</Typography>

        <Typography
          variant='h6'
          sx={{ fontWeight: 'normal' }}
          color='text.secondary'
        >
          {props.wallet.fingerprint}
        </Typography>
      </Box>

      <TextField
        sx={{ mt: 4 }}
        label='Wallet Name'
        fullWidth
        value={name}
        error={!name}
        onChange={(event) => setName(event.target.value)}
        onBlur={() => {
          if (name !== config?.name) {
            if (config) {
              config.name = name;
              setConfig(config);
            }
            if (name) commands.renameWallet(props.wallet.fingerprint, name);
          }
        }}
      />

      <FormControl fullWidth sx={{ mt: 4 }}>
        <InputLabel id='derivation-mode'>Derivation Mode</InputLabel>
        <Select
          labelId='derivaiton-mode'
          value={
            derivationMode ?? config?.derivation_mode ?? DerivationMode.Generate
          }
          label='Derivation Mode'
          onChange={(event) => {
            const mode = event.target.value as DerivationMode;
            commands.setDerivationMode(props.wallet.fingerprint, mode);
            setDerivationMode(mode);
          }}
        >
          <MenuItem value={DerivationMode.Generate}>
            Generate addresses automatically
          </MenuItem>
          <MenuItem value={DerivationMode.Cycle}>
            Cycle through existing addresses
          </MenuItem>
          <MenuItem value={DerivationMode.Reuse}>
            Reuse the same address
          </MenuItem>
        </Select>
      </FormControl>
    </>
  );
}
