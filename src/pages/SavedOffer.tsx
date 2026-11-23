import { commands, events, OfferRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { OfferCard } from '@/components/OfferCard';
import { useErrors } from '@/hooks/useErrors';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';

export function SavedOffer() {
  const { offer_id: offerId } = useParams();
  const { addError } = useErrors();

  const [record, setRecord] = useState<OfferRecord | null>(null);

  useEffect(() => {
    if (!offerId) return;

    const refresh = () => {
      commands
        .getOffer({ offer_id: offerId })
        .then((data) => setRecord(data.offer))
        .catch(addError);
    };

    refresh();

    // The offer's status changes as its coins are split, taken, or cancelled.
    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'coin_state') {
        refresh();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [offerId, addError]);

  return (
    <>
      <Header title='Saved Offer' />

      <Container>
        {record && (
          <OfferCard
            offerId={record.offer_id}
            offer={record.offer}
            status={record.status}
            summary={record.summary}
            creationTimestamp={record.creation_timestamp}
          />
        )}
      </Container>
    </>
  );
}
