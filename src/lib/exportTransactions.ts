import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { formatTimestamp } from '@/lib/utils';
import { TransactionRecord } from '@/bindings';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

export async function exportTransactions(transactions: TransactionRecord[]) {
    try {
        if (transactions.length === 0) {
            toast.error(t`No transactions to export`);
            return;
        }

        console.log('Starting export...');

        // Create CSV content
        const headers = ['Height', 'Timestamp', 'Type', 'Amount', 'SignedAmount', 'Address', 'Item Type', 'Item ID'];
        const rows = transactions.flatMap((tx) => {
            const timestamp = formatTimestamp(tx.timestamp);
            return [
                ...tx.spent.map((coin) => [
                    tx.height,
                    timestamp.replace(/,/g, ''),
                    'Sent',
                    coin.amount.toString().replace(/,/g, ''),
                    "-" + coin.amount.toString().replace(/,/g, ''),
                    (coin.address || '').replace(/,/g, ''),
                    coin.type.toUpperCase(),
                    (coin.type === 'cat' ? coin.asset_id :
                        coin.type === 'nft' ? coin.launcher_id :
                            coin.type === 'did' ? coin.launcher_id : '').replace(/,/g, ''),
                ]),
                ...tx.created.map((coin) => [
                    tx.height,
                    timestamp.replace(/,/g, ''),
                    'Received',
                    coin.amount.toString().replace(/,/g, ''),
                    coin.amount.toString().replace(/,/g, ''),
                    (coin.address || '').replace(/,/g, ''),
                    coin.type.toUpperCase(),
                    (coin.type === 'cat' ? coin.asset_id :
                        coin.type === 'nft' ? coin.launcher_id :
                            coin.type === 'did' ? coin.launcher_id : '').replace(/,/g, ''),
                ]),
            ];
        });

        const csvContent = [
            headers.join(','),
            ...rows.map((row) => row.join(',')),
        ].join('\n');

        console.log('CSV content generated, opening save dialog...');

        // Open save dialog
        const filePath = await save({
            filters: [
                {
                    name: 'CSV',
                    extensions: ['csv'],
                },
            ],
            defaultPath: 'transactions.csv',
        });

        console.log('Save dialog result:', filePath);

        if (filePath) {
            console.log('Writing file to:', filePath);
            await writeTextFile(filePath, csvContent);
            toast.success(t`Transactions exported successfully`);
        } else {
            console.log('Save dialog cancelled');
        }
    } catch (error) {
        console.error('Failed to export transactions:', error);
        toast.error(t`Failed to export transactions: ${error}`);
    }
} 