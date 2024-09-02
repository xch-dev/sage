import { useNavigate } from 'react-router-dom';
import Container from '../components/Container';
import NavBar from '../components/NavBar';

export default function Receive() {
  const navigate = useNavigate();

  return (
    <>
      <NavBar label='Receive XCH' back={() => navigate(-1)} />

      <Container></Container>
    </>
  );
}
