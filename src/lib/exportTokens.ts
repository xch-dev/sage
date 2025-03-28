import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';
import { TokenRecord } from '@/types/TokenViewProps';

export async function exportTokens(
    currentTokens: TokenRecord[]
) {
    try {
        console.log('Current Tokens:', currentTokens.length);
        if (currentTokens.length === 0) {
            toast.error(t`No tokens to export`);
            return;
        }

        // Create CSV content
        const headers = [
            'Name',
            'Ticker',
            'Asset ID',
            'Balance',
            'Balance (USD)',
            'Price (USD)',
            'Icon URL',
            'Type',
        ];

        const rows = currentTokens.map((token) => [
            (token.name || '').replace(/,/g, ''),
            (token.ticker || '').replace(/,/g, ''),
            (token.asset_id || ''),
            token.balance || '',
            token.balanceInUsd?.toString() || '',
            token.priceInUsd?.toString() || '',
            (token.icon_url || '').replace(/,/g, ''),
            token.isXch ? 'XCH' : 'CAT'
        ]);

        const csvContent = [
            headers.join(','),
            ...rows.map((row) => row.join(',')),
        ].join('\n');

        toast.dismiss();
        // Open save dialog
        const filePath = await save({
            filters: [
                {
                    name: 'CSV',
                    extensions: ['csv'],
                },
            ],
            defaultPath: 'tokens.csv',
        });

        if (filePath) {
            await writeTextFile(filePath, csvContent);
            toast.success(t`Tokens exported successfully`);
        } else {
            console.log('Save dialog cancelled');
        }
    } catch (error) {
        console.error('Failed to export tokens:', error);
        toast.dismiss();
        toast.error(t`Failed to export tokens: ${error}`);
    }
} 