import { Button, TextField, Typography } from '@mui/material';
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
          label='Destination address'
          fullWidth
          value={address}
          onChange={(e) => setAddress(e.target.value)}
          error={address.length > 0 && !addressValid}
        />
        <Typography sx={{ mt: 1 }} variant='subtitle1' color='text.secondary'>
          Transactions cannot be reversed, so make sure that this is the correct
          address before sending.
        </Typography>

        <TextField
          sx={{ mt: 2 }}
          label='Amount'
          fullWidth
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          error={amount.length > 0 && amountNum.isNaN()}
        />
        <Typography sx={{ mt: 1 }} variant='subtitle1' color='text.secondary'>
          The amount to send in {walletState.sync.unit.ticker}. Your balance is
          <b>{' ' + walletState.sync.balance}.</b>
        </Typography>

        <TextField
          sx={{ mt: 2 }}
          label='Fee'
          fullWidth
          value={fee}
          onChange={(e) => setFee(e.target.value)}
          error={fee.length > 0 && feeNum.isNaN()}
        />
        <Typography sx={{ mt: 1 }} variant='subtitle1' color='text.secondary'>
          The network fee that will be included. If the mempool is full, this
          will help expedite the transaction.
        </Typography>

        <Button
          sx={{ mt: 2 }}
          variant='contained'
          fullWidth
          disabled={
            greaterThanBalance ||
            amountNum.isNaN() ||
            feeNum.isNaN() ||
            !addressValid
          }
          onClick={submit}
        >
          Send
        </Button>
      </Container>
    </>
  );
}
