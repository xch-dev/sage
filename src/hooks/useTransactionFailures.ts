import { useEffect } from 'react';
import { toast } from 'react-toastify';
import { t } from '@lingui/core/macro';
import { events } from '../bindings';

export function useTransactionFailures() {
  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'transaction_failed') {
        const { transaction_id, error } = event.payload;

        // Log error to console
        console.error('Transaction failed:', {
          transaction_id,
          error,
          timestamp: new Date().toISOString(),
        });

        // Show toast notification with error message if available
        const message = error
          ? t`Transaction failed: ${error}`
          : t`Transaction failed: ${transaction_id.slice(0, 8)}...`;

        toast.error(message, {
          autoClose: false, // Don't auto-close, let user dismiss manually
          toastId: `transaction-failed-${transaction_id}`, // Prevent duplicates
          style: {
            whiteSpace: 'pre-wrap', // Preserve line breaks and wrap text
            wordBreak: 'break-word', // Break long words if needed
            maxWidth: '500px', // Increase max width for longer messages
          },
        });
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);
}
