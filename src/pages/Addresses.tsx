import Container from '@/components/Container';
import Header from '@/components/Header';
import { PasteInput } from '@/components/PasteInput';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { SignMessageDialog } from '@/components/dialogs/SignMessageDialog';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ContactsView } from '@/components/contacts/ContactsView';
import { ResolvedHandlesView } from '@/components/contacts/ResolvedHandlesView';
import { CustomError } from '@/contexts/ErrorContext';
import { useDerivationState } from '@/hooks/useDerivationState';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { xchandlesEnabled } from '@/lib/features';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CheckIcon, SignatureIcon, XCircleIcon } from 'lucide-react';
import { useCallback, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands } from '../bindings';
import AddressList from '../components/AddressList';
import { useWalletState } from '../state';

const TAB_TRIGGER_CLASS =
  'flex-1 md:flex-none rounded-md px-3 py-1 text-sm font-medium';

export default function Addresses() {
  const [searchParams, setSearchParams] = useSearchParams();
  const requestedTab = searchParams.get('tab');
  // Contacts for raw addresses are useful without XCHandles, so only the
  // resolved handles view is hidden when it's disabled.
  const tab =
    requestedTab === 'contacts' ||
    (requestedTab === 'handles' && xchandlesEnabled)
      ? requestedTab
      : 'receive';

  const { addError } = useErrors();
  const walletState = useWalletState();
  const ticker = walletState.sync.unit.ticker;
  const [hardened, setHardened] = useState(false);
  const [addressToCheck, setAddressToCheck] = useState('');
  const [checkStatus, setCheckStatus] = useState<'idle' | 'valid' | 'invalid'>(
    'idle',
  );
  const [signOpen, setSignOpen] = useState(false);
  const [signAddress, setSignAddress] = useState('');

  const openSignMessage = useCallback((address: string) => {
    setSignAddress(address);
    setSignOpen(true);
  }, []);

  const {
    derivations,
    currentPage,
    totalDerivations,
    pageSize,
    setCurrentPage,
  } = useDerivationState(hardened);

  const { handleScanOrPaste, handleScanImage } = useScannerOrClipboard(
    (text: string) => {
      setAddressToCheck(text);
      setCheckStatus('idle');
    },
  );

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

  const title =
    tab === 'contacts'
      ? t`Contacts`
      : tab === 'handles'
        ? t`Resolved Handles`
        : t`Receive ${ticker}`;

  return (
    <>
      <Header title={title} alwaysShowChildren>
        {tab === 'receive' && (
          <Button variant='outline' onClick={() => openSignMessage('')}>
            <SignatureIcon className='h-4 w-4 mr-2' aria-hidden='true' />
            <Trans>Sign Message</Trans>
          </Button>
        )}
      </Header>

      <Container className='flex flex-col gap-4 max-w-screen-lg'>
        <Tabs
          value={tab}
          onValueChange={(value) => setSearchParams({ tab: value })}
          className='w-full'
        >
          <TabsList className='w-full md:w-auto inline-flex h-9 items-center justify-start rounded-lg bg-muted p-1 text-muted-foreground'>
            <TabsTrigger value='receive' className={TAB_TRIGGER_CLASS}>
              <Trans>My Addresses</Trans>
            </TabsTrigger>
            <TabsTrigger value='contacts' className={TAB_TRIGGER_CLASS}>
              <Trans>Contacts</Trans>
            </TabsTrigger>
            {xchandlesEnabled && (
              <TabsTrigger value='handles' className={TAB_TRIGGER_CLASS}>
                <Trans>Resolved Handles</Trans>
              </TabsTrigger>
            )}
          </TabsList>

          <TabsContent value='receive' className='flex flex-col gap-4 mt-4'>
            <Card>
              <CardHeader>
                <CardTitle className='text-lg font-medium'>
                  <Trans>Fresh Address</Trans>
                </CardTitle>
                <CardDescription>
                  <Trans>
                    The wallet generates a new address after each transaction.
                    Old ones stay valid.
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
                <PasteInput
                  placeholder={t`Enter address`}
                  aria-label={t`Address to check`}
                  aria-describedby='checkAddressDescription'
                  value={addressToCheck}
                  onChange={(e) => {
                    setAddressToCheck(e.target.value);
                    setCheckStatus('idle');
                  }}
                  onEndIconClick={handleScanOrPaste}
                  onScanImage={handleScanImage}
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
                  className='w-9 h-9'
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
                  onSignMessage={openSignMessage}
                />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value='contacts' className='mt-4'>
            <ContactsView />
          </TabsContent>

          {xchandlesEnabled && (
            <TabsContent value='handles' className='mt-4'>
              <ResolvedHandlesView />
            </TabsContent>
          )}
        </Tabs>
      </Container>

      <SignMessageDialog
        open={signOpen}
        setOpen={setSignOpen}
        address={signAddress}
      />
    </>
  );
}
