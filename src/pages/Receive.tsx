import { useNavigate } from 'react-router-dom';
import Container from '../components/Container';
import NavBar from '../components/NavBar';
import { useWalletState } from '../state';

export default function Receive() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  return (
    <>
      <NavBar
        label={`Receive ${walletState.sync.unit.ticker}`}
        back={() => navigate(-1)}
      />

      <Container>{walletState.sync.receive_address}</Container>
    </>
  );
}
