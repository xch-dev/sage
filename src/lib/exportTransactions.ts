import { TransactionRecord } from '@/bindings';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';
import { commands } from '@/bindings';
import { exportText } from './exportText';

interface TransactionQueryParams {
  search: string | null;
  ascending: boolean;
  page?: number;
  pageSize?: number;
}

export async function queryTransactions(
  params: TransactionQueryParams,
): Promise<{
  transactions: TransactionRecord[];
  total: number;
}> {
  return await commands.getTransactions({
    offset: params.page ? (params.page - 1) * (params.pageSize || 10) : 0,
    limit: params.pageSize || 1000000, // Use large limit if no pagination
    ascending: params.ascending,
    find_value: params.search || null,
  });
}

export async function exportTransactions(params: TransactionQueryParams) {
  try {
    toast.info(t`Fetching transactions...`, { autoClose: 45000 });

    // Fetch all transactions using the shared query function
    const { transactions: allTransactions } = await queryTransactions({
      ...params,
      pageSize: 1000000, // Get all transactions
    });

    // Create CSV content
    const headers = [
      'Height',
      'Timestamp (UTC)',
      'Type',
      'Amount',
      'Signed Amount',
      'Address',
      'Coin ID',
      'Coin Type',
      'Item ID',
      'Coin Name',
    ];
    const rows = allTransactions.flatMap((tx) => {
      // Convert timestamp to UTC
      const timestamp = tx.timestamp
        ? new Date(tx.timestamp * 1000).toISOString()
        : '';
      return [
        ...tx.spent.map((coin) => [
          tx.height,
          timestamp.replace(/,/g, ''),
          'Sent',
          coin.amount.toString(),
          '-' + coin.amount.toString(),
          coin.address || '',
          coin.coin_id,
          coin.type.toUpperCase(),
          coin.type === 'token' || coin.type === 'nft' || coin.type === 'did'
            ? coin.asset_id || ''
            : '',
          (coin.type === 'token' || coin.type === 'nft' || coin.type === 'did'
            ? coin.name || ''
            : ''
          ).replace(/,/g, ''),
        ]),
        ...tx.created.map((coin) => [
          tx.height,
          timestamp.replace(/,/g, ''),
          'Received',
          coin.amount.toString(),
          coin.amount.toString(),
          coin.address || '',
          coin.coin_id,
          coin.type.toUpperCase(),
          coin.type === 'token' || coin.type === 'nft' || coin.type === 'did'
            ? coin.asset_id || ''
            : '',
          (coin.type === 'token' || coin.type === 'nft' || coin.type === 'did'
            ? coin.name || ''
            : ''
          ).replace(/,/g, ''),
        ]),
      ];
    });

    const csvContent = [
      headers.join(','),
      ...rows.map((row) => row.join(',')),
    ].join('\n');

    toast.dismiss();

    if (await exportText(csvContent, 'transactions')) {
      toast.success(t`Transactions exported successfully`);
    }
  } catch (error) {
    console.error('Failed to export transactions:', error);
    toast.dismiss();
    toast.error(t`Failed to export transactions: ${error}`);
  }
}
