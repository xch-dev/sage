import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  TextField,
  Typography,
} from '@mui/material';
import BigNumber from 'bignumber.js';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands } from '../bindings';
import Container from '../components/Container';
import NavBar from '../components/NavBar';
import { useWalletState } from '../state';

export default function Send() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const balance = BigNumber(walletState.sync.balance);

  const [address, setAddress] = useState('');
  const [validAddress, setValidAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [fee, setFee] = useState('');
  const [isConfirmOpen, setConfirmOpen] = useState(false);

  const amountNum = BigNumber(amount);
  const feeNum = BigNumber(fee);
  const total = amountNum.plus(feeNum);
  const greaterThanBalance = total.isGreaterThan(balance);

  useEffect(() => {
    if (address === validAddress) {
      return;
    }

    commands.validateAddress(address).then((valid) => {
      if (valid.status === 'ok' && valid.data) {
        setValidAddress(address);
      }
    });
  }, [address, validAddress]);

  const addressValid = address === validAddress;

  const submit = () => {
    commands.send(address, amount, fee).then((result) => {
      if (result.status === 'ok') {
        navigate(-1);
      } else {
        console.error(result.error);
      }
    });
  };

  return (
    <>
      <NavBar
        label={`Send ${walletState.sync.unit.ticker}`}
        back={() => navigate(-1)}
      />

      <Container>
        <TextField
          label='Destination Address'
          fullWidth
          value={address}
          onChange={(e) => setAddress(e.target.value)}
          error={address.length > 0 && !addressValid}
        />
        <Typography sx={{ mt: 1 }} variant='subtitle1'>
          Ensure this is correct. Transactions cannot be reversed.
        </Typography>

        <TextField
          sx={{ mt: 3 }}
          label='Amount'
          fullWidth
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          error={amount.length > 0 && amountNum.isNaN()}
        />
        <Typography sx={{ mt: 1 }} variant='subtitle1'>
          Your balance is {walletState.sync.balance}{' '}
          {walletState.sync.unit.ticker}.
        </Typography>

        <TextField
          sx={{ mt: 3 }}
          label='Network Fee'
          fullWidth
          value={fee}
          onChange={(e) => setFee(e.target.value)}
          error={fee.length > 0 && feeNum.isNaN()}
        />
        <Typography sx={{ mt: 1 }} variant='subtitle1'>
          This will help ensure your transaction is processed quickly.
        </Typography>

        <Button
          sx={{ mt: 3 }}
          variant='contained'
          fullWidth
          disabled={
            greaterThanBalance ||
            amountNum.isNaN() ||
            feeNum.isNaN() ||
            !addressValid
          }
          onClick={() => setConfirmOpen(true)}
        >
          Send
        </Button>

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
                {amount} {walletState.sync.unit.ticker} (with a fee of {fee}{' '}
                {walletState.sync.unit.ticker})
              </Typography>
              <Typography variant='h6' color='text.primary' mt={2}>
                Address
              </Typography>
              <Typography sx={{ wordBreak: 'break-all' }}>{address}</Typography>
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
    </>
  );
}
