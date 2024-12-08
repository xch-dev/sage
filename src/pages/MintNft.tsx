import ConfirmationDialog from '@/components/ConfirmationDialog';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useDids } from '@/hooks/useDids';
import { useErrors } from '@/hooks/useErrors';
import { amount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { LoaderCircleIcon } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands, TransactionResponse } from '../bindings';
import Container from '../components/Container';
import { useWalletState } from '../state';

export default function MintNft() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const { dids } = useDids();
  const { addError } = useErrors();

  const [pending, setPending] = useState(false);
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const formSchema = z.object({
    profile: z.string().min(1, 'Profile is required'),
    fee: amount(walletState.sync.unit.decimals).optional(),
    royaltyAddress: z.string().optional(),
    royaltyPercent: amount(2),
    dataUris: z.string(),
    metadataUris: z.string(),
    licenseUris: z.string().optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    setPending(true);

    commands
      .bulkMintNfts({
        fee: toMojos(
          values.fee?.toString() || '0',
          walletState.sync.unit.decimals,
        ),
        did_id: values.profile,
        mints: [
          {
            edition_number: null,
            edition_total: null,
            royalty_address: values.royaltyAddress || null,
            royalty_ten_thousandths: Number(values.royaltyPercent) * 100,
            data_uris: values.dataUris
              .split(',')
              .map((uri) => uri.trim())
              .filter(Boolean),
            metadata_uris: values.metadataUris
              .split(',')
              .map((uri) => uri.trim())
              .filter(Boolean),
            license_uris: (values.licenseUris ?? '')
              .split(',')
              .map((uri) => uri.trim())
              .filter(Boolean),
          },
        ],
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setPending(false));
  };

  return (
    <>
      <Header title='Mint NFT' />

      <Container className='max-w-xl'>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='space-y-4'>
            <FormField
              control={form.control}
              name='profile'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Profile</FormLabel>
                  <FormControl>
                    <Select value={field.value} onValueChange={field.onChange}>
                      <SelectTrigger id='profile' aria-label='Select profile'>
                        <SelectValue placeholder='Select profile' />
                      </SelectTrigger>
                      <SelectContent>
                        {dids
                          .filter((did) => did.visible)
                          .map((did) => {
                            return (
                              <SelectItem
                                key={did.launcher_id}
                                value={did.launcher_id}
                              >
                                {did.name ??
                                  `${did.launcher_id.slice(0, 14)}...${did.launcher_id.slice(-4)}`}
                              </SelectItem>
                            );
                          })}
                      </SelectContent>
                    </Select>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='dataUris'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Data URLs</FormLabel>
                  <FormControl>
                    <Input
                      type='text'
                      placeholder='Enter comma separated URLs'
                      {...field}
                      className='pr-12'
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='metadataUris'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Metadata URLs</FormLabel>
                  <FormControl>
                    <Input
                      type='text'
                      placeholder='Enter comma separated URLs'
                      {...field}
                      className='pr-12'
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='licenseUris'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>License URLs</FormLabel>
                  <FormControl>
                    <Input
                      type='text'
                      placeholder='Enter comma separated URLs'
                      {...field}
                      className='pr-12'
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='royaltyAddress'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Royalty Address</FormLabel>
                  <FormControl>
                    <Input
                      type='text'
                      placeholder='Enter address'
                      {...field}
                      className='pr-12'
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className='grid sm:grid-cols-2 gap-4'>
              <FormField
                control={form.control}
                name='royaltyPercent'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Royalty Percent</FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <Input
                          type='text'
                          placeholder='Enter percent'
                          {...field}
                          className='pr-12'
                        />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span className='text-gray-500 text-sm'>%</span>
                        </div>
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className='grid sm:grid-cols-2 gap-4'>
              <FormField
                control={form.control}
                name='fee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Network Fee</FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <Input
                          type='text'
                          placeholder='0.00'
                          {...field}
                          className='pr-12'
                        />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span className='text-gray-500 text-sm'>
                            {walletState.sync.unit.ticker}
                          </span>
                        </div>
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <Button type='submit' disabled={pending}>
              {pending && (
                <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
              )}
              {pending ? 'Minting' : 'Mint'} NFT
            </Button>
          </form>
        </Form>
      </Container>

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => navigate('/nfts')}
      />
    </>
  );
}
