import { TextField } from '@mui/material';
import { useNavigate } from 'react-router-dom';
import Container from '../components/Container';
import NavBar from '../components/NavBar';

export default function ImportWallet() {
  const navigate = useNavigate();

  return (
    <>
      <NavBar label='Import Wallet' back={() => navigate('/')} />
      <Container>
        <TextField label='Wallet Name' variant='outlined' fullWidth />
      </Container>
    </>
  );
}
