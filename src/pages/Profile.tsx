import { AddressItem } from '@/components/AddressItem';
import { AssetIcon } from '@/components/AssetIcon';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useDidData } from '@/hooks/useDidData';
import spacescanLogo from '@/images/spacescan-logo-192.png';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ExternalLink, FileText, User } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
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
          <div className='space-y-6'>
            <Card>
              <CardHeader>
                <CardTitle className='flex items-center gap-2'>
                  <User className='h-5 w-5' />
                  <Trans>DID Profile</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className='animate-pulse space-y-4'>
                  <div className='h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/3'></div>
                  <div className='h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/2'></div>
                  <div className='h-4 bg-gray-200 dark:bg-gray-700 rounded w-2/3'></div>
                </div>
              </CardContent>
            </Card>
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
          <Card>
            <CardHeader>
              <CardTitle className='flex items-center gap-2'>
                <User className='h-5 w-5' />
                <Trans>DID Profile</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-center text-gray-500 dark:text-gray-400'>
                <Trans>DID not found</Trans>
              </div>
            </CardContent>
          </Card>
        </Container>
      </>
    );
  }

  return (
    <>
      <Header title={didAsset?.name ?? t`DID Profile`} />
      <Container>
        {/* DID Profile Display */}
        <Card className='mb-6'>
          <CardHeader>
            <CardTitle className='flex items-center gap-2'>
              <User className='h-5 w-5' />
              <Trans>DID Profile</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='flex flex-col md:flex-row gap-6 items-start'>
              <div className='flex-shrink-0 w-full md:w-auto md:max-w-[200px]'>
                {didAsset && (
                  <div className='flex flex-col items-center gap-4 p-4 border rounded-lg bg-gray-50 dark:bg-gray-800'>
                    <AssetIcon asset={didAsset} size='xxl' />
                    <div className='text-center'>
                      <h3 className='font-semibold text-lg'>{didAsset.name}</h3>
                    </div>
                  </div>
                )}
              </div>
              <div className='flex-1 min-w-0 space-y-4'>
                <AddressItem label={t`Launcher ID`} address={did.launcher_id} />
                <AddressItem label={t`Address`} address={did.address} />
              </div>
            </div>
          </CardContent>
        </Card>

        <div className='grid grid-cols-1 lg:grid-cols-2 gap-6'>
          {/* Left Column */}
          <div className='space-y-6'>
            {/* Technical Information */}
            <Card>
              <CardHeader>
                <CardTitle className='flex items-center gap-2'>
                  <FileText className='h-5 w-5' />
                  <Trans>Technical Information</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent className='space-y-4'>
                {did.coin_id && did.coin_id !== '0' && (
                  <AddressItem label={t`Coin ID`} address={did.coin_id} />
                )}
                <AddressItem
                  label={t`Recovery Hash`}
                  address={did.recovery_hash ?? ''}
                />
              </CardContent>
            </Card>
          </div>

          {/* Right Column */}
          <div className='space-y-6'>
            {/* External Links */}
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
                  View on MintGarden
                </Button>

                <Button
                  variant='outline'
                  className='w-full justify-start'
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
                  View on Spacescan.io
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </Container>
    </>
  );
}
