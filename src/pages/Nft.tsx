import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { nftUri } from '@/lib/nftUri';
import { open } from '@tauri-apps/plugin-shell';
import { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import { commands, events, NetworkConfig, NftInfo } from '../bindings';

export default function Nft() {
  const { launcher_id: launcherId } = useParams();
  const { addError } = useErrors();

  const [nft, setNft] = useState<NftInfo | null>(null);

  const updateNft = useMemo(
    () => () => {
      commands
        .getNft({ nft_id: launcherId! })
        .then((data) => setNft(data.nft))
        .catch(addError);
    },
    [launcherId, addError],
  );

  useEffect(() => {
    updateNft();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'nft_data'
      ) {
        updateNft();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateNft]);

  const metadata = useMemo(() => {
    if (!nft || !nft.metadata) return {};
    try {
      return JSON.parse(nft.metadata) ?? {};
    } catch {
      return {};
    }
  }, [nft]);

  const [config, setConfig] = useState<NetworkConfig | null>(null);

  useEffect(() => {
    commands.networkConfig().then(setConfig).catch(addError);
  }, [addError]);

  return (
    <>
      <Header title={metadata.name ?? 'Unknown NFT'} />
      <Container>
        <div className='flex flex-col gap-2 mx-auto sm:w-full md:w-[50%] max-w-[400px]'>
          <img
            alt='NFT image'
            src={nftUri(nft?.data_mime_type ?? null, nft?.data ?? null)}
            className='rounded-lg'
          />
          <CopyBox title='Launcher Id' content={nft?.launcher_id ?? ''} />
        </div>

        <div className='my-4 grid grid-cols-1 md:grid-cols-2 gap-y-3 gap-x-10'>
          <div className='flex flex-col gap-3'>
            {metadata.description && (
              <div>
                <h6 className='text-md font-bold'>Description</h6>
                <div className='break-all text-sm'>{metadata.description}</div>
              </div>
            )}

            {nft?.collection_name && (
              <div>
                <h6 className='text-md font-bold'>Collection Name</h6>
                <div className='break-all text-sm cursor-pointer'>
                  {nft.collection_name}
                </div>
              </div>
            )}

            {(metadata.attributes?.length ?? 0) > 0 && (
              <div className='flex flex-col gap-1'>
                <h6 className='text-md font-bold'>Attributes</h6>
                <div className='grid grid-cols-2 gap-2'>
                  {metadata.attributes.map((attr: any, i: number) => (
                    <div key={i} className='px-2 py-1 border-2 rounded-lg'>
                      <h6 className='text-sm font-semibold'>
                        {attr.trait_type}
                      </h6>
                      <div className='text-sm'>{attr.value}</div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {(nft?.data_uris.length || null) && (
              <div>
                <h6 className='text-md font-bold'>Data URIs</h6>
                {nft!.data_uris.map((uri, i) => (
                  <div
                    key={i}
                    className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer'
                    onClick={() => open(uri)}
                  >
                    {uri}
                  </div>
                ))}
              </div>
            )}

            {(nft?.metadata_uris.length || null) && (
              <div>
                <h6 className='text-md font-bold'>Metadata URIs</h6>
                {nft!.metadata_uris.map((uri, i) => (
                  <div
                    key={i}
                    className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer'
                    onClick={() => open(uri)}
                  >
                    {uri}
                  </div>
                ))}
              </div>
            )}

            {(nft?.license_uris.length || null) && (
              <div>
                <h6 className='text-md font-bold'>License URIs</h6>
                {nft!.license_uris.map((uri, i) => (
                  <div
                    key={i}
                    className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer'
                    onClick={() => open(uri)}
                  >
                    {uri}
                  </div>
                ))}
              </div>
            )}

            {nft?.data_hash && (
              <div>
                <h6 className='text-md font-bold'>Data Hash</h6>
                <div className='break-all text-sm'>{nft.data_hash}</div>
              </div>
            )}

            {nft?.metadata_hash && (
              <div>
                <h6 className='text-md font-bold'>Metadata Hash</h6>
                <div className='break-all text-sm'>{nft.metadata_hash}</div>
              </div>
            )}

            {nft?.license_hash && (
              <div>
                <h6 className='text-md font-bold'>License Hash</h6>
                <div className='break-all text-sm'>{nft.license_hash}</div>
              </div>
            )}
          </div>

          <div className='flex flex-col gap-3'>
            <div>
              <h6 className='text-md font-bold'>Minter DID</h6>
              <div className='break-all text-sm'>
                {nft?.minter_did ?? 'None'}
              </div>
            </div>

            <div>
              <h6 className='text-md font-bold'>Owner DID</h6>
              <div className='break-all text-sm'>
                {nft?.owner_did ?? 'None'}
              </div>
            </div>

            <div>
              <h6 className='text-md font-bold'>Address</h6>
              <div className='break-all text-sm'>{nft?.address}</div>
            </div>

            <div>
              <h6 className='text-md font-bold'>Coin Id</h6>
              <div className='break-all text-sm'>{nft?.coin_id}</div>
            </div>

            <div>
              <h6 className='text-md font-bold'>
                Royalties ({(nft?.royalty_ten_thousandths ?? 0) * 100}%)
              </h6>
              <div className='break-all text-sm'>{nft?.royalty_address}</div>
            </div>

            <div className='flex flex-col gap-1'>
              <h6 className='text-md font-bold'>External Links</h6>

              <Button
                variant='outline'
                onClick={() => {
                  open(
                    `https://${config?.network_id !== 'mainnet' ? 'testnet.' : ''}mintgarden.io/nfts/${nft?.launcher_id}`,
                  );
                }}
                disabled={
                  !['mainnet', 'testnet11'].includes(config?.network_id ?? '')
                }
              >
                <img
                  src='https://mintgarden.io/mint-logo.svg'
                  className='h-4 w-4 mr-2'
                  alt='MintGarden logo'
                />
                MintGarden
              </Button>

              <Button
                variant='outline'
                onClick={() => {
                  open(
                    `https://${config?.network_id !== 'mainnet' ? 'testnet11.' : ''}spacescan.io/nft/${nft?.launcher_id}`,
                  );
                }}
                disabled={
                  !['mainnet', 'testnet11'].includes(config?.network_id ?? '')
                }
              >
                <img
                  src='https://spacescan.io/images/spacescan-logo-192.png'
                  className='h-4 w-4 mr-2'
                  alt='Spacescan.io logo'
                />
                Spacescan.io
              </Button>
            </div>
          </div>
        </div>
      </Container>
    </>
  );
}
