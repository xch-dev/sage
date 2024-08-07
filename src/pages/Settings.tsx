import { PersonSearch, WifiFind } from '@mui/icons-material';
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Divider,
  FormControl,
  FormControlLabel,
  IconButton,
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
import { isValidU32 } from '../validation';

export default function Settings() {
  const navigate = useNavigate();

  const [wallet, setWallet] = useState<WalletInfo | null>(null);

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
        <GlobalSettings />
        <NetworkSettings />
        {wallet && <WalletSettings wallet={wallet} />}
      </Container>
    </>
  );
}

function GlobalSettings() {
  const { dark, setDark } = useContext(DarkModeContext);

  return (
    <>
      <Typography variant='h5'>Global</Typography>

      <FormControlLabel
        sx={{ mt: 2, display: 'block' }}
        control={
          <Switch
            checked={dark}
            onChange={(event) => setDark(event.target.checked)}
          />
        }
        label='Dark mode'
      />
    </>
  );
}

function NetworkSettings() {
  const [networkId, setNetworkId] = useState<string>('mainnet');

  const [isInfoOpen, setInfoOpen] = useState(false);

  return (
    <>
      <Divider sx={{ mt: 3 }} />

      <Box
        display='flex'
        justifyContent='space-between'
        alignItems='center'
        mt={3}
      >
        <Typography variant='h5'>Network</Typography>

        <IconButton onClick={() => setInfoOpen(true)}>
          <WifiFind />
        </IconButton>
      </Box>

      <FormControlLabel
        sx={{ mt: 2, display: 'block' }}
        control={<Switch defaultChecked={true} />}
        label='Discover peers automatically'
      />

      <TextField sx={{ mt: 3 }} label='Target Peers' fullWidth value={5} />

      <FormControl sx={{ mt: 4 }} fullWidth>
        <InputLabel id='network-label'>Selected Network</InputLabel>
        <Select
          label='Selected Network'
          labelId='network-label'
          value={networkId}
          onChange={(event) => setNetworkId(event.target.value)}
        >
          <MenuItem value='mainnet'>Mainnet</MenuItem>
          <MenuItem value='testnet11'>Testnet11</MenuItem>
        </Select>
      </FormControl>

      <Dialog open={isInfoOpen} onClose={() => setInfoOpen(false)}>
        <DialogTitle>Network Info</DialogTitle>
        <DialogContent>
          <DialogContentText>Test</DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setInfoOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </>
  );
}

function WalletSettings(props: { wallet: WalletInfo }) {
  const [name, setName] = useState(props.wallet.name);

  const [derivationMode, setDerivationMode] = useState<DerivationMode | null>(
    null,
  );

  const [derivationBatchSizeText, setDerivationBatchSize] = useState<
    string | null
  >(null);

  const derivationBatchSize =
    derivationBatchSizeText === null ? null : parseInt(derivationBatchSizeText);

  const invalidDerivationBatchSize =
    derivationBatchSize === null || !isValidU32(derivationBatchSize, 1);

  const [config, setConfig] = useState<WalletConfig | null>(null);

  useEffect(() => {
    commands.walletConfig(props.wallet.fingerprint).then(setConfig);
  }, [props.wallet.fingerprint]);

  const [isInfoOpen, setInfoOpen] = useState(false);

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

        <IconButton onClick={() => setInfoOpen(true)}>
          <PersonSearch />
        </IconButton>
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

      <FormControlLabel
        sx={{ mt: 4, display: 'block' }}
        control={
          <Switch
            checked={
              (derivationMode ??
                config?.derivation_mode ??
                DerivationMode.Automatic) == DerivationMode.Automatic
            }
            onChange={(event) => {
              const mode = event.target.checked
                ? DerivationMode.Automatic
                : DerivationMode.Manual;
              commands.setDerivationMode(props.wallet.fingerprint, mode);
              setDerivationMode(mode);
            }}
          />
        }
        label='Generate addresses automatically'
      />

      <TextField
        sx={{ mt: 3 }}
        label='Address Batch Size'
        fullWidth
        value={derivationBatchSizeText ?? config?.derivation_batch_size ?? 500}
        error={derivationBatchSizeText !== null && invalidDerivationBatchSize}
        disabled={
          (derivationMode ?? config?.derivation_mode) !==
          DerivationMode.Automatic
        }
        onChange={(event) => setDerivationBatchSize(event.target.value)}
        onBlur={() => {
          if (invalidDerivationBatchSize) return;

          if (derivationBatchSize !== config?.derivation_batch_size) {
            if (config) {
              config.derivation_batch_size = derivationBatchSize;
              setConfig(config);
            }
            commands.setDerivationBatchSize(
              props.wallet.fingerprint,
              derivationBatchSize,
            );
          }
        }}
      />

      <Dialog open={isInfoOpen} onClose={() => setInfoOpen(false)}>
        <DialogTitle>Wallet Info</DialogTitle>
        <DialogContent>
          <DialogContentText>{props.wallet.fingerprint}</DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setInfoOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </>
  );
}
