import { commands, ContactRecord } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { refreshAddressLabels } from '@/lib/addressContacts';
import { toCustomError } from '@/lib/errors';
import { xchandlesEnabled } from '@/lib/features';
import { isHandleInput, validateRecipient } from '@/lib/handles';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { HandleResolution } from '../HandleResolution';
import { PasteInput } from '../PasteInput';
import { Button } from '../ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '../ui/form';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';

export interface ContactDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  /** The contact to edit, or nothing to add one. */
  contact?: ContactRecord | null;
  /** Prefills a new contact, such as a resolved handle being saved. */
  initial?: { name?: string; recipient?: string };
  onSaved: (contact: ContactRecord) => void;
}

/**
 * Adds a contact, or renames one. A contact's recipient is its identity, so
 * it can't be changed once saved; a different recipient is a new contact.
 */
export function ContactDialog({
  open,
  setOpen,
  contact,
  initial,
  onSaved,
}: ContactDialogProps) {
  const { addError } = useErrors();
  const editing = !!contact;

  const schema = z.object({
    name: z
      .string()
      .trim()
      .min(1, t`Name is required`)
      .max(64, t`Name must be at most 64 characters`),
    recipient: z
      .string()
      .trim()
      .min(1, t`Address is required`)
      .refine(
        (value) => xchandlesEnabled || !isHandleInput(value),
        t`Handles are not supported in this build`,
      ),
    note: z.string().max(1000, t`Note must be at most 1000 characters`),
  });

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { name: '', recipient: '', note: '' },
  });

  useEffect(() => {
    if (!open) return;

    form.reset({
      name: contact?.name ?? initial?.name ?? '',
      recipient: contact?.recipient ?? initial?.recipient ?? '',
      note: contact?.note ?? '',
    });
  }, [open, contact, initial, form]);

  const { handleScanOrPaste, handleScanImage } = useScannerOrClipboard(
    (value) => form.setValue('recipient', value, { shouldValidate: true }),
  );

  const recipient = form.watch('recipient');

  const handler = async (values: z.infer<typeof schema>) => {
    const note = values.note.trim() || null;

    try {
      if (contact) {
        const { contact: updated } = await commands.updateContact({
          id: contact.id,
          name: values.name,
          note,
        });
        onSaved(updated);
      } else {
        if (!(await validateRecipient(values.recipient))) {
          form.setError('recipient', {
            message: xchandlesEnabled
              ? t`Invalid address or handle`
              : t`Invalid address`,
          });
          return;
        }

        const { contact: added } = await commands.addContact({
          name: values.name,
          recipient: values.recipient,
          note,
        });
        onSaved(added);
      }

      refreshAddressLabels();
      setOpen(false);
    } catch (error) {
      addError(toCustomError(error));
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {editing ? <Trans>Edit Contact</Trans> : <Trans>Add Contact</Trans>}
          </DialogTitle>
          <DialogDescription>
            {editing ? (
              <Trans>
                You can rename a contact or change its note. To use a different
                recipient, add a new contact.
              </Trans>
            ) : xchandlesEnabled ? (
              <Trans>
                Save an address or an @handle under a name. Handles are verified
                again every time you send to them.
              </Trans>
            ) : (
              <Trans>Save an address under a name.</Trans>
            )}
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handler)} className='space-y-4'>
            <FormField
              control={form.control}
              name='name'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    <Trans>Name</Trans>
                  </FormLabel>
                  <FormControl>
                    <Input {...field} placeholder={t`Contact name`} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name='recipient'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    {xchandlesEnabled ? (
                      <Trans>Address or Handle</Trans>
                    ) : (
                      <Trans>Address</Trans>
                    )}
                  </FormLabel>
                  <FormControl>
                    {editing ? (
                      <Input {...field} readOnly disabled />
                    ) : (
                      <PasteInput
                        {...field}
                        autoCorrect='off'
                        autoCapitalize='off'
                        autoComplete='off'
                        placeholder={
                          xchandlesEnabled
                            ? t`Enter address or @handle`
                            : t`Enter address`
                        }
                        onEndIconClick={handleScanOrPaste}
                        onScanImage={handleScanImage}
                      />
                    )}
                  </FormControl>
                  {!editing && <HandleResolution value={recipient} />}
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name='note'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    <Trans>Note</Trans>
                  </FormLabel>
                  <FormControl>
                    <Textarea
                      {...field}
                      placeholder={t`Optional private note`}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <DialogFooter className='gap-2'>
              <Button
                type='button'
                variant='outline'
                onClick={() => setOpen(false)}
              >
                <Trans>Cancel</Trans>
              </Button>
              <Button type='submit' disabled={form.formState.isSubmitting}>
                {editing ? <Trans>Save</Trans> : <Trans>Add Contact</Trans>}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
