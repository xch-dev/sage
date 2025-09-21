import ConfirmationDialog from '@/components/ConfirmationDialog';
import Header from '@/components/Header';
import { PasteInput } from '@/components/PasteInput';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { LoadingButton } from '@/components/ui/loading-button';
import {
  FeeAmountInput,
  IntegerInput,
  MaskedInput,
} from '@/components/ui/masked-input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useDids } from '@/hooks/useDids';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { amount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
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
    profile: z.string().min(1, t`Profile is required`),
    fee: amount(walletState.sync.unit.precision).optional(),
    royaltyAddress: z.string().optional(),
    royaltyPercent: amount(2),
    dataUris: z.string(),
    metadataUris: z.string(),
    licenseUris: z.string().optional(),
    editionCount: z.number().min(1).default(1),
    editionStart: z.number().min(1).default(1),
    editionTotal: z.number().min(1).optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      editionCount: 1,
      editionStart: 1,
      editionTotal: undefined,
    },
  });

  const { handleScanOrPaste: handleRoyaltyPaste } = useScannerOrClipboard(
    (scanResValue) => {
      form.setValue('royaltyAddress', scanResValue);
    },
  );

  const { handleScanOrPaste: handleDataUrisPaste } = useScannerOrClipboard(
    (scanResValue) => {
      form.setValue('dataUris', scanResValue);
    },
  );

  const { handleScanOrPaste: handleMetadataUrisPaste } = useScannerOrClipboard(
    (scanResValue) => {
      form.setValue('metadataUris', scanResValue);
    },
  );

  const { handleScanOrPaste: handleLicenseUrisPaste } = useScannerOrClipboard(
    (scanResValue) => {
      form.setValue('licenseUris', scanResValue);
    },
  );

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    setPending(true);

    const mintDetails = {
      did_id: values.profile,
      royalty_address: values.royaltyAddress || null,
      royalty_percent: Number(values.royaltyPercent),
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
    };

    const mints =
      values.editionCount > 1
        ? Array.from({ length: values.editionCount }, (_, i) => ({
            edition_number: i + values.editionStart,
            edition_total: values.editionTotal ?? values.editionCount,
            royalty_address: values.royaltyAddress || null,
            royalty_ten_thousandths: Number(values.royaltyPercent) * 100,
            data_uris: mintDetails.data_uris,
            metadata_uris: mintDetails.metadata_uris,
            license_uris: mintDetails.license_uris,
          }))
        : [
            {
              edition_number: null,
              edition_total: null,
              royalty_address: values.royaltyAddress || null,
              royalty_ten_thousandths: Number(values.royaltyPercent) * 100,
              data_uris: mintDetails.data_uris,
              metadata_uris: mintDetails.metadata_uris,
              license_uris: mintDetails.license_uris,
            },
          ];

    commands
      .bulkMintNfts({
        fee: toMojos(
          values.fee?.toString() || '0',
          walletState.sync.unit.precision,
        ),
        did_id: values.profile,
        mints,
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setPending(false));
  };

  return (
    <>
      <Header title={t`Mint NFT`} />

      <Container className='max-w-xl'>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='space-y-4'>
            <FormField
              control={form.control}
              name='profile'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    <Trans>Profile</Trans>
                  </FormLabel>
                  <FormControl>
                    <Select value={field.value} onValueChange={field.onChange}>
                      <SelectTrigger
                        id='profile'
                        aria-label={t`Select profile`}
                      >
                        <SelectValue placeholder={t`Select profile`} />
                      </SelectTrigger>
                      <SelectContent>
                        {dids
                          .filter((did) => did.visible)
                          .map((did) => (
                            <SelectItem
                              key={did.launcher_id}
                              value={did.launcher_id}
                            >
                              {did.name ??
                                `${did.launcher_id.slice(0, 14)}...${did.launcher_id.slice(-4)}`}
                            </SelectItem>
                          ))}
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
                  <FormLabel>
                    <Trans>Data URLs</Trans>
                  </FormLabel>
                  <FormControl>
                    <PasteInput
                      type='text'
                      placeholder={t`Enter comma separated URLs`}
                      {...field}
                      onEndIconClick={handleDataUrisPaste}
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
                  <FormLabel>
                    <Trans>Metadata URLs</Trans>
                  </FormLabel>
                  <FormControl>
                    <PasteInput
                      type='text'
                      placeholder={t`Enter comma separated URLs`}
                      {...field}
                      onEndIconClick={handleMetadataUrisPaste}
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
                  <FormLabel>
                    <Trans>License URLs</Trans>
                  </FormLabel>
                  <FormControl>
                    <PasteInput
                      type='text'
                      placeholder={t`Enter comma separated URLs`}
                      {...field}
                      onEndIconClick={handleLicenseUrisPaste}
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
                  <FormLabel>
                    <Trans>Royalty Address</Trans>
                  </FormLabel>
                  <FormControl>
                    <PasteInput
                      type='text'
                      placeholder={t`Enter address`}
                      {...field}
                      onEndIconClick={handleRoyaltyPaste}
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
                    <FormLabel>
                      <Trans>Royalty Percent</Trans>
                    </FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <MaskedInput
                          title={t`The maximum royalty percent is 655.36%`}
                          placeholder={t`Enter percent`}
                          allowLeadingZeros={true}
                          allowNegative={false}
                          decimalScale={2}
                          isAllowed={(values) => {
                            const { floatValue } = values;
                            return !floatValue || floatValue <= 655.36;
                          }}
                          {...field}
                          className='pr-12'
                        />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span className='text-muted-foreground text-sm'>
                            %
                          </span>
                        </div>
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name='editionCount'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Edition Count</Trans>
                    </FormLabel>
                    <FormControl>
                      <IntegerInput
                        min={1}
                        placeholder={t`Enter count`}
                        {...field}
                        onChange={(e) =>
                          field.onChange(parseInt(e.target.value, 10))
                        }
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className='grid sm:grid-cols-2 gap-4'>
              <FormField
                control={form.control}
                name='editionStart'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Edition Start</Trans>
                    </FormLabel>
                    <FormControl>
                      <IntegerInput
                        min={1}
                        placeholder={t`Enter start`}
                        {...field}
                        onChange={(e) =>
                          field.onChange(parseInt(e.target.value, 10))
                        }
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name='editionTotal'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Edition Total</Trans>
                    </FormLabel>
                    <FormControl>
                      <IntegerInput
                        min={1}
                        placeholder={t`Enter total (defaults to count)`}
                        {...field}
                        onChange={(e) =>
                          field.onChange(parseInt(e.target.value, 10))
                        }
                      />
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
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <FeeAmountInput {...field} className='pr-12' />
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <LoadingButton
              type='submit'
              loading={pending}
              loadingText={t`Minting`}
            >
              <Trans>Mint</Trans> NFT
            </LoadingButton>
          </form>
        </Form>
      </Container>

      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => setResponse(null)}
        onConfirm={() => navigate('/nfts')}
        additionalData={
          response
            ? {
                title: 'NFT Minting Details',
                content: (
                  <div className='space-y-1 text-xs'>
                    <div>
                      <strong>Profile:</strong> {form.getValues('profile')}
                    </div>
                    {form.getValues('royaltyAddress') && (
                      <div>
                        <strong>Royalty Address:</strong>{' '}
                        {form.getValues('royaltyAddress')}
                      </div>
                    )}
                    {form.getValues('editionCount') > 1 && (
                      <div>
                        <strong>Edition Count:</strong>{' '}
                        {form.getValues('editionCount')}
                      </div>
                    )}
                    <div>
                      <strong>Royalty Percent:</strong>{' '}
                      {form.getValues('royaltyPercent')}%
                    </div>
                    <div>
                      <strong>Data URLs:</strong>{' '}
                      {form
                        .getValues('dataUris')
                        .split(',')
                        .map((uri) => uri.trim())
                        .filter(Boolean)
                        .map((uri, index, array) => (
                          <>
                            <a
                              href={uri}
                              target='_blank'
                              rel='noopener noreferrer'
                              className='text-blue-500 hover:underline'
                            >
                              {uri}
                            </a>
                            {index < array.length - 1 ? ', ' : ''}
                          </>
                        ))}
                    </div>
                    <div>
                      <strong>Metadata URLs:</strong>{' '}
                      {form
                        .getValues('metadataUris')
                        .split(',')
                        .map((uri) => uri.trim())
                        .filter(Boolean)
                        .map((uri, index, array) => (
                          <>
                            <a
                              href={uri}
                              target='_blank'
                              rel='noopener noreferrer'
                              className='text-blue-500 hover:underline'
                            >
                              {uri}
                            </a>
                            {index < array.length - 1 ? ', ' : ''}
                          </>
                        ))}
                    </div>
                    {form.getValues('licenseUris') && (
                      <div>
                        <strong>License URLs:</strong>{' '}
                        {form
                          .getValues('licenseUris')
                          ?.split(',')
                          .map((uri) => uri.trim())
                          .filter(Boolean)
                          .map((uri, index, array) => (
                            <>
                              <a
                                href={uri}
                                target='_blank'
                                rel='noopener noreferrer'
                                className='text-blue-500 hover:underline'
                              >
                                {uri}
                              </a>
                              {index < array.length - 1 ? ', ' : ''}
                            </>
                          ))}
                      </div>
                    )}
                  </div>
                ),
              }
            : undefined
        }
      />
    </>
  );
}
