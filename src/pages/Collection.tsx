import Container from '@/components/Container';
import Header from '@/components/Header';
import { useParams } from 'react-router-dom';

export default function Collection() {
  const { collection_id: collectionId } = useParams();

  return (
    <>
      <Header title='Unknown Collection' />
      <Container></Container>
    </>
  );
}
