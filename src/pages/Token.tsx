import { useParams } from 'react-router-dom';
import ListContainer from '../components/ListContainer';

export default function Token() {
  const { asset_id: assetId } = useParams();

  return (
    <>
      <ListContainer>{assetId}</ListContainer>
    </>
  );
}
