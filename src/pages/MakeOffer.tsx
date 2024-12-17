import { Assets, commands } from '@/bindings';
import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { TokenAmountInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { toMojos } from '@/lib/utils';
import { clearOffer, useOfferState, useWalletState } from '@/state';
import {
  HandCoins,
  Handshake,
  ImageIcon,
  PlusIcon,
  TrashIcon,
} from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function MakeOffer() {
  const state = useOfferState();
  const walletState = useWalletState();
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [offer, setOffer] = useState('');

  const make = () => {
    commands
      .makeOffer({
        offered_assets: {
          xch: toMojos(
            (state.offered.xch || '0').toString(),
            walletState.sync.unit.decimals,
          ),
          cats: state.offered.cats.map((cat) => ({
            asset_id: cat.asset_id,
            amount: toMojos((cat.amount || '0').toString(), 3),
          })),
          nfts: state.offered.nfts,
        },
        requested_assets: {
          xch: toMojos(
            (state.requested.xch || '0').toString(),
            walletState.sync.unit.decimals,
          ),
          cats: state.requested.cats.map((cat) => ({
            asset_id: cat.asset_id,
            amount: toMojos((cat.amount || '0').toString(), 3),
          })),
          nfts: state.requested.nfts,
        },
        fee: toMojos(
          (state.fee || '0').toString(),
          walletState.sync.unit.decimals,
        ),
        expires_at_second:
          state.expiration === null
            ? null
            : Math.ceil(Date.now() / 1000) +
              Number(state.expiration.days || '0') * 24 * 60 * 60 +
              Number(state.expiration.hours || '0') * 60 * 60 +
              Number(state.expiration.minutes || '0') * 60,
      })
      .then((data) => setOffer(data.offer))
      .catch(addError);
  };

  const invalid =
    state.expiration !== null &&
    (isNaN(Number(state.expiration.days)) ||
      isNaN(Number(state.expiration.hours)) ||
      isNaN(Number(state.expiration.minutes)));

  return (
    <>
      <Header title='New Offer' />

      <Container>
        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-screen-lg'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                Offered
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                Add the assets you are offering.
              </div>

              <AssetSelector
                prefix='offer'
                assets={state.offered}
                setAssets={(assets) =>
                  useOfferState.setState({ offered: assets })
                }
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <Handshake className='mr-2 h-4 w-4' />
                Requested
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground'>
                Add the assets you are requesting.
              </div>

              <AssetSelector
                prefix='requested'
                assets={state.requested}
                setAssets={(assets) =>
                  useOfferState.setState({ requested: assets })
                }
              />
            </CardContent>
          </Card>

          <div className='flex flex-col gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='fee'>Network Fee</Label>
              <div className='relative'>
                <Input
                  id='fee'
                  type='text'
                  placeholder='0.00'
                  className='pr-12'
                  value={state.fee}
                  onChange={(e) =>
                    useOfferState.setState({ fee: e.target.value })
                  }
                />

                <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                  <span className='text-gray-500 text-sm' id='price-currency'>
                    {walletState.sync.unit.ticker}
                  </span>
                </div>
              </div>
            </div>

            <div className='flex flex-col gap-2'>
              <div className='flex items-center gap-2'>
                <label htmlFor='expiring'>Expiring offer</label>
                <Switch
                  id='expiring'
                  checked={state.expiration !== null}
                  onCheckedChange={(value) => {
                    if (value) {
                      useOfferState.setState({
                        expiration: {
                          days: '1',
                          hours: '',
                          minutes: '',
                        },
                      });
                    } else {
                      useOfferState.setState({
                        expiration: null,
                      });
                    }
                  }}
                />
              </div>

              {state.expiration !== null && (
                <div className='flex gap-2'>
                  <div className='relative'>
                    <Input
                      className='pr-12'
                      value={state.expiration.days}
                      placeholder='0'
                      onChange={(e) => {
                        if (state.expiration === null) return;

                        useOfferState.setState({
                          expiration: {
                            ...state.expiration,
                            days: e.target.value,
                          },
                        });
                      }}
                    />
                    <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                      <span className='text-gray-500 text-sm'>Days</span>
                    </div>
                  </div>

                  <div className='relative'>
                    <Input
                      className='pr-12'
                      value={state.expiration.hours}
                      placeholder='0'
                      onChange={(e) => {
                        if (state.expiration === null) return;

                        useOfferState.setState({
                          expiration: {
                            ...state.expiration,
                            hours: e.target.value,
                          },
                        });
                      }}
                    />
                    <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                      <span className='text-gray-500 text-sm'>Hours</span>
                    </div>
                  </div>

                  <div className='relative'>
                    <Input
                      className='pr-12'
                      value={state.expiration.minutes}
                      placeholder='0'
                      onChange={(e) => {
                        if (state.expiration === null) return;

                        useOfferState.setState({
                          expiration: {
                            ...state.expiration,
                            minutes: e.target.value,
                          },
                        });
                      }}
                    />
                    <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                      <span className='text-gray-500 text-sm'>Minutes</span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className='mt-4 flex gap-2'>
          <Button
            variant='outline'
            onClick={() => {
              clearOffer();
              navigate('/offers', { replace: true });
            }}
          >
            Cancel Offer
          </Button>
          <Button onClick={make} disabled={invalid}>
            Create Offer
          </Button>
        </div>

        <Dialog open={!!offer} onOpenChange={() => setOffer('')}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Offer Details</DialogTitle>
              <DialogDescription>
                Copy the offer file below and send it to the intended recipient
                or make it public to be accepted by anyone.
                <CopyBox title='Offer File' content={offer} className='mt-2' />
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button
                onClick={() => {
                  commands
                    .importOffer({ offer })
                    .then(() => {
                      setOffer('');
                      clearOffer();
                      navigate('/offers', { replace: true });
                    })
                    .catch(addError);
                }}
              >
                Save
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Container>
    </>
  );
}

interface AssetSelectorProps {
  prefix: string;
  assets: Assets;
  setAssets: (value: Assets) => void;
}

function AssetSelector({ prefix, assets, setAssets }: AssetSelectorProps) {
  const [includeAmount, setIncludeAmount] = useState(!!assets.xch);

  return (
    <>
      <div className='mt-4 flex gap-2 w-full items-center'>
        <Button
          variant='outline'
          className='flex-grow'
          disabled={includeAmount}
          onClick={() => {
            setIncludeAmount(true);
          }}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' />
          XCH
        </Button>
        <Button
          variant='outline'
          className='flex-grow'
          onClick={() => {
            setAssets({
              ...assets,
              nfts: ['', ...assets.nfts],
            });
          }}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' /> NFT
        </Button>

        <Button
          variant='outline'
          className='flex-grow'
          onClick={() => {
            setAssets({
              ...assets,
              cats: [{ asset_id: '', amount: '' }, ...assets.cats],
            });
          }}
        >
          <PlusIcon className='mr-0.5 h-3 w-3' /> Token
        </Button>
      </div>

      {includeAmount && (
        <div className='mt-4 flex flex-col space-y-1.5'>
          <Label htmlFor={`${prefix}-amount`}>XCH</Label>
          <div className='flex'>
            <Input
              id={`${prefix}-amount`}
              className='rounded-r-none z-10'
              placeholder='Enter amount'
              value={assets.xch}
              onChange={(e) =>
                setAssets({
                  ...assets,
                  xch: e.target.value,
                })
              }
            />
            <Button
              variant='outline'
              size='icon'
              className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0'
              onClick={() => {
                setIncludeAmount(false);
              }}
            >
              <TrashIcon className='h-4 w-4' />
            </Button>
          </div>
        </div>
      )}

      {assets.nfts.length > 0 && (
        <div className='flex flex-col gap-4 mt-4'>
          {assets.nfts.map((nft, i) => (
            <div key={i} className='flex flex-col space-y-1.5'>
              <Label
                htmlFor={`${prefix}-nft-${i}`}
                className='flex items-center gap-1'
              >
                <ImageIcon className='h-4 w-4' />
                <span>NFT {i + 1}</span>
              </Label>
              <div className='flex'>
                <Input
                  id={`${prefix}-nft-${i}`}
                  className='rounded-r-none z-10'
                  placeholder='Enter launcher id'
                  value={nft}
                  onChange={(e) => {
                    assets.nfts[i] = e.target.value;
                    setAssets({ ...assets });
                  }}
                />
                <Button
                  variant='outline'
                  size='icon'
                  className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0'
                  onClick={() => {
                    assets.nfts.splice(i, 1);
                    setAssets({ ...assets });
                  }}
                >
                  <TrashIcon className='h-4 w-4' />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {assets.cats.length > 0 && (
        <div className='flex flex-col gap-4 mt-4'>
          {assets.cats.map((cat, i) => (
            <div key={i} className='flex flex-col space-y-1.5'>
              <Label
                htmlFor={`${prefix}-cat-${i}`}
                className='flex items-center gap-1'
              >
                <HandCoins className='h-4 w-4' />
                <span>Token {i + 1}</span>
              </Label>
              <div className='flex'>
                <Input
                  id={`${prefix}-cat-${i}`}
                  className='rounded-r-none z-10'
                  placeholder='Enter asset id'
                  value={cat.asset_id}
                  onChange={(e) => {
                    assets.cats[i].asset_id = e.target.value;
                    setAssets({ ...assets });
                  }}
                />
                <TokenAmountInput
                  id={`${prefix}-cat-${i}-amount`}
                  className='border-l-0 z-10 rounded-l-none rounded-r-none w-[100px]'
                  placeholder='Amount'
                  value={cat.amount}
                  onChange={(e) => {
                    assets.cats[i].amount = e.target.value;
                    setAssets({ ...assets });
                  }}
                />
                <Button
                  variant='outline'
                  size='icon'
                  className='border-l-0 rounded-l-none flex-shrink-0 flex-grow-0'
                  onClick={() => {
                    assets.cats.splice(i, 1);
                    setAssets({ ...assets });
                  }}
                >
                  <TrashIcon className='h-4 w-4' />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
