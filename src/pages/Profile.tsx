import { AssetIcon } from '@/components/AssetIcon';
import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { useDidData } from '@/hooks/useDidData';
import spacescanLogo from '@/images/spacescan-logo-192.png';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands, NetworkKind } from '../bindings';

export default function Profile() {
  const { launcher_id: launcherId } = useParams();
  const { did, isLoading, didAsset } = useDidData({
    did: launcherId || '',
  });
  const [network, setNetwork] = useState<NetworkKind | null>(null);

  useEffect(() => {
    commands
      .getNetwork({})
      .then((data) => setNetwork(data.kind))
      .catch(() => {
        // Network fetch failed, continue with default state
      });
  }, []);

  if (!launcherId) {
    return (
      <>
        <Header title={t`Invalid DID`} />
        <Container>
          <div className='text-center text-gray-500 dark:text-gray-400'>
            <Trans>Invalid DID ID</Trans>
          </div>
        </Container>
      </>
    );
  }

  if (isLoading) {
    return (
      <>
        <Header title={t`DID Profile`} />
        <Container>
          <div className='mx-auto sm:w-full md:w-[50%] max-w-[600px]'>
            <div className='flex flex-col gap-4'>
              <div className='animate-pulse'>
                <div className='h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/3 mb-2'></div>
                <div className='h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/2'></div>
              </div>
            </div>
          </div>
        </Container>
      </>
    );
  }

  if (!did) {
    return (
      <>
        <Header title={t`DID Profile`} />
        <Container>
          <div className='mx-auto sm:w-full md:w-[50%] max-w-[600px]'>
            <div className='flex flex-col gap-4'>
              <div className='text-center text-gray-500 dark:text-gray-400'>
                <Trans>DID not found</Trans>
              </div>
            </div>
          </div>
        </Container>
      </>
    );
  }

  return (
    <>
      <Header title={t`DID Profile`} />
      <Container>
        <div className='mx-auto sm:w-full md:w-[50%] max-w-[600px]'>
          <div className='flex flex-col gap-4'>
            <h2 className='text-xl font-semibold'>{t`DID Profile`}</h2>

            <div className='flex flex-col gap-3'>
              <div>
                <h6 className='text-md font-bold'>{didAsset?.name}</h6>
                {didAsset && <AssetIcon asset={didAsset} size='xxl' />}
              </div>

              <div>
                <h6 className='text-md font-bold'>
                  <Trans>Launcher ID</Trans>
                </h6>
                <CopyBox
                  title={t`Launcher ID`}
                  value={did.launcher_id}
                  onCopy={() =>
                    toast.success(t`Launcher ID copied to clipboard`)
                  }
                />
              </div>

              {did.address && (
                <div>
                  <h6 className='text-md font-bold'>
                    <Trans>Address</Trans>
                  </h6>
                  <CopyBox
                    title={t`Address`}
                    value={did.address}
                    onCopy={() => toast.success(t`Address copied to clipboard`)}
                  />
                </div>
              )}

              {did.coin_id && did.coin_id !== '0' && (
                <div>
                  <h6 className='text-md font-bold'>
                    <Trans>Coin ID</Trans>
                  </h6>
                  <CopyBox
                    title={t`Coin ID`}
                    value={did.coin_id}
                    onCopy={() => toast.success(t`Coin ID copied to clipboard`)}
                  />
                </div>
              )}

              {did.recovery_hash && (
                <div>
                  <h6 className='text-md font-bold'>
                    <Trans>Recovery Hash</Trans>
                  </h6>
                  <CopyBox
                    title={t`Recovery Hash`}
                    value={did.recovery_hash}
                    onCopy={() =>
                      toast.success(t`Recovery Hash copied to clipboard`)
                    }
                  />
                </div>
              )}

              <div className='flex flex-col gap-1'>
                <h6 className='text-md font-bold'>
                  <Trans>External Links</Trans>
                </h6>

                <Button
                  variant='outline'
                  onClick={() => {
                    openUrl(
                      `https://${network === 'testnet' ? 'testnet.' : ''}mintgarden.io/${did.launcher_id}`,
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
                      `https://${network === 'testnet' ? 'testnet11.' : ''}spacescan.io/did/${did.launcher_id}`,
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
        </div>
      </Container>
    </>
  );
}
