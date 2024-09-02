import { useNavigate } from 'react-router-dom';
import Container from '../components/Container';
import NavBar from '../components/NavBar';
import { useWalletState } from '../state';

export default function Receive() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  return (
    <>
      <NavBar label='Receive XCH' back={() => navigate(-1)} />

      <Container>{walletState.syncInfo.address}</Container>
    </>
  );
}
