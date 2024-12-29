import { commands, OfferSummary, TakeOfferResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { OfferCard } from '@/components/OfferCard';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useErrors } from '@/hooks/useErrors';
import { toDecimal, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import BigNumber from 'bignumber.js';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Loading } from '@/components/ui/loading';

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
      try {
        setIsLoading(true);
        setLoadingStatus('Decoding offer...');
        await new Promise(resolve => setTimeout(resolve, 500));

        setLoadingStatus('Fetching offer details...');
        const data = await commands.viewOffer({ offer });

        setLoadingStatus('Processing offer data...');
        await new Promise(resolve => setTimeout(resolve, 300));

        setSummary(data.offer);
      } catch (error) {
        setLoadingStatus('Error loading offer');
        addError(error);
      } finally {
        setIsLoading(false);
      }
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
    try {
      const result = await commands.takeOffer({
        offer: offer!,
        fee: toMojos(fee || '0', walletState.sync.unit.decimals),
      });
      setResponse(result);
    } catch (error) {
      addError(error);
    }
  };

  return (
    <>
      <Header title='View Offer' />

      <Container>
        {isLoading ? (
          <Loading className="my-8" text={loadingStatus} />
        ) : summary && (
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
