import {
  commands,
  NetworkKind,
  NftCollectionRecord,
  NftRecord,
} from '@/bindings';
import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { toast } from 'react-toastify';

type MetadataContent = {
  collection?: {
    [key: string]: any;
  };
};

type AttributeType = {
  type: string;
  value: string;
};

export default function CollectionMetaData() {
  const { collection_id } = useParams();
  const { addError } = useErrors();
  const [collection, setCollection] = useState<NftCollectionRecord | null>(
    null,
  );
  const [firstNft, setFirstNft] = useState<NftRecord | null>(null);
  const [metadataContent, setMetadataContent] =
    useState<MetadataContent | null>(null);
  const [loading, setLoading] = useState(true);
  const [network, setNetwork] = useState<NetworkKind | null>(null);

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
            setFirstNft(nft);

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
      } catch (error: any) {
        addError(error);
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, [collection_id, addError]);

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

  const renderMetadataValue = (value: any): JSX.Element => {
    // Helper function to render a string that might be a link
    const renderPossibleLink = (str: string, isDescription = false) => {
      if (str.match(/^(https?|ipfs|data):\/\/\S+/i)) {
        return (
          <div
            className='text-blue-700 dark:text-blue-300 cursor-pointer hover:underline truncate'
            onClick={() => openUrl(str)}
          >
            {str}
          </div>
        );
      }
      return <div className={isDescription ? '' : 'break-all'}>{str}</div>;
    };

    if (typeof value === 'string') {
      return renderPossibleLink(value);
    }
    if (typeof value === 'number' || typeof value === 'boolean') {
      return <span>{String(value)}</span>;
    }
    if (Array.isArray(value)) {
      // Special handling for attributes array
      if (
        value.length > 0 &&
        value.every(
          (item) =>
            typeof item === 'object' &&
            item !== null &&
            'type' in item &&
            'value' in item,
        )
      ) {
        const sortedAttributes = [...value].sort((a, b) =>
          a.type.toLowerCase().localeCompare(b.type.toLowerCase()),
        );

        return (
          <div className='grid grid-cols-2 gap-2'>
            {sortedAttributes.map((item, index) => (
              <div
                key={index}
                className='px-2 py-1 border-2 rounded-lg'
                title={item.value}
              >
                <h6 className='text-sm font-semibold truncate'>{item.type}</h6>
                {typeof item.value === 'string' &&
                item.value.match(/^(https?|ipfs|data):\/\/\S+/i) ? (
                  <div
                    onClick={() => openUrl(item.value)}
                    className='text-sm break-all text-blue-700 dark:text-blue-300 cursor-pointer hover:underline truncate'
                  >
                    {item.value}
                  </div>
                ) : (
                  <div className='text-sm break-all'>{item.value}</div>
                )}
              </div>
            ))}
          </div>
        );
      }
      // Default array handling for non-attribute arrays
      return (
        <ul className='list-disc pl-4'>
          {value.map((item, index) => (
            <li
              key={index}
              className={typeof item === 'string' ? 'break-all' : ''}
            >
              {renderMetadataValue(item)}
            </li>
          ))}
        </ul>
      );
    }
    if (typeof value === 'object' && value !== null) {
      // Special handling for single attribute object with type and value
      if (
        'type' in value &&
        'value' in value &&
        typeof value.type === 'string' &&
        typeof value.value === 'string'
      ) {
        return (
          <span>
            <span className='font-bold'>{value.type}</span>:{' '}
            {renderPossibleLink(value.value, value.type === 'description')}
          </span>
        );
      }
      return (
        <div className='pl-4'>
          {Object.entries(value).map(([key, val]) => (
            <div key={key} className='mb-2'>
              <span className='font-medium'>{key}: </span>
              {renderMetadataValue(val)}
            </div>
          ))}
        </div>
      );
    }
    return <span>null</span>;
  };

  const collectionName = collection.name || t`Unnamed Collection`;

  return (
    <>
      <Header title={collection?.name ?? t`Unknown Collection`} />
      <Container>
        <div className='relative'>
          {getBannerUrl() && (
            <div className='w-full h-48 mb-4 rounded-lg overflow-hidden'>
              <img
                src={getBannerUrl()!}
                alt={t`Banner for ${collectionName}`}
                className='w-full h-full object-cover'
              />
            </div>
          )}
          <div
            className={`flex flex-col gap-2 mx-auto sm:w-full md:w-[50%] max-w-[200px] ${getBannerUrl() ? '-mt-24 relative z-10' : ''}`}
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

        <div className='my-4 grid grid-cols-1 md:grid-cols-2 gap-y-3 gap-x-10'>
          <div className='flex flex-col gap-3'>
            {metadataContent?.collection?.attributes && (
              <>
                {/* Find and display description first */}
                {metadataContent.collection.attributes.find(
                  (attr: AttributeType) =>
                    attr.type.toLowerCase() === 'description',
                ) && (
                  <div>
                    <h6 className='text-md font-bold'>
                      <Trans>Description</Trans>
                    </h6>
                    <div className='break-all text-sm'>
                      {
                        metadataContent.collection.attributes.find(
                          (attr: AttributeType) =>
                            attr.type.toLowerCase() === 'description',
                        )?.value
                      }
                    </div>
                  </div>
                )}

                {/* Display remaining attributes */}
                <div>
                  <h6 className='text-md font-bold mb-3'>
                    <Trans>Attributes</Trans>
                  </h6>
                  {renderMetadataValue(
                    metadataContent.collection.attributes.filter(
                      (attr: AttributeType) =>
                        attr.type.toLowerCase() !== 'description',
                    ),
                  )}
                </div>
              </>
            )}
          </div>

          <div className='flex flex-col gap-3'>
            <div>
              <h6 className='text-md font-bold'>
                <Trans>Collection ID</Trans>
              </h6>
              <CopyBox
                title={t`Collection ID`}
                value={collection.collection_id}
                onCopy={() =>
                  toast.success(t`Collection ID copied to clipboard`)
                }
              />
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Metadata Collection ID</Trans>
              </h6>
              <CopyBox
                title={t`Metadata Collection ID`}
                value={collection.metadata_collection_id}
                onCopy={() =>
                  toast.success(t`Metadata Collection ID copied to clipboard`)
                }
              />
            </div>

            <div>
              <h6 className='text-md font-bold'>
                <Trans>Minter DID</Trans>
              </h6>
              <CopyBox
                title={t`Minter DID`}
                value={collection.did_id}
                onCopy={() => toast.success(t`Minter DID copied to clipboard`)}
              />
            </div>

            <div className='flex flex-col gap-1'>
              <h6 className='text-md font-bold'>
                <Trans>External Links</Trans>
              </h6>
              <Button
                variant='outline'
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
                MintGarden
              </Button>
              <Button
                variant='outline'
                onClick={() =>
                  openUrl(
                    `https://${network === 'testnet' ? 'testnet11.' : ''}spacescan.io/collection/${collection.collection_id}`,
                  )
                }
                disabled={network === 'unknown'}
              >
                <img
                  src='https://spacescan.io/images/spacescan-logo-192.png'
                  className='h-4 w-4 mr-2'
                  alt='Spacescan.io logo'
                />
                Spacescan.io
              </Button>
            </div>

            {metadataContent?.collection &&
              Object.entries(metadataContent.collection)
                .filter(([key]) => !['name', 'id', 'attributes'].includes(key))
                .map(([key, value]) => (
                  <div key={key}>
                    <h6 className='text-md font-bold capitalize mb-1'>{key}</h6>
                    <div>{renderMetadataValue(value)}</div>
                  </div>
                ))}
          </div>
        </div>
      </Container>
    </>
  );
}
