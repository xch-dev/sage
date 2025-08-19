import { commands, NetworkKind, NftCollectionRecord } from '@/bindings';
import { AddressItem } from '@/components/AddressItem';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { LabeledItem } from '@/components/LabeledItem';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import missing from '@/images/missing.png';
import spacescanLogo from '@/images/spacescan-logo-192.png';
import { getMintGardenProfile } from '@/lib/marketplaces';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ExternalLink, FileImage, Info, Tag } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';

interface MetadataContent {
  collection?: {
    attributes: AttributeType[];
  };
}

interface AttributeType {
  type: string;
  value: string;
}

export default function CollectionMetaData() {
  const { collection_id } = useParams();
  const { addError } = useErrors();
  const [collection, setCollection] = useState<NftCollectionRecord | null>(
    null,
  );
  const [metadataContent, setMetadataContent] =
    useState<MetadataContent | null>(null);
  const [loading, setLoading] = useState(true);
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [minterProfile, setMinterProfile] = useState<{
    encoded_id: string;
    name: string;
    avatar_uri: string | null;
  } | null>(null);

  useEffect(() => {
    async function fetchData() {
      if (!collection_id) return;

      try {
        // Fetch collection data
        const collectionResponse = await commands.getNftCollection({
          collection_id: collection_id,
        });

        if (collectionResponse.collection) {
          setCollection(collectionResponse.collection);

          // Fetch first NFT in the collection
          const nftsResponse = await commands.getNfts({
            collection_id: collection_id,
            offset: 0,
            limit: 1,
            sort_mode: 'name',
            include_hidden: true,
          });

          if (nftsResponse.nfts.length > 0) {
            const nft = nftsResponse.nfts[0];

            // Find first HTTPS metadata URI
            const httpsUri = nft.metadata_uris.find((uri) =>
              uri.startsWith('https://'),
            );
            if (httpsUri) {
              try {
                const response = await fetch(httpsUri);
                const json = await response.json();
                setMetadataContent(json);
              } catch (error) {
                console.error('Failed to fetch metadata content:', error);
              }
            }
          }
        }
      } catch (error: unknown) {
        addError(error as CustomError);
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, [collection_id, addError]);

  useEffect(() => {
    if (!collection?.did_id) {
      setMinterProfile(null);
      return;
    }

    getMintGardenProfile(collection.did_id).then(setMinterProfile);
  }, [collection?.did_id]);

  useEffect(() => {
    commands
      .getNetwork({})
      .then((data) => setNetwork(data.kind))
      .catch(addError);
  }, [addError]);

  // Find banner URL from attributes if it exists
  const getBannerUrl = () => {
    if (!metadataContent?.collection) return null;
    const attributes = metadataContent.collection.attributes;
    if (!Array.isArray(attributes)) return null;

    const bannerAttr = attributes.find(
      (attr) =>
        typeof attr === 'object' &&
        attr !== null &&
        'type' in attr &&
        'value' in attr &&
        attr.type === 'banner' &&
        typeof attr.value === 'string',
    );

    return bannerAttr?.value || null;
  };

  if (loading) {
    return (
      <>
        <Header title={t`Loading Collection...`} />
        <Container>
          <div className='animate-pulse'>
            <div className='h-32 w-32 bg-neutral-200 dark:bg-neutral-800 rounded-lg mx-auto mb-4' />
            <div className='h-8 bg-neutral-200 dark:bg-neutral-800 rounded w-3/4 mx-auto mb-4' />
            <div className='h-4 bg-neutral-200 dark:bg-neutral-800 rounded w-1/2 mx-auto' />
          </div>
        </Container>
      </>
    );
  }

  if (!collection) {
    return (
      <>
        <Header title={t`Collection Not Found`} />
        <Container>
          <div className='text-center'>
            <h2 className='text-xl font-semibold mb-2'>
              <Trans>No Collection Found</Trans>
            </h2>
            <p>
              <Trans>This collection ID does not exist.</Trans>
            </p>
          </div>
        </Container>
      </>
    );
  }

  const collectionName = collection.name || t`Unnamed Collection`;

  return (
    <>
      <Header title={collection?.name ?? t`Unknown Collection`} />
      <Container>
        <Card className='mb-4'>
          <CardHeader>
            <CardTitle className='flex items-center gap-2'>
              <FileImage className='h-5 w-5' />
              <Trans>Collection Preview</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='flex flex-col md:flex-row gap-6 items-start'>
              <div className='flex-shrink-0 w-full md:w-auto md:max-w-[400px]'>
                <div className='relative'>
                  {getBannerUrl() && (
                    <div className='w-full h-48 mb-4 rounded-lg overflow-hidden'>
                      <img
                        src={getBannerUrl() ?? missing}
                        alt={t`Banner for ${collectionName}`}
                        className='w-full h-full object-cover'
                      />
                    </div>
                  )}
                  <div
                    className={`mx-auto max-w-[200px] ${getBannerUrl() ? '-mt-24 relative z-10' : ''}`}
                  >
                    {collection.icon ? (
                      <div className='rounded-lg overflow-hidden bg-white dark:bg-neutral-900 shadow-lg'>
                        <img
                          src={collection.icon}
                          alt={t`Icon for ${collectionName}`}
                          className='w-full aspect-square object-contain'
                        />
                      </div>
                    ) : (
                      <div className='w-full aspect-square bg-neutral-100 dark:bg-neutral-800 rounded-lg flex items-center justify-center shadow-lg'>
                        <span className='text-neutral-400 dark:text-neutral-600'>
                          <Trans>No Icon</Trans>
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className='flex-1 min-w-0 space-y-2'>
                <AddressItem
                  label={t`Collection ID`}
                  address={collection.collection_id}
                />

                <div className='space-y-2'>
                  <AddressItem
                    label={t`Minter DID`}
                    address={collection.did_id}
                  />
                  {minterProfile && (
                    <div
                      className='flex items-center gap-2 mt-1 cursor-pointer text-blue-700 dark:text-blue-300 hover:underline'
                      onClick={() =>
                        openUrl(`https://mintgarden.io/${collection.did_id}`)
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

                <AddressItem
                  label={t`Metadata Collection ID`}
                  address={collection.metadata_collection_id}
                />

                {metadataContent?.collection?.attributes?.find(
                  (attr: AttributeType) =>
                    attr.type.toLowerCase() === 'description',
                ) && (
                  <LabeledItem label={t`Description`} content={null}>
                    <div className='break-words text-sm'>
                      {
                        metadataContent.collection.attributes.find(
                          (attr: AttributeType) =>
                            attr.type.toLowerCase() === 'description',
                        )?.value
                      }
                    </div>
                  </LabeledItem>
                )}

                {metadataContent?.collection &&
                  Object.entries(metadataContent.collection)
                    .filter(
                      ([key]) => !['name', 'id', 'attributes'].includes(key),
                    )
                    .map(([key, value]) => (
                      <LabeledItem
                        key={key}
                        label={key}
                        className='capitalize'
                        content={null}
                      >
                        <div className='text-sm'>
                          {typeof value === 'string' &&
                          (value as string).match(
                            /^(https?|ipfs|data):\/\/\S+/i,
                          ) ? (
                            <div
                              className='text-blue-700 dark:text-blue-300 cursor-pointer hover:underline truncate'
                              onClick={() => openUrl(value as string)}
                            >
                              {value}
                            </div>
                          ) : (
                            <span>{String(value)}</span>
                          )}
                        </div>
                      </LabeledItem>
                    ))}
              </div>
            </div>
          </CardContent>
        </Card>

        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4'>
          {metadataContent?.collection?.attributes && (
            <Card>
              <CardHeader>
                <CardTitle className='flex items-center gap-2'>
                  <Tag className='h-5 w-5' />
                  <Trans>Attributes</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className='grid grid-cols-2 gap-3'>
                  {metadataContent.collection.attributes
                    .filter(
                      (attr: AttributeType) =>
                        attr.type.toLowerCase() !== 'description',
                    )
                    .map((attr: AttributeType) => (
                      <div
                        key={attr.type}
                        className='px-3 py-2 border rounded-lg'
                      >
                        <LabeledItem
                          label={attr.type}
                          className='truncate'
                          content={null}
                        >
                          {typeof attr.value === 'string' &&
                          attr.value.match(/^(https?|ipfs|data):\/\/\S+/i) ? (
                            <div
                              onClick={() => openUrl(attr.value)}
                              className='text-sm text-blue-700 dark:text-blue-300 cursor-pointer hover:underline truncate'
                            >
                              {attr.value}
                            </div>
                          ) : (
                            <div className='text-sm truncate'>{attr.value}</div>
                          )}
                        </LabeledItem>
                      </div>
                    ))}
                </div>
              </CardContent>
            </Card>
          )}

          <Card>
            <CardHeader>
              <CardTitle className='flex items-center gap-2'>
                <Info className='h-5 w-5' />
                <Trans>Collection Information</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent className='space-y-3'>
              <AddressItem
                label={t`Collection ID`}
                address={collection.collection_id}
              />

              <AddressItem
                label={t`Metadata Collection ID`}
                address={collection.metadata_collection_id}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className='flex items-center gap-2'>
                <ExternalLink className='h-5 w-5' />
                <Trans>External Links</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent className='space-y-3'>
              <Button
                variant='outline'
                className='w-full justify-start'
                onClick={() =>
                  openUrl(
                    `https://${network === 'testnet' ? 'testnet.' : ''}mintgarden.io/collections/${collection.collection_id}`,
                  )
                }
                disabled={network === 'unknown'}
              >
                <img
                  src='https://mintgarden.io/mint-logo.svg'
                  className='h-4 w-4 mr-2'
                  alt='MintGarden logo'
                />
                <Trans>View on MintGarden</Trans>
              </Button>
              <Button
                variant='outline'
                className='w-full justify-start'
                onClick={() =>
                  openUrl(
                    `https://${network === 'testnet' ? 'testnet11.' : ''}spacescan.io/collection/${collection.collection_id}`,
                  )
                }
                disabled={network === 'unknown'}
              >
                <img
                  src={spacescanLogo}
                  className='h-4 w-4 mr-2'
                  alt='Spacescan.io logo'
                />
                <Trans>View on Spacescan.io</Trans>
              </Button>
            </CardContent>
          </Card>
        </div>
      </Container>
    </>
  );
}
