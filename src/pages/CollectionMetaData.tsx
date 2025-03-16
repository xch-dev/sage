import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { NftCollectionRecord, NftRecord } from '@/bindings';
import { Button } from '@/components/ui/button';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'react-toastify';

type MetadataContent = {
  collection?: {
    [key: string]: any;
  };
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
    const renderPossibleLink = (str: string) => {
      if (str.match(/^(https?|ipfs|data):\/\/\S+/i)) {
        return (
          <span
            className='text-blue-700 dark:text-blue-300 cursor-pointer hover:underline'
            onClick={() => openUrl(str)}
          >
            {str}
          </span>
        );
      }
      return <span>{str}</span>;
    };

    if (typeof value === 'string') {
      return renderPossibleLink(value);
    }
    if (typeof value === 'number' || typeof value === 'boolean') {
      return <span>{String(value)}</span>;
    }
    if (Array.isArray(value)) {
      return (
        <ul className='list-disc pl-4'>
          {value.map((item, index) => (
            <li key={index}>
              {/* Special handling for attribute objects with type and value */}
              {typeof item === 'object' &&
              item !== null &&
              'type' in item &&
              'value' in item &&
              typeof item.type === 'string' &&
              typeof item.value === 'string' ? (
                <span>
                  <span className='font-bold'>{item.type}</span>:{' '}
                  {renderPossibleLink(item.value)}
                </span>
              ) : (
                renderMetadataValue(item)
              )}
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
            {renderPossibleLink(value.value)}
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

  return (
    <>
      <Header title={collection?.name ?? t`Unknown Collection`} />
      <Container>
        <div className='flex flex-col gap-2 mx-auto sm:w-full md:w-[50%] max-w-[200px]'>
          {collection.icon ? (
            <img
              src={collection.icon}
              alt={t`Icon for ${collection.name || 'Unnamed Collection'}`}
              className='w-full aspect-square object-contain rounded-lg'
            />
          ) : (
            <div className='w-full aspect-square bg-neutral-100 dark:bg-neutral-800 rounded-lg flex items-center justify-center'>
              <span className='text-neutral-400 dark:text-neutral-600'>
                <Trans>No Icon</Trans>
              </span>
            </div>
          )}
          <CopyBox
            title={t`Collection ID`}
            value={collection.collection_id}
            onCopy={() => toast.success(t`Collection ID copied to clipboard`)}
          />
        </div>

        <div className='my-4 grid grid-cols-1 md:grid-cols-2 gap-y-3 gap-x-10'>
          <div className='flex flex-col gap-3'>
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
          </div>

          <div className='flex flex-col gap-3'>
            <div className='flex flex-col gap-1'>
              <h6 className='text-md font-bold'>
                <Trans>External Links</Trans>
              </h6>
              <Button
                variant='outline'
                onClick={() =>
                  openUrl(
                    `https://mintgarden.io/collections/${collection.collection_id}`,
                  )
                }
              >
                <img
                  src='https://mintgarden.io/mint-logo.svg'
                  className='h-4 w-4 mr-2'
                  alt='MintGarden logo'
                />
                MintGarden
              </Button>
            </div>
          </div>
        </div>

        {metadataContent?.collection && (
          <div className='mt-6'>
            <h6 className='text-md font-bold mb-3'>
              <Trans>Collection Metadata</Trans>
            </h6>
            <div className='text-sm bg-neutral-100 dark:bg-neutral-800 p-4 rounded-lg'>
              {Object.entries(metadataContent.collection)
                .filter(([key]) => !['name', 'id'].includes(key))
                .map(([key, value]) => (
                  <div key={key} className='mb-3'>
                    <div className='font-medium capitalize mb-1'>{key}:</div>
                    <div className='pl-4'>{renderMetadataValue(value)}</div>
                  </div>
                ))}
            </div>
          </div>
        )}
      </Container>
    </>
  );
}
