import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { open } from '@tauri-apps/plugin-shell';
import { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import { commands, events, NetworkConfig, NftRecord } from '../bindings';

export default function Nft() {
  const { launcher_id: launcherId } = useParams();

  const [nft, setNft] = useState<NftRecord | null>(null);

  const updateNft = () => {
    commands.getNft(launcherId!).then((res) => {
      if (res.status === 'ok') {
        setNft(res.data);
      }
    });
  };

  useEffect(() => {
    updateNft();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'nft_update') {
        updateNft();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

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
    commands.networkConfig().then((res) => {
      if (res.status === 'error') {
        return;
      }
      setConfig(res.data);
    });
  }, []);

  return (
    <>
      <Header title={metadata.name ?? 'Unknown NFT'} />
      <Container>
        <div className='text-muted-foreground text-sm'>{nft?.launcher_id}</div>
        <div className='grid lg:grid-cols-2 gap-4 mt-4'>
          <div className='py-2'>
            <img
              alt='NFT image'
              src={`data:${nft?.data_mime_type};base64,${nft?.data}`}
              className='w-full rounded-lg'
            />
          </div>
          <div className='p-2 flex flex-col '>
            {metadata.description && (
              <>
                <h6 className='text-lg font-bold '>Description</h6>
                <div className='break-all text-sm mb-4'>
                  {metadata.description}
                </div>
              </>
            )}
            <h6 className='text-lg font-bold'>Owner DID</h6>
            <div className='break-all font-mono tracking-tight text-sm'>
              {nft?.owner_did ?? 'None'}
            </div>

            <h6 className='text-lg font-bold mt-4'>Address</h6>
            <div className='break-all font-mono tracking-tight text-sm'>
              {nft?.address}
            </div>

            <h6 className='text-lg font-bold mt-4'>Coin Id</h6>
            <div className='break-all font-mono tracking-tight text-sm'>
              {nft?.coin_id}
            </div>

            <h6 className='text-lg font-bold mt-4'>
              Royalties ({nft?.royalty_percent}%)
            </h6>
            <div className='break-all font-mono tracking-tight text-sm'>
              {nft?.royalty_address}
            </div>
            <Button
              variant='outline'
              className='mt-4'
              onClick={() =>
                open(
                  `https://${config?.network_id !== 'mainnet' ? 'testnet.' : ''}mintgarden.io/nfts/${nft?.launcher_id}`,
                )
              }
            >
              <img
                src='https://mintgarden.io/mint-logo.svg'
                className='h-4 w-4 mr-2'
                alt='MintGarden logo'
              />
              Inspect on MintGarden
            </Button>
          </div>
        </div>
      </Container>
    </>
  );
}
