import { commands, OfferSummary, TakeOfferResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Loading } from '@/components/Loading';
import { OfferCard } from '@/components/OfferCard';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useErrors } from '@/hooks/useErrors';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

export function ViewOffer() {
  const { offer } = useParams();
  const { addError } = useErrors();
  const walletState = useWalletState();
  const navigate = useNavigate();

  const [isLoading, setIsLoading] = useState(true);
  const [loadingStatus, setLoadingStatus] = useState('Initializing...');
  const [summary, setSummary] = useState<OfferSummary | null>(null);
  const [response, setResponse] = useState<TakeOfferResponse | null>(null);
  const [fee, setFee] = useState('');

  useEffect(() => {
    if (!offer) return;

    const loadOffer = async () => {
      setIsLoading(true);
      setLoadingStatus('Fetching offer details...');

      commands
        .viewOffer({ offer })
        .then((data) => {
          setSummary(data.offer);
          setLoadingStatus('Processing offer data...');
        })
        .catch(addError)
        .finally(() => setIsLoading(false));
    };

    loadOffer();
  }, [offer, addError]);

  const importOffer = () => {
    commands
      .importOffer({ offer: offer! })
      .then(() => navigate('/offers'))
      .catch(addError);
  };

  const take = async () => {
    await commands
      .takeOffer({
        offer: offer!,
        fee: toMojos(fee || '0', walletState.sync.unit.decimals),
      })
      .then((result) => setResponse(result))
      .catch(addError);
  };

  return (
    <>
      <Header title='View Offer' />

      <Container>
        {isLoading ? (
          <Loading className='my-8' text={loadingStatus} />
        ) : (
          summary && (
            <>
              <OfferCard summary={summary}>
                <div className='flex flex-col space-y-1.5'>
                  <Label htmlFor='fee'>Network Fee</Label>
                  <Input
                    id='fee'
                    type='text'
                    placeholder='0.00'
                    className='pr-12'
                    value={fee}
                    onChange={(e) => setFee(e.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter') {
                        event.preventDefault();
                        take();
                      }
                    }}
                  />
                </div>
              </OfferCard>

              <div className='mt-4 flex gap-2'>
                <Button variant='outline' onClick={importOffer}>
                  Save Offer
                </Button>

                <Button onClick={take}>Take Offer</Button>
              </div>
            </>
          )
        )}
      </Container>

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => navigate('/offers')}
      />
    </>
  );
}
