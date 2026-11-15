import { exportText } from '@/lib/exportText';
import { PricedTokenRecord } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

/** Exports the tokens as CSV, with their value and price in `currency`, or without them if it's null. */
export async function exportTokens(
  tokens: PricedTokenRecord[],
  currency: string | null,
) {
  try {
    if (tokens.length === 0) {
      toast.error(t`No tokens to export`);
      return;
    }

    // Create CSV content
    const headers = [
      'Name',
      'Ticker',
      'Asset ID',
      'Balance',
      ...(currency ? [`Balance (${currency})`, `Price (${currency})`] : []),
      'Icon URL',
      'Type',
    ];

    const rows = tokens.map((token) => [
      (token.name || '').replace(/,/g, ''),
      (token.ticker || '').replace(/,/g, ''),
      token.asset_id || '',
      token.balance || '',
      ...(currency
        ? [
            token.balanceInFiat?.toString() || '',
            token.priceInFiat?.toString() || '',
          ]
        : []),
      (token.icon_url || '').replace(/,/g, ''),
      !token.asset_id ? 'XCH' : 'CAT',
    ]);

    const csvContent = [
      headers.join(','),
      ...rows.map((row) => row.join(',')),
    ].join('\n');

    toast.dismiss();

    if (await exportText(csvContent, 'tokens')) {
      toast.success(t`Tokens exported successfully`);
    }
  } catch (error) {
    console.error('Failed to export tokens:', error);
    toast.dismiss();
    const message = error as string;
    toast.error(t`Failed to export tokens: ${message}`);
  }
}
