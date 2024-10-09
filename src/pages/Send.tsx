import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Typography,
} from '@mui/material';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, Error } from '../bindings';
import Container from '../components/Container';
import ErrorDialog from '../components/ErrorDialog';
import Form, { FormValue } from '../components/Form';
import { useWalletState } from '../state';
import Header from '@/components/Header';

export default function Send() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const [isConfirmOpen, setConfirmOpen] = useState(false);
  const [values, setValues] = useState({
    address: '',
    amount: '',
    fee: '',
  });
  const [error, setError] = useState<Error | null>(null);

  const submit = () => {
    commands.send(values.address, values.amount, values.fee).then((result) => {
      if (result.status === 'ok') {
        navigate(-1);
      } else {
        console.error(result.error);
        setError(result.error);
      }
    });
  };

  return (
    <>
      <Header
        title={`Send ${walletState.sync.unit.ticker}`}
        back={() => navigate(-1)}
      />

      <Container>
        <Form
          fields={[
            { id: 'address', type: 'text', label: 'Address' },
            {
              id: 'amount',
              type: 'amount',
              label: 'Amount',
              unit: walletState.sync.unit,
            },
            {
              id: 'fee',
              type: 'amount',
              label: 'Fee',
              unit: walletState.sync.unit,
            },
          ]}
          values={values}
          setValues={setValues as (values: Record<string, FormValue>) => void}
          submitName={`Send ${walletState.sync.unit.ticker}`}
          onSubmit={() => setConfirmOpen(true)}
        />

        <Dialog open={isConfirmOpen} onClose={() => setConfirmOpen(false)}>
          <DialogTitle>
            Are you sure you want to send {walletState.sync.unit.ticker}?
          </DialogTitle>
          <DialogContent>
            <DialogContentText>
              This transaction cannot be reversed once it has been initiated.
              <Typography variant='h6' color='text.primary' mt={2}>
                Amount
              </Typography>
              <Typography sx={{ wordBreak: 'break-all' }}>
                {values.amount} {walletState.sync.unit.ticker} (with a fee of{' '}
                {values.fee} {walletState.sync.unit.ticker})
              </Typography>
              <Typography variant='h6' color='text.primary' mt={2}>
                Address
              </Typography>
              <Typography sx={{ wordBreak: 'break-all' }}>
                {values.address}
              </Typography>
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
      </Container>

      <ErrorDialog error={error} setError={setError} />
    </>
  );
}
