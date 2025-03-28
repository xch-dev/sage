import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import { NftRecord } from '@/bindings';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

export async function exportNfts(
    currentNfts: NftRecord[],
    fetchAllNfts: () => Promise<NftRecord[]>,
) {
    try {
        if (currentNfts.length === 0) {
            toast.error(t`No NFTs to export`);
            return;
        }

        toast.info(t`Fetching NFTs...`, { autoClose: 30000 });

        // Fetch all NFTs
        const allNfts = await fetchAllNfts();

        if (allNfts.length === 0) {
            toast.error(t`No NFTs found to export`);
            return;
        }

        // If we're in a filtered view, only export the current NFTs
        const nftsToExport = currentNfts.length < allNfts.length ? currentNfts : allNfts;

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

        const rows = nftsToExport.map((nft) => [
            (nft.name || '').replace(/,/g, ''),
            (nft.collection_name || '').replace(/,/g, ''),
            (nft.collection_id),
            (nft.owner_did || ''),
            (nft.minter_did || ''),
            (nft.launcher_id || ''),
            (nft.coin_id || ''),
            nft.created_height?.toString() || '',
            (nft.data_uris || []).join(';').replace(/,/g, ''),
            (nft.data_hash || ''),
            nft.edition_number?.toString() || '',
            nft.edition_total?.toString() || '',
            (nft.license_uris || []).join(';').replace(/,/g, ''),
            (nft.license_hash || ''),
            (nft.metadata_uris || []).join(';').replace(/,/g, ''),
            (nft.metadata_hash || ''),
            (nft.royalty_ten_thousandths / 100).toString() + '%',
            (nft.royalty_address || ''),
            (nft.address || ''),
            nft.sensitive_content ? 'true' : 'false',
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
            defaultPath: 'nfts.csv',
        });

        if (filePath) {
            await writeTextFile(filePath, csvContent);
            toast.success(t`NFTs exported successfully`);
        }
    } catch (error) {
        console.error('Failed to export NFTs:', error);
        toast.dismiss();
        toast.error(t`Failed to export NFTs: ${error}`);
    }
} 