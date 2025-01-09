import { useLocation, useNavigate } from 'react-router-dom';
import {
  scan,
  Format,
  cancel,
  startScan,
  stopScan,
  Scanned,
} from '@tauri-apps/plugin-barcode-scanner';
import Layout from '@/components/Layout';
import Header from '@/components/Header';
import { useEffect, useCallback, useState, useRef } from 'react';
import { useNavigationStore } from '@/state';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';
import { fetch } from '@tauri-apps/plugin-http';
import { useErrors } from '@/hooks/useErrors';
import { CustomError } from '@/contexts/ErrorContext';

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

const fetchOfferCodeOffer = async (id: string): Promise<string> => {
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

const isValidHostname = (url: string, expectedHostname: string) => {
  try {
    const parsedUrl = new URL(url);
    return parsedUrl.hostname === expectedHostname;
  } catch {
    return false;
  }
};

const extractDexieOfferId = (url: string) => {
  try {
    const parsedUrl = new URL(url);
    const segments = parsedUrl.pathname.split('/');
    const lastSegment = segments[segments.length - 1];

    if (segments.includes('offers') && lastSegment) {
      return lastSegment;
    }

    return null;
  } catch {
    return null;
  }
};

const extractOfferId = (url: string) => {
  try {
    const parsedUrl = new URL(url);
    const path = parsedUrl.pathname.replace(/^\//, '');
    return path || null;
  } catch {
    return null;
  }
};

export default function QRScanner() {
  const navigate = useNavigate();
  const { state } = useLocation();
  const returnPath = state?.returnTo || '/';
  const { setReturnValue } = useNavigationStore();
  const { addError } = useErrors();
  const scannedQRCode = useRef(false);
  const scannerInitialized = useRef(false);

  const handleScanSuccess = useCallback(
    async (result: Scanned) => {
      if (scannedQRCode.current) {
        return;
      }

      scannedQRCode.current = true;
      const content = result.content;

      if (returnPath.startsWith('/offers')) {
        if (isValidHostname(content, 'dexie.space')) {
          const offerId = extractDexieOfferId(content);
          if (!offerId) {
            throw {
              kind: 'api',
              reason: t`Invalid Dexie offer URL format`,
            } as CustomError;
          }
          const data = await fetchDexieOffer(offerId);
          if (!data) {
            throw {
              kind: 'api',
              reason: t`Failed to fetch Dexie offer data`,
            } as CustomError;
          }
          navigate(`/offers/view/${encodeURIComponent(data.trim())}`, {
            replace: true,
          });
        } else if (isValidHostname(content, 'offerco.de')) {
          const offerId = extractOfferId(content);
          if (!offerId) {
            throw {
              kind: 'api',
              reason: t`Invalid Offerco.de offer URL format`,
            } as CustomError;
          }
          const data = await fetchOfferCodeOffer(offerId);
          if (!data) {
            throw {
              kind: 'api',
              reason: t`Failed to fetch Offerco.de offer data`,
            } as CustomError;
          }
          navigate(`/offers/view/${encodeURIComponent(data.trim())}`, {
            replace: true,
          });
        } else {
          navigate(`/offers/view/${encodeURIComponent(content.trim())}`, {
            replace: true,
          });
        }
      } else {
        setReturnValue(returnPath, { status: 'success', data: content });
        navigate(-1);
      }
    },
    [navigate, returnPath, setReturnValue],
  );

  useEffect(() => {
    if (scannerInitialized.current) {
      return;
    }

    const initializeScanner = async () => {
      try {
        scannerInitialized.current = true;
        await startScan(
          {
            windowed: true,
            formats: [Format.QRCode],
          },
          async (result) => {
            try {
              await handleScanSuccess(result);
            } catch (err) {
              // Convert unknown error to CustomError
              const error = err as Error;
              addError({
                kind: 'api',
                reason: error.message || 'Failed to process QR code',
              });
              navigate(returnPath, { replace: true });
            }
          },
        );
      } catch (err) {
        const error = err as Error;
        addError({
          kind: 'api',
          reason: error.message || 'Failed to initialize scanner',
        });
        scannerInitialized.current = false;
        navigate(returnPath, { replace: true });
      }
    };

    initializeScanner();

    return () => {
      stopScan().catch(console.error);
    };
  }, [handleScanSuccess, addError, navigate, returnPath]);

  const cancelScan = useCallback(() => {
    stopScan()
      .catch(console.error)
      .finally(() => navigate(returnPath, { replace: true }));
  }, [navigate, returnPath]);

  return (
    <Layout transparentBackground={true}>
      <Header title={t`Scan QR Code`} back={cancelScan} />
      <div className='relative w-full h-full bg-opacity-1'>
        <div className='absolute inset-0 bg-black bg-opacity-0'>
          <div className='absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2'>
            <div className='relative w-64 h-64'>
              <div className='absolute top-0 left-0 w-8 h-8 border-l-4 border-t-4 border-white' />
              <div className='absolute top-0 right-0 w-8 h-8 border-r-4 border-t-4 border-white' />
              <div className='absolute bottom-0 left-0 w-8 h-8 border-l-4 border-b-4 border-white' />
              <div className='absolute bottom-0 right-0 w-8 h-8 border-r-4 border-b-4 border-white' />
            </div>
            <p className='text-white text-center mt-8'>
              <Trans>Position the QR code within the frame</Trans>
            </p>
          </div>
        </div>
      </div>
    </Layout>
  );
}
