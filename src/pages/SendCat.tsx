import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { CatRecord, commands, Error, events } from '../bindings';
import Container from '../components/Container';
import ErrorDialog from '../components/ErrorDialog';
import Form, { FormValue } from '../components/Form';
import { useWalletState } from '../state';
import Header from '@/components/Header';

export default function SendCat() {
  const { asset_id: assetId } = useParams();
  const navigate = useNavigate();
  const walletState = useWalletState();

  const [cat, setCat] = useState<CatRecord | null>(null);
  const [isConfirmOpen, setConfirmOpen] = useState(false);
  const [values, setValues] = useState({
    address: '',
    amount: '',
    fee: '',
  });
  const [error, setError] = useState<Error | null>(null);

  const submit = () => {
    commands
      .sendCat(assetId!, values.address, values.amount, values.fee)
      .then((result) => {
        if (result.status === 'ok') {
          navigate(-1);
        } else {
          console.error(result.error);
          setError(result.error);
        }
      });
  };

  const updateCat = () => {
    commands.getCat(assetId!).then((result) => {
      if (result.status === 'ok') {
        setCat(result.data);
      } else {
        console.error(result.error);
        setError(result.error);
      }
    });
  };

  useEffect(() => {
    updateCat();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'cat_update') {
        updateCat();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  const ticker = cat?.ticker ?? 'CAT';

  return (
    <>
      <Header title={`Send ${ticker}`} />
      <Container>
        <Form
          fields={[
            { id: 'address', type: 'text', label: 'Address' },
            {
              id: 'amount',
              type: 'amount',
              label: 'Amount',
              unit: {
                ticker,
                decimals: 3,
              },
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
          submitName={`Send ${ticker}`}
          onSubmit={() => setConfirmOpen(true)}
        />

        <Dialog open={isConfirmOpen} onClose={() => setConfirmOpen(false)}>
          <DialogTitle>Are you sure you want to send {ticker}?</DialogTitle>
          <DialogContent>
            <DialogContentText>
              This transaction cannot be reversed once it has been initiated.
              <Typography variant='h6' color='text.primary' mt={2}>
                Amount
              </Typography>
              <Typography sx={{ wordBreak: 'break-all' }}>
                {values.amount} {ticker} (with a fee of {values.fee}{' '}
                {walletState.sync.unit.ticker})
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
