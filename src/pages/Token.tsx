import { useNavigate, useParams } from 'react-router-dom';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';

export default function Token() {
  const navigate = useNavigate();
  const { asset_id: assetId } = useParams();

  return (
    <>
      <NavBar label='Token' back={() => navigate(-1)} />

      <ListContainer>{assetId}</ListContainer>
    </>
  );
}
