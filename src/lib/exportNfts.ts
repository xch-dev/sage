import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';
import { commands, NftRecord } from '@/bindings';
import { NftGroupMode, NftSortMode } from '@/hooks/useNftParams';
import { isValidAddress } from '@/lib/utils';
import { exportText } from './exportText';

interface ExportParams {
  sort: NftSortMode;
  group: NftGroupMode;
  showHidden: boolean;
  query: string | null;
  collectionId?: string;
  ownerDid?: string;
  minterDid?: string;
}

// Shared utility function for querying NFTs
export async function queryNfts(params: ExportParams): Promise<NftRecord[]> {
  if (params.query && isValidAddress(params.query, 'nft')) {
    const response = await commands.getNft({ nft_id: params.query });
    return response.nft ? [response.nft] : [];
  }

  const queryParams = {
    name: params.query || null,
    collection_id:
      params.collectionId === 'No collection'
        ? 'none'
        : (params.collectionId ?? null),
    owner_did_id:
      params.ownerDid === 'No did' ? 'none' : (params.ownerDid ?? null),
    minter_did_id:
      params.minterDid === 'No did' ? 'none' : (params.minterDid ?? null),
    offset: 0,
    limit: 1000000, // A large number to get all NFTs
    sort_mode: params.sort,
    include_hidden: params.showHidden,
  };

  const response = await commands.getNfts(queryParams);
  return response.nfts;
}

export async function exportNfts(params: ExportParams) {
  try {
    toast.info(t`Fetching NFTs...`, { autoClose: 30000 });

    const nfts = await queryNfts(params);

    if (nfts.length === 0) {
      toast.error(t`No NFTs to export`);
      return;
    }

    // Create CSV content
    const headers = [
      'Name',
      'Collection',
      'Collection ID',
      'Owner DID',
      'Minter DID',
      'Launcher ID',
      'Coin ID',
      'Created Height',
      'Data URIs',
      'Data Hash',
      'Edition Number',
      'Edition Total',
      'License URIs',
      'License Hash',
      'Metadata URIs',
      'Metadata Hash',
      'Royalty Percentage',
      'Royalty Address',
      'Address',
      'Sensitive Content',
    ];

    const rows = nfts.map((nft) => [
      (nft.name || '').replace(/,/g, ''),
      (nft.collection_name || '').replace(/,/g, ''),
      nft.collection_id,
      nft.owner_did || '',
      nft.minter_did || '',
      nft.launcher_id || '',
      nft.coin_id || '',
      nft.created_height?.toString() || '',
      (nft.data_uris || []).join(';').replace(/,/g, ''),
      nft.data_hash || '',
      nft.edition_number?.toString() || '',
      nft.edition_total?.toString() || '',
      (nft.license_uris || []).join(';').replace(/,/g, ''),
      nft.license_hash || '',
      (nft.metadata_uris || []).join(';').replace(/,/g, ''),
      nft.metadata_hash || '',
      (nft.royalty_ten_thousandths / 100).toString() + '%',
      nft.royalty_address || '',
      nft.address || '',
      nft.sensitive_content ? 'true' : 'false',
    ]);

    const csvContent = [
      headers.join(','),
      ...rows.map((row) => row.join(',')),
    ].join('\n');

    toast.dismiss();

    if (await exportText(csvContent, 'nfts')) {
      toast.success(t`NFTs exported successfully`);
    }
  } catch (error) {
    console.error('Failed to export NFTs:', error);
    toast.dismiss();
    toast.error(t`Failed to export NFTs: ${error}`);
  }
}
