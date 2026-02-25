import Header from '@/components/Header';
import { MnemonicDisplay } from '@/components/MnemonicDisplay';
import SafeAreaView from '@/components/SafeAreaView';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { KeyIcon, PlusIcon, ShieldIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, WalletRecord } from '../bindings';
import Container from '../components/Container';

export default function Keys() {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [keys, setKeys] = useState<WalletRecord[] | null>(null);
  const [generateOpen, setGenerateOpen] = useState(false);
  const [newMnemonic, setNewMnemonic] = useState('');

  useEffect(() => {
    commands
      .getKeys({})
      .then((data) => setKeys(data.keys))
      .catch(addError);
  }, [addError]);

  const loadMnemonic = useCallback(() => {
    commands
      .generateMnemonic({ use_24_words: true })
      .then((data) => setNewMnemonic(data.mnemonic))
      .catch(addError);
  }, [addError]);

  const openGenerateDialog = () => {
    loadMnemonic();
    setGenerateOpen(true);
  };

  return (
    <SafeAreaView>
      <Header title={t`Keys`} back={() => navigate(-1 as unknown as string)} />
      <Container>
        <div className='max-w-xl mx-auto space-y-6'>
          <Card>
            <CardHeader>
              <div className='flex items-center justify-between'>
                <div>
                  <CardTitle className='flex items-center gap-2'>
                    <KeyIcon className='h-5 w-5' />
                    <Trans>BLS Keys</Trans>
                  </CardTitle>
                  <CardDescription>
                    <Trans>BLS keys derived from mnemonic seed phrases.</Trans>
                  </CardDescription>
                </div>
                <Button size='sm' onClick={openGenerateDialog}>
                  <PlusIcon className='h-4 w-4 mr-1' />
                  <Trans>Generate</Trans>
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              {keys === null ? (
                <div className='text-sm text-muted-foreground'>
                  <Trans>Loading...</Trans>
                </div>
              ) : keys.length === 0 ? (
                <div className='text-sm text-muted-foreground text-center py-4'>
                  <Trans>
                    No BLS keys found. Generate or import one to get started
                  </Trans>
                </div>
              ) : (
                <div className='space-y-2'>
                  {keys.map((key) => (
                    <div
                      key={key.fingerprint}
                      className='flex items-center justify-between rounded-lg border p-3'
                    >
                      <div className='flex items-center gap-3'>
                        {key.emoji && (
                          <span className='text-lg'>{key.emoji}</span>
                        )}
                        <div>
                          <div className='font-medium text-sm'>{key.name}</div>
                          <div className='text-xs text-muted-foreground'>
                            <Trans>Fingerprint:</Trans> {key.fingerprint}
                          </div>
                        </div>
                      </div>
                      <div className='flex items-center gap-1'>
                        {key.type === 'bls' && key.has_secrets ? (
                          <Badge variant='default' className='text-xs'>
                            <Trans>Hot</Trans>
                          </Badge>
                        ) : (
                          <Badge variant='secondary' className='text-xs'>
                            <Trans>Cold</Trans>
                          </Badge>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          <Card className='opacity-60'>
            <CardHeader>
              <CardTitle className='flex items-center gap-2'>
                <ShieldIcon className='h-5 w-5' />
                <Trans>Secure Element Keys</Trans>
              </CardTitle>
              <CardDescription>
                <Trans>
                  Hardware-backed keys stored in the device&apos;s secure
                  element. Coming soon.
                </Trans>
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className='text-sm text-muted-foreground text-center py-4'>
                <Trans>Secure element support is not yet available.</Trans>
              </div>
            </CardContent>
          </Card>
        </div>

        <Dialog open={generateOpen} onOpenChange={setGenerateOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                <Trans>Generate New BLS Key</Trans>
              </DialogTitle>
              <DialogDescription>
                <Trans>
                  A new mnemonic has been generated. Save it somewhere safe
                  before closing this dialog.
                </Trans>
              </DialogDescription>
            </DialogHeader>

            <MnemonicDisplay
              mnemonic={newMnemonic}
              onRegenerate={loadMnemonic}
            />

            <DialogFooter>
              <Button variant='outline' onClick={() => setGenerateOpen(false)}>
                <Trans>Close</Trans>
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Container>
    </SafeAreaView>
  );
}
