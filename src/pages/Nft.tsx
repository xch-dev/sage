import { AddressItem } from '@/components/AddressItem';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { LabeledItem } from '@/components/LabeledItem';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useTheme } from '@/contexts/ThemeContext';
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
import { formatTimestamp, getOfferStatus } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { FileImage, FileText, HandCoins, Hash, Tag, Users } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  commands,
  events,
  NetworkKind,
  NftData,
  NftRecord,
  OfferRecord,
} from '../bindings';
import { useTheme } from 'theme-o-rama';

export default function Nft() {
  const navigate = useNavigate();
  const { launcher_id: launcherId } = useParams();
  const { addError } = useErrors();
  const { reloadThemes } = useTheme();
  const [nft, setNft] = useState<NftRecord | null>(null);
  const [nftIsOwned, setNftIsOwned] = useState<boolean>(false);
  const [data, setData] = useState<NftData | null>(null);
  const [themeExists, setThemeExists] = useState<boolean>(false);
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const royaltyPercentage = (nft?.royalty_ten_thousandths ?? 0) / 100;

  const [requestedOffers, setRequestedOffers] = useState<DexieOffer[]>([]);
  const [offeredOffers, setOfferedOffers] = useState<DexieOffer[]>([]);
  const [offersForAsset, setOffersForAsset] = useState<OfferRecord[]>([]);

  // Check for open offers when NFT loads
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

      commands
        .getOffersForAsset({ asset_id: nft.launcher_id })
        .then((response) => {
          setOffersForAsset(response.offers);
        });
    }
  }, [nft?.launcher_id]);

  const checkThemeExists = useCallback(async () => {
    if (launcherId && nft?.special_use_type === 'theme') {
      try {
        const response = await commands.getUserTheme({ nft_id: launcherId });
        setThemeExists(response.theme !== null);
      } catch {
        setThemeExists(false);
      }
    }
  }, [launcherId, nft?.special_use_type]);

  const updateNft = useMemo(
    () => () => {
      if (launcherId) {
        commands
          .getNft({ nft_id: launcherId })
          .then((data) => setNft(data.nft))
          .catch(addError);
        commands
          .isAssetOwned({ asset_id: launcherId })
          .then((data) => setNftIsOwned(data.owned))
          .catch(addError);
      }
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
    checkThemeExists();
  }, [checkThemeExists]);

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
        <Card className='mb-4'>
          <CardHeader>
            <CardTitle className='flex items-center gap-2'>
              <FileImage className='h-5 w-5' />
              <Trans>NFT Preview</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='flex flex-col md:flex-row gap-6 items-start'>
              <div className='w-full md:w-auto md:max-w-[280px] lg:max-w-[350px] xl:max-w-[400px]'>
                {isImage(data?.mime_type ?? null) ? (
                  <img
                    alt='NFT image'
                    src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
                    className='rounded-lg w-full'
                  />
                ) : isText(data?.mime_type ?? null) ? (
                  <div className='border border-border rounded-lg p-4 bg-muted overflow-auto max-h-[400px]'>
                    <pre className='whitespace-pre-wrap text-sm'>
                      {data?.blob ? atob(data.blob) : ''}
                    </pre>
                  </div>
                ) : isJson(data?.mime_type ?? null) ? (
                  <div className='border border-border rounded-lg p-4 bg-muted overflow-auto max-h-[400px]'>
                    <pre className='whitespace-pre-wrap text-sm'>
                      {data?.blob
                        ? JSON.stringify(JSON.parse(atob(data.blob)), null, 2)
                        : ''}
                    </pre>
                  </div>
                ) : isAudio(data?.mime_type ?? null) ? (
                  <div className='flex flex-col items-center justify-center p-4 border border-border rounded-lg bg-muted'>
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

                {nft?.special_use_type === 'theme' && nftIsOwned && (
                  <Button
                    variant='outline'
                    className='w-full mt-3'
                    disabled={themeExists || isSaving}
                    onClick={async () => {
                      if (nft?.launcher_id) {
                        setIsSaving(true);
                        try {
                          await commands.saveUserTheme({
                            nft_id: nft.launcher_id,
                          });
                          await checkThemeExists();
                          await reloadThemes();
                        } catch (error) {
                          addError({
                            kind: 'internal',
                            reason:
                              error instanceof Error
                                ? error.message
                                : 'Unknown error occurred',
                          });
                        } finally {
                          setIsSaving(false);
                        }
                      }
                    }}
                  >
                    <Trans>
                      {themeExists
                        ? t`Theme Saved`
                        : isSaving
                          ? t`Saving...`
                          : t`Save Theme`}
                    </Trans>
                  </Button>
                )}
              </div>

              <div className='flex-1 min-w-0 space-y-3'>
                <AddressItem
                  label={t`Launcher ID`}
                  address={nft?.launcher_id ?? ''}
                />

                {nft?.edition_total != null && nft?.edition_total > 1 && (
                  <LabeledItem
                    label={t`Edition`}
                    content={`${nft.edition_number} of ${nft.edition_total}`}
                  />
                )}

                <LabeledItem
                  label={t`Description`}
                  content={metadata?.description}
                />

                <LabeledItem
                  label={t`Collection`}
                  content={nft?.collection_name}
                  onClick={() => {
                    if (nft?.collection_id) {
                      navigate(
                        `/nfts/collections/${nft.collection_id}/metadata`,
                      );
                    }
                  }}
                />

                {nft?.created_timestamp && (
                  <LabeledItem
                    label={t`Created`}
                    content={formatTimestamp(
                      nft.created_timestamp,
                      'short',
                      'short',
                    )}
                  />
                )}

                <LabeledItem label={t`External Links`} content={null}>
                  <Button
                    variant='outline'
                    className='w-full'
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
                      aria-hidden='true'
                    />
                    View on MintGarden
                  </Button>

                  <Button
                    variant='outline'
                    className='w-full mt-1'
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
                      aria-hidden='true'
                    />
                    <Trans>View on Spacescan.io</Trans>
                  </Button>
                </LabeledItem>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
          {(metadata.attributes?.length ?? 0) > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className='flex items-center gap-2'>
                  <Tag className='h-5 w-5' aria-hidden='true' />
                  <Trans>Attributes</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className='grid grid-cols-2 gap-2'>
                  {metadata.attributes.map(
                    (attr: { trait_type: string; value: string }) => (
                      <div
                        key={`${attr?.trait_type}_${attr?.value}`}
                        className='px-3 py-2 border rounded-lg '
                      >
                        <LabeledItem
                          label={attr.trait_type}
                          content={attr.value}
                          className='text-sm truncate'
                        />
                      </div>
                    ),
                  )}
                </div>
              </CardContent>
            </Card>
          )}

          <Card>
            <CardHeader>
              <CardTitle className='flex items-center gap-2'>
                <Users className='h-5 w-5' aria-hidden='true' />
                <Trans>Ownership</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent className='space-y-3'>
              <div className='space-y-2'>
                <AddressItem
                  label={t`Minter DID`}
                  address={nft?.minter_did ?? ''}
                />
                {minterProfile && (
                  <div
                    className='flex items-center gap-2 mt-1 cursor-pointer text-blue-600 hover:text-blue-800 hover:underline'
                    onClick={() =>
                      openUrl(`https://mintgarden.io/${nft?.minter_did}`)
                    }
                  >
                    {minterProfile.avatar_uri && (
                      <img
                        src={minterProfile.avatar_uri}
                        alt={`${minterProfile.name} avatar`}
                        className='w-6 h-6 rounded-full'
                        aria-hidden='true'
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
                    className='flex items-center gap-2 mt-1 cursor-pointer text-blue-600 hover:text-blue-800 hover:underline'
                    onClick={() =>
                      openUrl(`https://mintgarden.io/${nft?.owner_did}`)
                    }
                  >
                    {ownerProfile.avatar_uri && (
                      <img
                        src={ownerProfile.avatar_uri}
                        alt={`${ownerProfile.name} avatar`}
                        className='w-6 h-6 rounded-full'
                        aria-hidden='true'
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
              <div className='flex flex-col gap-4'>
                {/* Requested Offers Section */}
                <div className='flex flex-col gap-1'>
                  <h6 className='text-md font-bold'>
                    <Trans>Offers Requesting This NFT</Trans>
                  </h6>

                  {requestedOffers.length === 0 ? (
                    <div className='text-sm text-muted-foreground'>
                      <Trans>No Dexie offers requesting this NFT</Trans>
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
                                  openUrl(
                                    `https://dexie.space/offers/${offer.id}`,
                                  );
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
                      <Trans>
                        This NFT is not currently offered for sale on Dexie
                      </Trans>
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
                                  openUrl(
                                    `https://dexie.space/offers/${offer.id}`,
                                  );
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

                {/* Local Offers Section */}
                {offersForAsset.length > 0 && (
                  <div className='flex flex-col gap-1'>
                    <div className='grid gap-2'>
                      {offersForAsset.map((localOffer) => (
                        <div
                          key={localOffer.offer_id}
                          className='border rounded-lg p-3'
                        >
                          <div className='grid grid-cols-2 gap-4'>
                            <div>
                              <div className='text-sm font-medium mb-2'>
                                <Trans>Local Offer</Trans>
                              </div>
                              <div className='text-sm text-muted-foreground'>
                                <Trans>
                                  Status: {getOfferStatus(localOffer.status)}
                                </Trans>
                              </div>
                              {localOffer.creation_timestamp && (
                                <div className='text-sm text-muted-foreground'>
                                  <Trans>
                                    Created:{' '}
                                    {formatTimestamp(
                                      localOffer.creation_timestamp,
                                      'short',
                                      'short',
                                    )}
                                  </Trans>
                                </div>
                              )}
                            </div>
                            <div className='flex flex-col gap-1 justify-start'>
                              <Button
                                variant='outline'
                                size='sm'
                                onClick={() => {
                                  navigate(
                                    `/offers/view_saved/${localOffer.offer_id}`,
                                  );
                                }}
                              >
                                <Trans>View Local Offer</Trans>
                              </Button>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className='flex items-center gap-2'>
                <FileText className='h-5 w-5' aria-hidden='true' />
                <Trans>Technical Information</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent className='space-y-3'>
              <AddressItem label={t`Address`} address={nft?.address ?? ''} />
              <AddressItem label={t`Coin ID`} address={nft?.coin_id ?? ''} />
            </CardContent>
          </Card>

          {(!!nft?.data_uris.length ||
            !!nft?.metadata_uris.length ||
            !!nft?.license_uris.length ||
            nft?.data_hash ||
            nft?.metadata_hash ||
            nft?.license_hash) && (
            <Card>
              <CardHeader>
                <CardTitle className='flex items-center gap-2'>
                  <Hash className='h-5 w-5' aria-hidden='true' />
                  <Trans>Data and License Details</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent className='space-y-3'>
                {!!nft?.data_uris.length && (
                  <LabeledItem label={t`Data URIs`} content={null}>
                    <>
                      {nft.data_uris.map((uri) => (
                        <div
                          key={uri}
                          className='truncate text-sm text-blue-600 hover:text-blue-800 cursor-pointer hover:underline'
                          onClick={() => openUrl(uri)}
                        >
                          {uri}
                        </div>
                      ))}
                    </>
                  </LabeledItem>
                )}

                {!!nft?.metadata_uris.length && (
                  <LabeledItem label={t`Metadata URIs`} content={null}>
                    <>
                      {nft.metadata_uris.map((uri) => (
                        <div
                          key={uri}
                          className='truncate text-sm text-blue-600 hover:text-blue-800 cursor-pointer hover:underline'
                          onClick={() => openUrl(uri)}
                        >
                          {uri}
                        </div>
                      ))}
                    </>
                  </LabeledItem>
                )}

                {!!nft?.license_uris.length && (
                  <LabeledItem label={t`License URIs`} content={null}>
                    <>
                      {nft.license_uris.map((uri) => (
                        <div
                          key={uri}
                          className='truncate text-sm text-blue-600 hover:text-blue-800 cursor-pointer hover:underline'
                          onClick={() => openUrl(uri)}
                        >
                          {uri}
                        </div>
                      ))}
                    </>
                  </LabeledItem>
                )}

                <LabeledItem
                  label={t`Data Hash`}
                  content={nft.data_hash}
                  className='font-mono'
                />

                <LabeledItem
                  label={t`Metadata Hash`}
                  content={nft.metadata_hash}
                  className='font-mono'
                />

                <LabeledItem
                  label={t`License Hash`}
                  content={nft.license_hash}
                  className='font-mono'
                />
              </CardContent>
            </Card>
          )}
        </div>
      </Container>
    </>
  );
}
