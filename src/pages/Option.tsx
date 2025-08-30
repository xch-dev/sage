import { commands, events, OptionRecord } from '@/bindings';
import { AddressItem } from '@/components/AddressItem';
import { AssetCoin } from '@/components/AssetCoin';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { LabeledItem } from '@/components/LabeledItem';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import spacescanLogo from '@/images/spacescan-logo-192.png';
import { formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { AlertCircle, Calendar, Clock, FilePenLine } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';

export default function Option() {
  const { option_id: optionId } = useParams();
  const [option, setOption] = useState<OptionRecord | null>(null);
  const [network, setNetwork] = useState<string | null>(null);

  const updateOption = useCallback(() => {
    commands
      .getOption({
        option_id: optionId ?? '',
      })
      .then((data) => {
        if (data.option) {
          setOption(data.option);
        } else {
          setOption(null);
        }
      })
      .catch((error) => {
        console.error('Failed to fetch option:', error);
        setOption(null);
      });
  }, [optionId]);

  useEffect(() => {
    updateOption();

    const unlisten = events.syncEvent.listen((data) => {
      switch (data.payload.type) {
        case 'coin_state':
        case 'puzzle_batch_synced':
          updateOption();
          break;
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateOption]);

  useEffect(() => {
    commands
      .getNetwork({})
      .then((data) => setNetwork(data.kind))
      .catch((error) => console.error('Failed to get network:', error));
  }, []);

  const isExpired = useMemo(() => {
    if (!option) return false;
    return Date.now() / 1000 > option.expiration_seconds;
  }, [option]);

  const isPending = option?.created_height === null;
  const isActive = option?.created_height !== null && !isExpired;

  if (!option) {
    return (
      <>
        <Header title={t`Option Contract`} />
        <Container>
          <Card className='p-6'>
            <div className='text-center text-muted-foreground'>
              <Trans>Option contract not found</Trans>
            </div>
          </Card>
        </Container>
      </>
    );
  }

  return (
    <>
      <Header title={option.name || t`Unnamed Option`} />
      <Container>
        <Card className='mb-6'>
          <CardHeader className='pb-2'>
            <CardTitle className='flex items-center gap-2'>
              <FilePenLine className='h-5 w-5' />
              <Trans>Option Contract Details</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='flex items-center gap-4 mb-4'>
              <div
                className={`px-3 py-1 rounded-full text-sm font-medium ${
                  isPending
                    ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300'
                    : isActive
                      ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300'
                      : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300'
                }`}
              >
                {isPending ? (
                  <Trans>Pending</Trans>
                ) : isActive ? (
                  <Trans>Active</Trans>
                ) : (
                  <Trans>Expired</Trans>
                )}
              </div>
              {isExpired && (
                <div className='flex items-center gap-1 text-red-600 dark:text-red-400'>
                  <AlertCircle className='h-4 w-4' />
                  <span className='text-sm'>
                    <Trans>This option has expired</Trans>
                  </span>
                </div>
              )}
            </div>

            <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
              <div className='flex items-center gap-2'>
                <Calendar className='h-4 w-4 text-muted-foreground' />
                <LabeledItem
                  label={t`Created`}
                  content={
                    option.created_timestamp
                      ? formatTimestamp(
                          option.created_timestamp,
                          'short',
                          'short',
                        )
                      : t`Pending confirmation`
                  }
                />
              </div>

              <div className='flex items-center gap-2'>
                <Clock className='h-4 w-4 text-muted-foreground' />
                <LabeledItem
                  label={t`Expires`}
                  content={formatTimestamp(
                    option.expiration_seconds,
                    'short',
                    'short',
                  )}
                />
              </div>
              <div className='flex items-center gap-2'>
                <LabeledItem
                  label={t`Block Height`}
                  content={option.created_height?.toString() ?? null}
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <div className='grid grid-cols-1 lg:grid-cols-2 gap-6'>
          <div className='space-y-6'>
            <Card>
              <CardHeader>
                <CardTitle>
                  <Trans>Contract Details</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent className='space-y-4'>
                <LabeledItem label={t`Underlying Asset`} content={null}>
                  <AssetCoin
                    asset={option.underlying_asset}
                    amount={option.underlying_amount}
                    coinId={option.coin_id}
                  />
                </LabeledItem>

                <LabeledItem label={t`Strike Asset`} content={null}>
                  <AssetCoin
                    asset={option.strike_asset}
                    amount={option.strike_amount}
                    coinId={null}
                  />
                </LabeledItem>
              </CardContent>
            </Card>
          </div>

          <div className='space-y-6'>
            <Card>
              <CardHeader>
                <CardTitle>
                  <Trans>Technical Information</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent className='space-y-4'>
                <AddressItem
                  label={t`Option ID`}
                  address={option.launcher_id}
                />
                <AddressItem label={t`Coin ID`} address={option.coin_id} />
                <AddressItem label={t`Address`} address={option.address} />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>
                  <Trans>External Links</Trans>
                </CardTitle>
              </CardHeader>
              <CardContent className='space-y-2'>
                <Button
                  variant='outline'
                  onClick={() => {
                    const baseUrl = network === 'testnet' ? 'testnet11.' : '';
                    openUrl(
                      `https://${baseUrl}spacescan.io/coin/${option.coin_id}`,
                    );
                  }}
                  disabled={!network || network === 'unknown'}
                >
                  <img
                    src={spacescanLogo}
                    className='h-4 w-4 mr-2'
                    alt='Spacescan.io logo'
                  />
                  Spacescan.io
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </Container>
    </>
  );
}
