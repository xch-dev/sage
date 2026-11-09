import { events } from '@/bindings';
import { processPendingOfferUploads } from '@/lib/pendingOfferUploads';
import { useEffect } from 'react';

/** How often to check for failed uploads that are due to be tried again. */
const RETRY_CHECK_MS = 30_000;

/**
 * Uploads the wallet's offers to the marketplaces chosen when they were made,
 * once the transactions splitting their coins are confirmed. This is checked
 * when the wallet is opened, whenever its coins or offers change, and
 * periodically so that failed uploads are tried again.
 */
export function usePendingOfferUploadsLifecycle(fingerprint: number | null) {
  useEffect(() => {
    if (fingerprint === null) return;

    let isRunning = false;
    let isQueued = false;

    const process = async () => {
      if (isRunning) {
        isQueued = true;
        return;
      }

      isRunning = true;

      try {
        do {
          isQueued = false;
          await processPendingOfferUploads(fingerprint);
        } while (isQueued);
      } catch (error) {
        console.error('Failed to upload pending offers', error);
      } finally {
        isRunning = false;
      }
    };

    void process();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'coin_state') {
        void process();
      }
    });

    const interval = setInterval(() => void process(), RETRY_CHECK_MS);

    return () => {
      unlisten.then((u) => u());
      clearInterval(interval);
    };
  }, [fingerprint]);
}
