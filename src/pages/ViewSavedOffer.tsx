import { commands, Error, OfferRecord } from '@/bindings';
import Container from '@/components/Container';
import ErrorDialog from '@/components/ErrorDialog';
import Header from '@/components/Header';
import { OfferCard } from '@/components/OfferCard';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

export function ViewSavedOffer() {
  const { offer_id: offerId } = useParams();

  const navigate = useNavigate();

  const [record, setRecord] = useState<OfferRecord | null>(null);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!offerId) return;

    commands.getOffer({ offer_id: offerId }).then((result) => {
      if (result.status === 'error') {
        setError(result.error);
      } else {
        setRecord(result.data.offer);
      }
    });
  }, [offerId]);

  return (
    <>
      <Header title='Saved Offer' />

      <Container>
        {record && <OfferCard summary={record.summary} />}

        <ErrorDialog
          error={error}
          setError={(error) => {
            setError(error);
            if (error === null) navigate('/offers');
          }}
        />
      </Container>
    </>
  );
}
