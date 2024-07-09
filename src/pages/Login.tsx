import {
  Button,
  Card,
  CardActionArea,
  CardContent,
  Typography,
} from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { useNavigate } from 'react-router-dom';
import Container from '../components/Container';
import NavBar from '../components/NavBar';

export default function Login() {
  const navigate = useNavigate();

  return (
    <>
      <NavBar label='Wallet Login' back={null} />
      <Container>
        <Typography variant='h3' component='div' textAlign='center'>
          Sage Wallet
        </Typography>

        <Typography variant='body1' component='div' textAlign='center' mt={3}>
          There aren't any wallets to log into yet. To get started, create a new
          wallet or import an existing one.
        </Typography>

        <Button
          variant='contained'
          color='primary'
          fullWidth
          sx={{ mt: 4 }}
          onClick={() => navigate('/create')}
        >
          Create Wallet
        </Button>
        <Button
          variant='outlined'
          color='secondary'
          fullWidth
          sx={{ mt: 2 }}
          onClick={() => navigate('/import')}
        >
          Import Wallet
        </Button>
      </Container>
    </>
  );
}

function WalletList() {
  return (
    <Grid2 container spacing={2} margin={2} justifyContent={'center'}>
      <WalletItem />
      <WalletItem />
      <WalletItem />
    </Grid2>
  );
}

function WalletItem() {
  return (
    <Grid2 xs={12} sm={6} md={4}>
      <Card>
        <CardActionArea>
          <CardContent>
            <Typography variant='h5' component='div'>
              Wallet Name
            </Typography>
            <Typography color='text.secondary' mt={1}>
              423431
            </Typography>
          </CardContent>
        </CardActionArea>
      </Card>
    </Grid2>
  );
}
