import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { isImage, isVideo, isText, isJson, nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { open } from '@tauri-apps/plugin-shell';
import { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  commands,
  events,
  NetworkConfig,
  NftData,
  NftRecord,
} from '../bindings';

export default function Nft() {
  const { launcher_id: launcherId } = useParams();
  const { addError } = useErrors();

  const [nft, setNft] = useState<NftRecord | null>(null);
  const [data, setData] = useState<NftData | null>(null);

  const royaltyPercentage = (nft?.royalty_ten_thousandths ?? 0) / 100;

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

  useEffect(() => {
    commands
      .getNftData({ nft_id: launcherId! })
      .then((response) => setData(response.data))
      .catch(addError);
  }, [launcherId, addError]);

  const metadata = useMemo(() => {
    if (!nft || !data?.metadata_json) return {};
    try {
      return JSON.parse(data.metadata_json) ?? {};
    } catch {
      return {};
    }
  }, [data?.metadata_json, nft]);

  const [config, setConfig] = useState<NetworkConfig | null>(null);

  useEffect(() => {
    commands.networkConfig().then(setConfig).catch(addError);
  }, [addError]);

  // New function to render NFT content based on mime type
  const renderNftContent = () => {
    const uri = nftUri(data?.mime_type ?? null, data?.blob ?? null);

    if (isJson(data?.mime_type ?? null)) {
      try {
        // Parse and format JSON
        const jsonObj = JSON.parse(uri);
        const formattedJson = JSON.stringify(jsonObj, null, 2);
        
        return (
          <div className='relative overflow-auto'>
            <pre
              className='m-0 p-4 whitespace-pre-wrap break-words font-mono text-left'
              style={{
                lineHeight: 1.4,
                fontSize: '14px',
                maxHeight: '80vh',
                backgroundColor: 'white',
                border: '1px solid #eee',
                borderRadius: '4px'
              }}
            >
              {formattedJson}
            </pre>
          </div>
        );
      } catch {
        // If JSON parsing fails, show raw text
        return (
          <div className='relative overflow-auto'>
            <pre className='m-0 p-4 whitespace-pre-wrap break-words font-mono text-left'>
              {uri}
            </pre>
          </div>
        );
      }
    }

    if (isText(data?.mime_type ?? null)) {
      return (
        <div className='relative grid h-full place-items-center overflow-hidden'>
          <pre
            className='m-0 p-[0.1em] whitespace-pre-wrap break-words text-center font-sans'
            style={{
              width: '400px',
              height: '400px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
            ref={(el) => {
              if (el) {
                const { width, height } = el.getBoundingClientRect();
                const columns = width / 16;
                const rows = height / 16;
                el.style.fontSize = `min(${550 / columns}vw, ${550 / rows}vh)`;
              }
            }}
          >
            {uri}
          </pre>
        </div>
      );
    }

    if (isVideo(data?.mime_type ?? null)) {
      return <video src={uri} className='rounded-lg' controls />;
    }

    if (isImage(data?.mime_type ?? null)) {
      return <img alt='NFT image' src={uri} className='rounded-lg' />;
    }

    return null;
  };

  return (
    <>
      <Header title={nft?.name ?? t`Unknown NFT`} />
      <Container>
        <div className='flex flex-col gap-2 mx-auto sm:w-full md:w-[50%] max-w-[400px]'>
          {renderNftContent()}
          <CopyBox title={t`Launcher Id`} value={nft?.launcher_id ?? ''} />
        </div>

        <div className='my-4 grid grid-cols-1 md:grid-cols-2 gap-y-3 gap-x-10'>
          <div className='flex flex-col gap-3'>
            {metadata.description && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>Description</Trans>
                </h6>
                <div className='break-all text-sm'>{metadata.description}</div>
              </div>
            )}

            {nft?.collection_name && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>Collection Name</Trans>
                </h6>
                <div className='break-all text-sm cursor-pointer'>
                  {nft.collection_name}
                </div>
              </div>
            )}

            {(metadata.attributes?.length ?? 0) > 0 && (
              <div className='flex flex-col gap-1'>
                <h6 className='text-md font-bold'>
                  <Trans>Attributes</Trans>
                </h6>
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
                <h6 className='text-md font-bold'>
                  <Trans>Data URIs</Trans>
                </h6>
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
                <h6 className='text-md font-bold'>
                  <Trans>Metadata URIs</Trans>
                </h6>
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
                <h6 className='text-md font-bold'>
                  <Trans>License URIs</Trans>
                </h6>
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
                <h6 className='text-md font-bold'>
                  <Trans>Data Hash</Trans>
                </h6>
                <div className='break-all text-sm'>{nft.data_hash}</div>
              </div>
            )}

            {nft?.metadata_hash && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>Metadata Hash</Trans>
                </h6>
                <div className='break-all text-sm'>{nft.metadata_hash}</div>
              </div>
            )}

            {nft?.license_hash && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>License Hash</Trans>
                </h6>
                <div className='break-all text-sm'>{nft.license_hash}</div>
              </div>
            )}
          </div>

          <div className='flex flex-col gap-3'>
            <div>
              <h6 className='text-md font-bold'>
                <Trans>Minter DID</Trans>
              </h6>
              <div className='break-all text-sm'>
                {nft?.minter_did ?? <Trans>None</Trans>}
              </div>
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Owner DID</Trans>
              </h6>
              <div className='break-all text-sm'>
                {nft?.owner_did ?? <Trans>None</Trans>}
              </div>
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Address</Trans>
              </h6>
              <div className='break-all text-sm'>{nft?.address}</div>
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Coin Id</Trans>
              </h6>
              <div className='break-all text-sm'>{nft?.coin_id}</div>
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Royalties {royaltyPercentage}%</Trans>
              </h6>
              <div className='break-all text-sm'>{nft?.royalty_address}</div>
            </div>

            <div className='flex flex-col gap-1'>
              <h6 className='text-md font-bold'>
                <Trans>External Links</Trans>
              </h6>

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
