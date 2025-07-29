import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import spacescanLogo from '@/images/spacescan-logo-192.png';
import { getMintGardenProfile } from '@/lib/marketplaces';
import { isAudio, isImage, isJson, isText, nftUri } from '@/lib/nftUri';
import { isValidUrl } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands, events, NetworkKind, NftData, NftRecord } from '../bindings';

export default function Nft() {
  const { launcher_id: launcherId } = useParams();
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [nft, setNft] = useState<NftRecord | null>(null);
  const [data, setData] = useState<NftData | null>(null);
  const royaltyPercentage = (nft?.royalty_ten_thousandths ?? 0) / 100;

  const updateNft = useMemo(
    () => () => {
      commands
        .getNft({ nft_id: launcherId ?? '' })
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
      .getNftData({ nft_id: launcherId ?? '' })
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

  const [network, setNetwork] = useState<NetworkKind | null>(null);

  useEffect(() => {
    commands
      .getNetwork({})
      .then((data) => setNetwork(data.kind))
      .catch(addError);
  }, [addError]);

  const [minterProfile, setMinterProfile] = useState<{
    encoded_id: string;
    name: string;
    avatar_uri: string | null;
  } | null>(null);

  const [ownerProfile, setOwnerProfile] = useState<{
    encoded_id: string;
    name: string;
    avatar_uri: string | null;
  } | null>(null);

  useEffect(() => {
    if (!nft?.minter_did) {
      setMinterProfile(null);
      return;
    }

    getMintGardenProfile(nft.minter_did).then(setMinterProfile);
  }, [nft?.minter_did]);

  useEffect(() => {
    if (!nft?.owner_did) {
      setOwnerProfile(null);
      return;
    }

    getMintGardenProfile(nft.owner_did).then(setOwnerProfile);
  }, [nft?.owner_did]);

  return (
    <>
      <Header title={nft?.name ?? t`Unknown NFT`} />
      <Container>
        <div className='flex flex-col gap-2 mx-auto sm:w-full md:w-[50%] max-w-[400px]'>
          {isImage(data?.mime_type ?? null) ? (
            <img
              alt='NFT image'
              src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
              className='rounded-lg'
            />
          ) : isText(data?.mime_type ?? null) ? (
            <div className='border rounded-lg p-4 bg-gray-50 dark:bg-gray-800 overflow-auto max-h-[400px]'>
              <pre className='whitespace-pre-wrap text-sm'>
                {data?.blob ? atob(data.blob) : ''}
              </pre>
            </div>
          ) : isJson(data?.mime_type ?? null) ? (
            <div className='border rounded-lg p-4 bg-gray-50 dark:bg-gray-800 overflow-auto max-h-[400px]'>
              <pre className='whitespace-pre-wrap text-sm'>
                {data?.blob
                  ? JSON.stringify(JSON.parse(atob(data.blob)), null, 2)
                  : ''}
              </pre>
            </div>
          ) : isAudio(data?.mime_type ?? null) ? (
            <div className='flex flex-col items-center justify-center p-4 border rounded-lg bg-gray-50 dark:bg-gray-800'>
              <div className='text-4xl mb-2'>🎵</div>
              <audio
                src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
                controls
                className='w-full'
              />
            </div>
          ) : (
            <video
              src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
              className='rounded-lg'
              controls
            />
          )}
          <CopyBox
            title={t`Launcher Id`}
            value={nft?.launcher_id ?? ''}
            onCopy={() => toast.success(t`Launcher Id copied to clipboard`)}
          />
        </div>

        <div className='my-4 grid grid-cols-1 md:grid-cols-2 gap-y-3 gap-x-10'>
          <div className='flex flex-col gap-3'>
            {nft?.edition_total != null && nft?.edition_total > 1 && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>
                    Edition {nft.edition_number} of {nft.edition_total}
                  </Trans>
                </h6>
              </div>
            )}

            {metadata.description && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>Description</Trans>
                </h6>
                <div className='break-words text-sm'>
                  {metadata.description}
                </div>
              </div>
            )}

            {nft?.collection_name && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>Collection Name</Trans>
                </h6>
                <div
                  className='break-all text-sm cursor-pointer text-blue-700 dark:text-blue-300 hover:underline'
                  onClick={() =>
                    nft?.collection_id &&
                    navigate(`/nfts/collections/${nft.collection_id}/metadata`)
                  }
                >
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
                  {metadata.attributes.map(
                    (attr: { trait_type: string; value: string }) => (
                      <div
                        key={`${attr?.trait_type}_${attr?.value}`}
                        className='px-2 py-1 border-2 rounded-lg'
                      >
                        <h6
                          className='text-sm font-semibold truncate'
                          title={attr.trait_type}
                        >
                          {attr.trait_type}
                        </h6>
                        {isValidUrl(attr.value) ? (
                          <div
                            onClick={() => openUrl(attr.value)}
                            className='text-sm break-all text-blue-700 dark:text-blue-300 cursor-pointer hover:underline'
                          >
                            {attr.value}
                          </div>
                        ) : (
                          <div className='text-sm break-all'>{attr.value}</div>
                        )}
                      </div>
                    ),
                  )}
                </div>
              </div>
            )}

            {!!nft?.data_uris.length && (
              <div>
                <h6 className='text-md font-bold'>
                  <Trans>Data URIs</Trans>
                </h6>
                {nft.data_uris.map((uri) => (
                  <div
                    key={uri}
                    className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer'
                    onClick={() => openUrl(uri)}
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
                {nft?.metadata_uris.map((uri) => (
                  <div
                    key={uri}
                    className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer'
                    onClick={() => openUrl(uri)}
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
                {nft?.license_uris.map((uri) => (
                  <div
                    key={uri}
                    className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer'
                    onClick={() => openUrl(uri)}
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
              <CopyBox
                title={t`Minter DID`}
                value={nft?.minter_did ?? t`None`}
                onCopy={() => toast.success(t`Minter DID copied to clipboard`)}
              />
              {minterProfile && (
                <div
                  className='flex items-center gap-2 mt-1 cursor-pointer text-blue-700 dark:text-blue-300 hover:underline'
                  onClick={() =>
                    openUrl(`https://mintgarden.io/${nft?.minter_did}`)
                  }
                >
                  {minterProfile.avatar_uri && (
                    <img
                      src={minterProfile.avatar_uri}
                      alt={`${minterProfile.name} avatar`}
                      className='w-6 h-6 rounded-full'
                    />
                  )}
                  <div className='text-sm'>{minterProfile.name}</div>
                </div>
              )}
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Owner DID</Trans>
              </h6>
              <CopyBox
                title={t`Owner DID`}
                value={nft?.owner_did ?? t`None`}
                onCopy={() => toast.success(t`Owner DID copied to clipboard`)}
              />
              {ownerProfile && (
                <div
                  className='flex items-center gap-2 mt-1 cursor-pointer text-blue-700 dark:text-blue-300 hover:underline'
                  onClick={() =>
                    openUrl(`https://mintgarden.io/${nft?.owner_did}`)
                  }
                >
                  {ownerProfile.avatar_uri && (
                    <img
                      src={ownerProfile.avatar_uri}
                      alt={`${ownerProfile.name} avatar`}
                      className='w-6 h-6 rounded-full'
                    />
                  )}
                  <div className='text-sm'>{ownerProfile.name}</div>
                </div>
              )}
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Address</Trans>
              </h6>
              <CopyBox
                title={t`Address`}
                value={nft?.address ?? ''}
                onCopy={() => toast.success(t`Address copied to clipboard`)}
              />
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Coin Id</Trans>
              </h6>
              <CopyBox
                title={t`Coin Id`}
                value={nft?.coin_id ?? ''}
                onCopy={() => toast.success(t`Coin ID copied to clipboard`)}
              />
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Royalties {royaltyPercentage}%</Trans>
              </h6>
              <CopyBox
                title={t`Royalty Address`}
                value={nft?.royalty_address ?? ''}
                onCopy={() =>
                  toast.success(t`Royalty address copied to clipboard`)
                }
              />
            </div>

            <div className='flex flex-col gap-1'>
              <h6 className='text-md font-bold'>
                <Trans>External Links</Trans>
              </h6>

              <Button
                variant='outline'
                onClick={() => {
                  openUrl(
                    `https://${network === 'testnet' ? 'testnet.' : ''}mintgarden.io/nfts/${nft?.launcher_id}`,
                  );
                }}
                disabled={network === 'unknown'}
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
                  openUrl(
                    `https://${network === 'testnet' ? 'testnet11.' : ''}spacescan.io/nft/${nft?.launcher_id}`,
                  );
                }}
                disabled={network === 'unknown'}
              >
                <img
                  src={spacescanLogo}
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
