import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
} from '@mui/material';
import { Error } from '../bindings';

export interface ErrorDialogProps {
  error: Error | null;
  setError: (error: Error | null) => void;
}

export default function ErrorDialog({ error, setError }: ErrorDialogProps) {
  let kind: string | null;

  switch (error?.kind) {
    case 'Client':
    case 'Database':
    case 'Keychain':
    case 'Logging':
    case 'Sync':
    case 'Serialization':
      kind = error.kind;
      break;

    case 'InsufficientFunds':
      kind = 'Transaction';
      break;

    case 'InvalidAddress':
      kind = 'Address';
      break;

    case 'InvalidAmount':
      kind = 'Amount';
      break;

    case 'InvalidAssetId':
      kind = 'Asset ID';
      break;

    case 'InvalidKey':
      kind = 'Key';
      break;

    case 'InvalidMnemonic':
      kind = 'Mnemonic';
      break;

    case 'InvalidLauncherId':
      kind = 'Launcher ID';
      break;

    case 'NotLoggedIn':
      kind = 'Account';
      break;

    case 'TransactionFailed':
      kind = 'Transaction';
      break;

    case 'UnknownNetwork':
      kind = 'Network';
      break;

    case 'UnknownFingerprint':
      kind = 'Fingerprint';
      break;

    case 'Wallet':
      kind = 'Wallet';
      break;

    case 'Io':
      kind = 'IO';
      break;

    default:
      kind = null;
  }

  return (
    <Dialog open={error !== null} onClose={() => setError(null)}>
      <DialogTitle>{kind ? `${kind} ` : ''}Error</DialogTitle>
      <DialogContent>
        <DialogContentText>{error?.reason}</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setError(null)} autoFocus>
          Ok
        </Button>
      </DialogActions>
    </Dialog>
  );
}
