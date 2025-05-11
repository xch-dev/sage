import { commands, OfferSummary, TakeOfferResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { OfferConfirmation } from '@/components/confirmations/OfferConfirmation';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Loading } from '@/components/Loading';
import { OfferCard } from '@/components/OfferCard';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';

const isValidHostname = (url: string, expectedHostname: string) => {
  try {
    const parsedUrl = new URL(url);
    return parsedUrl.hostname === expectedHostname;
  } catch {
    return false;
  }
};

const extractOfferId = (url: string) => {
  try {
    const segments = url.split('/');
    const lastSegment = segments[segments.length - 1];
    return lastSegment;
  } catch {
    return null;
  }
};

const fetchDexieOffer = async (id: string): Promise<string> => {
  const response = await fetch(`https://api.dexie.space/v1/offers/${id}`);
  const data = await response.json();

  if (!data) {
    throw {
      kind: 'api',
      reason: '[Dexie] Invalid response data',
    } as CustomError;
  }

  if (data.success && data.offer?.offer) {
    return data.offer.offer;
  }

  throw {
    kind: 'api',
    reason: '[Dexie] Offer not found or invalid format',
  } as CustomError;
};

const fetchOfferCoOffer = async (id: string): Promise<string> => {
  const response = await fetch('https://offerco.de/api/v1/getoffer', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      'X-API-Key': '602307f09cc95d490061bda377079f44',
    },
    body: `short_code=${id}`,
  });

  const data = await response.json();

  if (data.status === 'success' && data.data?.offer_code) {
    return data.data.offer_code;
  }

  throw {
    kind: 'api',
    reason: 'Failed to fetch offer from Offerco.de',
  } as CustomError;
};

const resolveOffer = async (offerData: string): Promise<string> => {
  try {
    if (isValidHostname(offerData, 'dexie.space')) {
      const offerId = extractOfferId(offerData);
      if (offerId) {
        const resolvedOffer = await fetchDexieOffer(offerId);
        if (resolvedOffer) {
          return resolvedOffer;
        }
      }
    }

    if (isValidHostname(offerData, 'offerco.de')) {
      const offerId = extractOfferId(offerData);
      if (offerId) {
        const resolvedOffer = await fetchOfferCoOffer(offerId);
        if (resolvedOffer) {
          return resolvedOffer;
        }
      }
    }
  } catch {
    throw {
      kind: 'api',
      reason: 'Failed to resolve offer',
    } as CustomError;
  }

  return offerData;
};

export function ViewOffer() {
  const { offer } = useParams();
  const { addError } = useErrors();
  const walletState = useWalletState();
  const navigate = useNavigate();

  const [isLoading, setIsLoading] = useState(true);
  const [loadingStatus, setLoadingStatus] = useState(t`Initializing...`);
  const [summary, setSummary] = useState<OfferSummary | null>(null);
  const [response, setResponse] = useState<TakeOfferResponse | null>(null);
  const [fee, setFee] = useState('');

  useEffect(() => {
    if (!offer) return;

    const loadOffer = async () => {
      setIsLoading(true);
      setLoadingStatus(t`Fetching offer details...`);

      try {
        const resolvedOffer = await resolveOffer(offer);
        const data = await commands.viewOffer({ offer: resolvedOffer });
        setSummary(data.offer);
        setLoadingStatus(t`Processing offer data...`);
      } catch (error) {
        addError(error as CustomError);
      } finally {
        setIsLoading(false);
      }
    };

    loadOffer();
  }, [offer, addError]);

  const importOffer = async () => {
    if (!offer) return;

    try {
      const resolvedOffer = await resolveOffer(offer);
      await commands.importOffer({ offer: resolvedOffer });
      navigate('/offers');
    } catch (error) {
      addError(error as CustomError);
    }
  };

  const take = async () => {
    if (!offer) return;

    try {
      const resolvedOffer = await resolveOffer(offer);
      const result = await commands.takeOffer({
        offer: resolvedOffer,
        fee: toMojos(fee || '0', walletState.sync.unit.decimals),
      });
      setResponse(result);
    } catch (error) {
      addError(error as CustomError);
    }
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
                  <Label htmlFor='fee'>
                    <Trans>Network Fee</Trans>
                  </Label>
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
                  <Trans>Save Offer</Trans>
                </Button>

                <Button onClick={take}>
                  <Trans>Take Offer</Trans>
                </Button>
              </div>
            </>
          )
        )}
      </Container>

      <ConfirmationDialog
        showRecipientDetails={false}
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => navigate('/offers')}
        additionalData={{
          title: t`Take Offer`,
          content: response && summary && (
            <OfferConfirmation type='take' offer={summary} />
          ),
        }}
      />
    </>
  );
}
