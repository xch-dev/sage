import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { useDerivationState } from '@/hooks/useDerivationState';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CheckIcon, XCircleIcon } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'react-toastify';
import { commands } from '../bindings';
import AddressList from '../components/AddressList';
import { useWalletState } from '../state';

export default function Addresses() {
  const { addError } = useErrors();
  const walletState = useWalletState();
  const ticker = walletState.sync.unit.ticker;
  const [hardened, setHardened] = useState(false);
  const [addressToCheck, setAddressToCheck] = useState('');
  const [checkStatus, setCheckStatus] = useState<'idle' | 'valid' | 'invalid'>(
    'idle',
  );

  const {
    derivations,
    currentPage,
    totalDerivations,
    pageSize,
    setCurrentPage,
  } = useDerivationState(hardened);

  const pageCount = Math.ceil(totalDerivations / pageSize);

  const handleCheckAddress = async () => {
    try {
      const result = await commands.checkAddress({ address: addressToCheck });
      setCheckStatus(result.valid ? 'valid' : 'invalid');
      if (result.valid) {
        toast.success(t`Address belongs to your wallet`);
      } else {
        toast.error(t`Address not found in your wallet`);
      }
    } catch (error) {
      addError(error as CustomError);
      setCheckStatus('idle');
    }
  };

  return (
    <>
      <Header title={t`Receive ${ticker}`} />

      <Container className='flex flex-col gap-4 max-w-screen-lg'>
        <Card>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>
              <Trans>Fresh Address</Trans>
            </CardTitle>
            <CardDescription>
              <Trans>
                The wallet generates a new address after each transaction. Old
                ones stay valid.
              </Trans>
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ReceiveAddress />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>
              <Trans>Check Address</Trans>
            </CardTitle>
            <CardDescription>
              <Trans>Check if an address is owned by this wallet.</Trans>
            </CardDescription>
          </CardHeader>
          <CardContent className='flex gap-2'>
            <Input
              placeholder={t`Enter address`}
              aria-label={t`Address to check`}
              value={addressToCheck}
              onChange={(e) => {
                setAddressToCheck(e.target.value);
                setCheckStatus('idle');
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && addressToCheck) {
                  handleCheckAddress();
                }
              }}
              aria-describedby='checkAddressDescription'
            />
            <Button
              variant='secondary'
              size='sm'
              onClick={handleCheckAddress}
              disabled={!addressToCheck}
              style={{
                ...(checkStatus === 'valid' && {
                  backgroundColor: '#16a34a',
                  color: 'white',
                }),
                ...(checkStatus === 'invalid' && {
                  backgroundColor: '#dc2626',
                  color: 'white',
                }),
              }}
              aria-live='polite'
              aria-label={
                checkStatus === 'idle'
                  ? t`Check address`
                  : checkStatus === 'valid'
                    ? t`Address is valid`
                    : t`Address is invalid`
              }
              className='w-10'
            >
              {checkStatus === 'idle' ? (
                <CheckIcon className='h-4 w-4' aria-hidden='true' />
              ) : checkStatus === 'valid' ? (
                <CheckIcon className='h-4 w-4' aria-hidden='true' />
              ) : (
                <XCircleIcon className='h-4 w-4' aria-hidden='true' />
              )}
            </Button>
            <div id='checkAddressDescription' className='sr-only'>
              <Trans>Press Enter to check the address after typing</Trans>
            </div>
          </CardContent>
        </Card>

        <Card className='max-w-full'>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>
              <Trans>All Addresses</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='flex items-center gap-2 mb-4'>
              <label htmlFor='hardened'>
                <Trans>Show hardened addresses</Trans>
              </label>
              <Switch
                id='hardened'
                checked={hardened}
                onCheckedChange={(value) => setHardened(value)}
              />
            </div>

            <AddressList
              derivations={derivations}
              currentPage={currentPage}
              totalPages={pageCount}
              setCurrentPage={setCurrentPage}
              totalDerivations={totalDerivations}
            />
          </CardContent>
        </Card>
      </Container>
    </>
  );
}
