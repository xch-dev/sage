import { Button, TextField, Typography } from '@mui/material';
import BigNumber from 'bignumber.js';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import * as commands from '../commands';
import Container from '../components/Container';
import NavBar from '../components/NavBar';
import { useWalletState } from '../state';

export default function Send() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const balance = BigNumber(walletState.syncInfo.balance);

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
      if (valid) {
        setValidAddress(address);
      }
    });
  }, [address, validAddress]);

  const addressValid = address === validAddress;

  return (
    <>
      <NavBar
        label={`Send ${walletState.syncInfo.ticker}`}
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
          The amount to send in {walletState.syncInfo.ticker}. Your balance is
          <b>{' ' + walletState.syncInfo.balance}.</b>
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
        >
          Send
        </Button>
      </Container>
    </>
  );
}
