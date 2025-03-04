import { commands, OfferRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { OfferCard } from '@/components/OfferCard';
import { useErrors } from '@/hooks/useErrors';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';

export function ViewSavedOffer() {
  const { offer_id: offerId } = useParams();
  const { addError } = useErrors();

  const [record, setRecord] = useState<OfferRecord | null>(null);

  useEffect(() => {
    if (!offerId) return;

    commands
      .getOffer({ offer_id: offerId })
      .then((data) => setRecord(data.offer))
      .catch(addError);
  }, [offerId, addError]);

  return (
    <>
      <Header title='Saved Offer' />

      <Container>
        {record && (
          <OfferCard
            summary={record.summary}
            status={record.status}
            creation_date={record.creation_date}
          />
        )}
      </Container>
    </>
  );
}
