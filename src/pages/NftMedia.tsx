import Container from '@/components/Container';
import Header from '@/components/Header';
import { useErrors } from '@/hooks/useErrors';
import { isAudio, isImage, isVideo, nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import { commands, events, NftData, NftRecord } from '../bindings';

export default function NftMedi() {
  const { launcher_id: launcherId } = useParams();
  const { addError } = useErrors();
  const [nft, setNft] = useState<NftRecord | null>(null);
  const [data, setData] = useState<NftData | null>(null);

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

  return (
    <>
      <Header title={nft?.name ?? t`Unknown NFT`} />
      <Container>
        <div className='flex flex-col gap-2 mx-auto w-full'>
          {isImage(data?.mime_type ?? null) ? (
            <img
              alt='NFT image'
              src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
              className='rounded-lg'
            />
          ) : isVideo(data?.mime_type ?? null) ? (
            <video
              src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
              className='rounded-lg'
              controls
            />
          ) : isAudio(data?.mime_type ?? null) ? (
            <div className='flex flex-col items-center justify-center p-4 border rounded-lg bg-gray-100 dark:bg-gray-800'>
              <div className='text-4xl mb-2'>🎵</div>
              <audio
                src={nftUri(data?.mime_type ?? null, data?.blob ?? null)}
                controls
                className='w-full'
              />
            </div>
          ) : (
            <div className='flex items-center justify-center p-4 border rounded-lg bg-gray-100 dark:bg-gray-800'>
              <span>{t`Unsupported media type`}</span>
            </div>
          )}
        </div>
      </Container>
    </>
  );
}
