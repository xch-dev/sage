import { AddressItem } from '@/components/AddressItem';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useErrors } from '@/hooks/useErrors';
import spacescanLogo from '@/images/spacescan-logo-192.png';
import { getMintGardenProfile } from '@/lib/marketplaces';
import { isAudio, isImage, isJson, isText, nftUri } from '@/lib/nftUri';
import {
  DexieAsset,
  DexieOffer,
  fetchOfferedDexieOffersFromNftId,
  fetchRequestedDexieOffersFromNftId,
} from '@/lib/offerData';
import { formatTimestamp, isValidUrl } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { FileImage, HandCoins, Hash, Tag, Users } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { commands, events, NetworkKind, NftData, NftRecord } from '../bindings';

export default function Nft() {
  const { launcher_id: launcherId } = useParams();
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [nft, setNft] = useState<NftRecord | null>(null);
  const [data, setData] = useState<NftData | null>(null);
  const royaltyPercentage = (nft?.royalty_ten_thousandths ?? 0) / 100;

  const [requestedOffers, setRequestedOffers] = useState<DexieOffer[]>([]);
  const [offeredOffers, setOfferedOffers] = useState<DexieOffer[]>([]);

  // Check for Dexie offers when NFT loads
  useEffect(() => {
    if (nft?.launcher_id) {
      // Fetch both requested and offered offers
      Promise.all([
        fetchRequestedDexieOffersFromNftId(nft.launcher_id),
        fetchOfferedDexieOffersFromNftId(nft.launcher_id),
      ])
        .then(([requested, offered]) => {
          setRequestedOffers(requested);
          setOfferedOffers(offered);
        })
        .catch(() => {
          setRequestedOffers([]);
          setOfferedOffers([]);
        });
    }
  }, [nft?.launcher_id]);

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
        {/* NFT Media Display */}
        <Card className='mb-6'>
          <CardHeader>
            <CardTitle className='flex items-center gap-2'>
              <FileImage className='h-5 w-5' />
              <Trans>NFT Preview</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='flex flex-col md:flex-row gap-6 items-start'>
              <div className='flex-shrink-0 w-full md:w-auto md:max-w-[400px]'>
                {isImage(data?.mime_type ?? null) ? (
                  <img
                    alt='NFT image'
                    src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
                    className='rounded-lg w-full'
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
                    className='rounded-lg w-full'
                    controls
                  />
                )}
              </div>
              <div className='flex-1 min-w-0 space-y-4'>
                <AddressItem
                  label={t`Launcher ID`}
                  address={nft?.launcher_id ?? ''}
                />

                {nft?.edition_total != null && nft?.edition_total > 1 && (
                  <div>
                    <div className='text-sm font-medium text-muted-foreground'>
                      <Trans>Edition</Trans>
                    </div>
                    <div className='text-sm'>
                      <Trans>
                        {nft.edition_number} of {nft.edition_total}
                      </Trans>
                    </div>
                  </div>
                )}

                {metadata.description && (
                  <div>
                    <div className='text-sm font-medium text-muted-foreground'>
                      <Trans>Description</Trans>
                    </div>
                    <div className='break-words text-sm'>
                      {metadata.description}
                    </div>
                  </div>
                )}

                {nft?.collection_name && (
                  <div>
                    <div className='text-sm font-medium text-muted-foreground'>
                      <Trans>Collection</Trans>
                    </div>
                    <div
                      className='break-all text-sm cursor-pointer text-blue-700 dark:text-blue-300 hover:underline'
                      onClick={() =>
                        nft?.collection_id &&
                        navigate(
                          `/nfts/collections/${nft.collection_id}/metadata`,
                        )
                      }
                    >
                      {nft.collection_name}
                    </div>
                  </div>
                )}

                {nft?.created_timestamp && (
                  <div>
                    <div className='text-sm font-medium text-muted-foreground'>
                      <Trans>Created</Trans>
                    </div>
                    <div className='text-sm'>
                      {formatTimestamp(nft.created_timestamp, 'short', 'short')}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        <div className='grid grid-cols-1 lg:grid-cols-2 gap-6'>
          {/* Left Column */}
          <div className='space-y-6'>
            {/* Attributes */}
            {(metadata.attributes?.length ?? 0) > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className='flex items-center gap-2'>
                    <Tag className='h-5 w-5' />
                    <Trans>Attributes</Trans>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className='grid grid-cols-2 gap-3'>
                    {metadata.attributes.map(
                      (attr: { trait_type: string; value: string }) => (
                        <div
                          key={`${attr?.trait_type}_${attr?.value}`}
                          className='px-3 py-2 border rounded-lg '
                        >
                          <div
                            className='text-sm font-medium text-muted-foreground truncate'
                            title={attr.trait_type}
                          >
                            {attr.trait_type}
                          </div>
                          {isValidUrl(attr.value) ? (
                            <div
                              onClick={() => openUrl(attr.value)}
                              className='text-sm break-all text-blue-700 dark:text-blue-300 cursor-pointer hover:underline'
                            >
                              {attr.value}
                            </div>
                          ) : (
                            <div className='text-sm break-all'>
                              {attr.value}
                            </div>
                          )}
                        </div>
                      ),
                    )}
                  </div>
                </CardContent>
              </Card>
            )}

            {/* URIs and Hashes */}
            {(!!nft?.data_uris.length ||
              !!nft?.metadata_uris.length ||
              !!nft?.license_uris.length ||
              nft?.data_hash ||
              nft?.metadata_hash ||
              nft?.license_hash) && (
              <Card>
                <CardHeader>
                  <CardTitle className='flex items-center gap-2'>
                    <Hash className='h-5 w-5' />
                    <Trans>Data and License Details</Trans>
                  </CardTitle>
                </CardHeader>
                <CardContent className='space-y-4'>
                  {!!nft?.data_uris.length && (
                    <div>
                      <div className='text-sm font-medium text-muted-foreground'>
                        <Trans>Data URIs</Trans>
                      </div>
                      <div className=''>
                        {nft.data_uris.map((uri) => (
                          <div
                            key={uri}
                            className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer hover:underline'
                            onClick={() => openUrl(uri)}
                          >
                            {uri}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {!!nft?.metadata_uris.length && (
                    <div>
                      <div className='text-sm font-medium text-muted-foreground'>
                        <Trans>Metadata URIs</Trans>
                      </div>
                      <div className=''>
                        {nft.metadata_uris.map((uri) => (
                          <div
                            key={uri}
                            className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer hover:underline'
                            onClick={() => openUrl(uri)}
                          >
                            {uri}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {!!nft?.license_uris.length && (
                    <div>
                      <div className='text-sm font-medium text-muted-foreground'>
                        <Trans>License URIs</Trans>
                      </div>
                      <div className=''>
                        {nft.license_uris.map((uri) => (
                          <div
                            key={uri}
                            className='truncate text-sm text-blue-700 dark:text-blue-300 cursor-pointer hover:underline'
                            onClick={() => openUrl(uri)}
                          >
                            {uri}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {nft?.data_hash && (
                    <div>
                      <div className='text-sm font-medium text-muted-foreground'>
                        <Trans>Data Hash</Trans>
                      </div>
                      <div className='break-all text-sm font-mono'>
                        {nft.data_hash}
                      </div>
                    </div>
                  )}

                  {nft?.metadata_hash && (
                    <div>
                      <div className='text-sm font-medium text-muted-foreground'>
                        <Trans>Metadata Hash</Trans>
                      </div>
                      <div className='break-all text-sm font-mono'>
                        {nft.metadata_hash}
                      </div>
                    </div>
                  )}

                  {nft?.license_hash && (
                    <div>
                      <div className='text-sm font-medium text-muted-foreground'>
                        <Trans>License Hash</Trans>
                      </div>
                      <div className='break-all text-sm font-mono'>
                        {nft.license_hash}
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            )}
          </div>

          {/* Ownership and Technical Information */}
          <div className='space-y-6'>
            {/* Ownership */}
            <Card>
              <CardHeader>
                <CardTitle className='flex items-center gap-2'>
                  <Users className='h-5 w-5' />
                  <Trans>Ownership</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent className='space-y-4'>
                <div className='space-y-2'>
                  <AddressItem
                    label={t`Minter DID`}
                    address={nft?.minter_did ?? ''}
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

                <div className='space-y-2'>
                  <AddressItem
                    label={t`Owner DID`}
                    address={nft?.owner_did ?? ''}
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

                <AddressItem
                  label={t`Royalties ${royaltyPercentage}%`}
                  address={nft?.royalty_address ?? ''}
                />
              </CardContent>
            </Card>

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

          <div className='flex flex-col gap-4'>
            {/* Requested Offers Section */}
            <div className='flex flex-col gap-1'>
              <h6 className='text-md font-bold'>
                <Trans>Offers Requesting This NFT</Trans>
              </h6>

              {requestedOffers.length === 0 ? (
                <div className='text-sm text-muted-foreground'>
                  <Trans>No offers requesting this NFT</Trans>
                </div>
              ) : (
                <div className='grid gap-2'>
                  {requestedOffers.map((offer: DexieOffer) => (
                    <div key={offer.id} className='border rounded-lg p-3'>
                      <div className='grid grid-cols-2 gap-4'>
                        <div>
                          <div className='text-sm font-medium mb-2'>
                            <Trans>Offered in exchange:</Trans>
                          </div>
                          <div className='space-y-1'>
                            {offer.offered?.map((item: DexieAsset) => (
                              <div key={item.id} className='text-sm'>
                                {item.amount} {item.name} ({item.code})
                              </div>
                            ))}
                          </div>
                        </div>
                        <div className='flex flex-col gap-1 justify-start'>
                          <Button
                            variant='outline'
                            size='sm'
                            onClick={() => {
                              navigate(
                                `/offers/view/${encodeURIComponent(offer.offer.trim())}`,
                              );
                            }}
                          >
                            <HandCoins className='h-4 w-4 mr-2' />
                            <Trans>View Offer</Trans>
                          </Button>
                          <Button
                            variant='outline'
                            size='sm'
                            onClick={() => {
                              openUrl(`https://dexie.space/offers/${offer.id}`);
                            }}
                          >
                            <img
                              src='https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg'
                              className='h-4 w-4 mr-2'
                              alt='Dexie.space logo'
                            />
                            <Trans>View Offer on Dexie</Trans>
                          </Button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Offered For Sale Section */}
            <div className='flex flex-col gap-1'>
              <h6 className='text-md font-bold'>
                <Trans>This NFT Offered For Sale</Trans>
              </h6>

              {offeredOffers.length === 0 ? (
                <div className='text-sm text-muted-foreground'>
                  <Trans>This NFT is not currently offered for sale</Trans>
                </div>
              ) : (
                <div className='grid gap-2'>
                  {offeredOffers.map((offer: DexieOffer) => (
                    <div key={offer.id} className='border rounded-lg p-3'>
                      <div className='grid grid-cols-2 gap-4'>
                        <div>
                          <div className='text-sm font-medium mb-2'>
                            <Trans>Requesting in exchange:</Trans>
                          </div>
                          <div className='space-y-1'>
                            {offer.requested?.map((item: DexieAsset) => (
                              <div key={item.id} className='text-sm'>
                                {item.amount} {item.name} ({item.code})
                              </div>
                            ))}
                          </div>
                        </div>
                        <div className='flex flex-col gap-1 justify-start'>
                          <Button
                            variant='outline'
                            size='sm'
                            onClick={() => {
                              openUrl(`https://dexie.space/offers/${offer.id}`);
                            }}
                          >
                            <img
                              src='https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg'
                              className='h-4 w-4 mr-2'
                              alt='Dexie.space logo'
                            />
                            <Trans>View Offer on Dexie</Trans>
                          </Button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </Container>
    </>
  );
}
