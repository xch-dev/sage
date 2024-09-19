import { Button, FormControlLabel, Switch, TextField } from '@mui/material';
import BigNumber from 'bignumber.js';
import { useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands } from '../bindings';
import Container from '../components/Container';
import NavBar from '../components/NavBar';

export default function IssueCat() {
  const navigate = useNavigate();

  const [name, setName] = useState('');
  const [amount, setAmount] = useState('');
  const [fee, setFee] = useState('');
  const [multiIssuance, setMultiIssuance] = useState(false);

  const amountRef = useRef<HTMLInputElement>(null);
  const feeRef = useRef<HTMLInputElement>(null);

  const amountNum = BigNumber(amount);
  const amountValid =
    !amountNum.isNaN() &&
    amountNum.isFinite() &&
    !amountNum.isLessThanOrEqualTo(0);

  const feeNum = BigNumber(fee);
  const feeValid =
    !feeNum.isNaN() && feeNum.isFinite() && !feeNum.isLessThanOrEqualTo(0);

  const valid = name.length > 0 && amountValid && feeValid;

  const issue = () => {
    if (!valid) return;

    commands.issueCat(name, amount, fee).then((result) => {
      if (result.status === 'error') {
        console.error(result.error);
        return;
      }

      navigate('/wallet/tokens');
    });
  };

  return (
    <>
      <NavBar label='Issue CAT' back={() => navigate(-1)} />

      <Container>
        <TextField
          label='Name'
          autoFocus
          fullWidth
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              amountRef.current?.focus();
            }
          }}
        />

        <TextField
          sx={{ mt: 2 }}
          inputRef={amountRef}
          label='Amount'
          autoFocus
          fullWidth
          value={amount}
          error={amount.length > 0 && !amountValid}
          onChange={(e) => setAmount(e.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              feeRef.current?.focus();
            }
          }}
        />

        <TextField
          sx={{ mt: 2 }}
          inputRef={feeRef}
          label='Fee'
          autoFocus
          fullWidth
          value={fee}
          error={fee.length > 0 && !feeValid}
          onChange={(e) => setFee(e.target.value)}
        />

        <FormControlLabel
          sx={{ mt: 2 }}
          control={
            <Switch
              checked={multiIssuance}
              onChange={(event) => setMultiIssuance(event.target.checked)}
            />
          }
          label='Allow more to be issued later'
        />

        <Button
          sx={{ mt: 2 }}
          variant='contained'
          fullWidth
          disabled={!valid}
          onClick={issue}
        >
          Issue CAT
        </Button>
      </Container>
    </>
  );
}
