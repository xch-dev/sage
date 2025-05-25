import { useEffect } from 'react';
import { toast } from 'react-toastify';
import { t } from '@lingui/core/macro';
import { events } from '../bindings';

export function useTransactionFailures() {
    useEffect(() => {
        const unlisten = events.syncEvent.listen((event) => {
            if (event.payload.type === 'transaction_failed') {
                const { transaction_id } = event.payload;

                // Log error to console
                console.error('Transaction failed:', {
                    transaction_id,
                    timestamp: new Date().toISOString(),
                });

                // Show toast notification
                toast.error(
                    t`Transaction failed: ${transaction_id.slice(0, 8)}...`,
                    {
                        autoClose: 8000, // Show for 8 seconds
                        toastId: `transaction-failed-${transaction_id}`, // Prevent duplicates
                    }
                );
            }
        });

        return () => {
            unlisten.then((u) => u());
        };
    }, []);
} 