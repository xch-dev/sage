import {
  Box,
  Divider,
  FormControlLabel,
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

      <FormControlLabel
        sx={{ mt: 4 }}
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
        sx={{ mt: 2 }}
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
    </>
  );
}
