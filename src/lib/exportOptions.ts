import { OptionRecord } from '@/bindings';
import { exportText } from '@/lib/exportText';
import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

export async function exportOptions(options: OptionRecord[]) {
  try {
    if (options.length === 0) {
      toast.error(t`No options to export`);
      return;
    }

    // Create CSV content
    const headers = [
      'Name',
      'Option ID',
      'Underlying Asset',
      'Underlying Amount',
      'Strike Asset',
      'Strike Amount',
      'Expiration',
      'Status',
      'Visible',
      'Created Height',
    ];

    const rows = options.map((option) => [
      (option.name || 'Untitled Option').replace(/,/g, ''),
      option.launcher_id,
      (
        option.underlying_asset.name ||
        option.underlying_asset.ticker ||
        'Unknown'
      ).replace(/,/g, ''),
      option.underlying_asset.kind === 'token'
        ? fromMojos(
            option.underlying_amount,
            option.underlying_asset.precision,
          ).toString()
        : '',
      (
        option.strike_asset.name ||
        option.strike_asset.ticker ||
        'Unknown'
      ).replace(/,/g, ''),
      option.strike_asset.kind === 'token'
        ? fromMojos(
            option.strike_amount,
            option.strike_asset.precision,
          ).toString()
        : '',
      formatTimestamp(option.expiration_seconds, 'short', 'short').replace(
        /,/g,
        '',
      ),
      option.created_height === null ? 'Pending' : 'Active',
      option.visible ? 'Yes' : 'No',
      option.created_height?.toString() || '',
    ]);

    const csvContent = [
      headers.join(','),
      ...rows.map((row) => row.join(',')),
    ].join('\n');

    toast.dismiss();

    if (await exportText(csvContent, 'options')) {
      toast.success(t`Options exported successfully`);
    }
  } catch (error) {
    console.error('Failed to export options:', error);
    toast.dismiss();
    toast.error(t`Failed to export options: ${error}`);
  }
}
