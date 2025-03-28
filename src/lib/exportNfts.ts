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
        console.log('Current NFTs:', currentNfts.length);
        if (currentNfts.length === 0) {
            toast.error(t`No NFTs to export`);
            return;
        }

        toast.info(t`Fetching NFTs...`, { autoClose: 30000 });

        // Fetch all NFTs
        console.log('Fetching all NFTs...');
        const allNfts = await fetchAllNfts();
        console.log('Fetched NFTs:', allNfts.length);

        if (allNfts.length === 0) {
            toast.error(t`No NFTs found to export`);
            return;
        }

        // If we're in a filtered view, only export the current NFTs
        const nftsToExport = currentNfts.length < allNfts.length ? currentNfts : allNfts;
        console.log('NFTs to export:', nftsToExport.length);

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
            (nft.license_hash || '').replace(/,/g, ''),
            (nft.metadata_uris || []).join(';').replace(/,/g, ''),
            (nft.metadata_hash || ''),
            (nft.royalty_ten_thousandths / 100).toString() + '%',
            (nft.royalty_address || '').replace(/,/g, ''),
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
            console.log('Writing file to:', filePath);
            await writeTextFile(filePath, csvContent);
            toast.success(t`NFTs exported successfully`);
        } else {
            console.log('Save dialog cancelled');
        }
    } catch (error) {
        console.error('Failed to export NFTs:', error);
        toast.dismiss();
        toast.error(t`Failed to export NFTs: ${error}`);
    }
} 