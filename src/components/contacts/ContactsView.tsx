import { commands, ContactEntry, ContactRecord, events } from '@/bindings';
import { useWallet } from '@/contexts/WalletContext';
import { useDebounce } from '@/hooks/useDebounce';
import { useErrors } from '@/hooks/useErrors';
import { contactKindFilter, contactState } from '@/lib/contacts';
import { toCustomError } from '@/lib/errors';
import { ExportType, exportText } from '@/lib/exportText';
import { xchandlesEnabled } from '@/lib/features';
import { formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  AlertTriangleIcon,
  DownloadIcon,
  MoreVerticalIcon,
  PencilIcon,
  PlusIcon,
  RefreshCwIcon,
  SearchIcon,
  SendIcon,
  TrashIcon,
  UploadIcon,
} from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { CopyButton } from '../CopyButton';
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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import { Input } from '../ui/input';
import { ContactDialog } from './ContactDialog';
import { ContactStatusBadge } from './ContactStatusBadge';

const PAGE_SIZE = 25;

/** The file format for exported contacts. */
interface ContactsFile {
  format: 'sage-contacts';
  version: 1;
  network_id: string;
  contacts: ContactEntry[];
}

function isContactsFile(value: unknown): value is ContactsFile {
  return (
    typeof value === 'object' &&
    value !== null &&
    'format' in value &&
    value.format === 'sage-contacts' &&
    'network_id' in value &&
    typeof value.network_id === 'string' &&
    'contacts' in value &&
    Array.isArray(value.contacts)
  );
}

export function ContactsView() {
  const { wallet } = useWallet();
  const { addError } = useErrors();
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const debouncedSearch = useDebounce(search, 250);
  const [page, setPage] = useState(0);
  const [contacts, setContacts] = useState<ContactRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [loaded, setLoaded] = useState(false);
  const [editing, setEditing] = useState<ContactRecord | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [deleting, setDeleting] = useState<ContactRecord | null>(null);
  const [verifying, setVerifying] = useState<number | null>(null);
  const importRef = useRef<HTMLInputElement>(null);

  const refresh = useCallback(() => {
    if (!wallet) return;

    commands
      .getContacts({
        search: debouncedSearch.trim() || null,
        kind: contactKindFilter,
        offset: page * PAGE_SIZE,
        limit: PAGE_SIZE,
      })
      .then((data) => {
        setContacts(data.contacts);
        setTotal(data.total);
        setLoaded(true);
      })
      .catch((error) => addError(toCustomError(error)));
  }, [wallet, debouncedSearch, page, addError]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    setPage(0);
  }, [debouncedSearch]);

  // Sends update the address a handle was last sent to, and reorgs can make a
  // mapping stale, so the list is refreshed as the wallet syncs.
  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'coin_state') refresh();
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [refresh]);

  const verify = async (contact: ContactRecord) => {
    if (!contact.handle) return;

    setVerifying(contact.id);

    try {
      await commands.resolveHandle({ handle: contact.handle });
    } catch (error) {
      addError(toCustomError(error));
    } finally {
      setVerifying(null);
      refresh();
    }
  };

  const remove = async (contact: ContactRecord) => {
    try {
      await commands.deleteContact({ id: contact.id });
      const name = contact.name;
      toast.success(t`Deleted ${name}`);
    } catch (error) {
      addError(toCustomError(error));
    } finally {
      setDeleting(null);
      refresh();
    }
  };

  const exportContacts = async () => {
    try {
      const data = await commands.exportContacts({});

      if (data.contacts.length === 0) {
        toast.error(t`No contacts to export`);
        return;
      }

      const file: ContactsFile = {
        format: 'sage-contacts',
        version: 1,
        network_id: data.network_id,
        contacts: data.contacts,
      };

      if (
        await exportText(
          JSON.stringify(file, null, 2),
          `contacts-${data.network_id}`,
          ExportType.JSON,
        )
      ) {
        toast.success(t`Contacts exported`);
      }
    } catch (error) {
      addError(toCustomError(error));
    }
  };

  const importContacts = async (file: File) => {
    try {
      const parsed: unknown = JSON.parse(await file.text());

      if (!isContactsFile(parsed)) {
        toast.error(t`This file doesn't contain Sage contacts`);
        return;
      }

      const result = await commands.importContacts({
        network_id: parsed.network_id,
        contacts: parsed.contacts,
      });

      const added = result.added.length;
      const skipped = result.skipped.length;

      if (skipped > 0) {
        toast.warning(
          t`Imported ${added} contacts. ${skipped} were skipped because they were invalid or already saved.`,
        );
      } else {
        toast.success(t`Imported ${added} contacts`);
      }
    } catch (error) {
      if (error instanceof SyntaxError) {
        toast.error(t`This file doesn't contain Sage contacts`);
      } else {
        addError(toCustomError(error));
      }
    } finally {
      refresh();
    }
  };

  const pageCount = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div className='flex flex-col gap-4'>
      <input
        ref={importRef}
        type='file'
        accept='application/json,.json'
        className='hidden'
        onChange={(event) => {
          const file = event.target.files?.[0];
          event.target.value = '';
          if (file) importContacts(file);
        }}
      />

      <div className='flex flex-wrap items-center gap-2'>
        <div className='relative flex-1 min-w-48'>
          <SearchIcon
            className='absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground'
            aria-hidden='true'
          />
          <Input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={
              xchandlesEnabled
                ? t`Search by name, note, handle or address`
                : t`Search by name, note or address`
            }
            aria-label={t`Search contacts`}
            className='pl-8'
          />
        </div>
        <Button
          onClick={() => {
            setEditing(null);
            setDialogOpen(true);
          }}
        >
          <PlusIcon className='h-4 w-4 mr-2' aria-hidden='true' />
          <Trans>Add Contact</Trans>
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant='outline'
              size='icon'
              aria-label={t`More address book actions`}
            >
              <MoreVerticalIcon className='h-4 w-4' aria-hidden='true' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='end'>
            <DropdownMenuItem onClick={() => importRef.current?.click()}>
              <UploadIcon className='h-4 w-4 mr-2' aria-hidden='true' />
              <Trans>Import Contacts</Trans>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={exportContacts}>
              <DownloadIcon className='h-4 w-4 mr-2' aria-hidden='true' />
              <Trans>Export Contacts</Trans>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {loaded && contacts.length === 0 && (
        <div className='text-sm text-muted-foreground py-6 text-center'>
          {debouncedSearch.trim() ? (
            <Trans>No contacts match your search.</Trans>
          ) : (
            <Trans>
              You haven&apos;t saved any contacts yet. Contacts can be picked as
              recipients when you send.
            </Trans>
          )}
        </div>
      )}

      <div className='flex flex-col divide-y rounded-md border'>
        {contacts.map((contact) => (
          <ContactRow
            key={contact.id}
            contact={contact}
            verifying={verifying === contact.id}
            onSend={() =>
              navigate(
                `/wallet/send/xch?to=${encodeURIComponent(contact.recipient)}`,
              )
            }
            onVerify={() => verify(contact)}
            onEdit={() => {
              setEditing(contact);
              setDialogOpen(true);
            }}
            onDelete={() => setDeleting(contact)}
          />
        ))}
      </div>

      {total > PAGE_SIZE && (
        <div className='flex items-center justify-end gap-2 text-sm'>
          <Button
            variant='outline'
            size='sm'
            disabled={page === 0}
            onClick={() => setPage((page) => page - 1)}
          >
            <Trans>Previous</Trans>
          </Button>
          <span className='text-muted-foreground'>
            {page + 1} / {pageCount}
          </span>
          <Button
            variant='outline'
            size='sm'
            disabled={page + 1 >= pageCount}
            onClick={() => setPage((page) => page + 1)}
          >
            <Trans>Next</Trans>
          </Button>
        </div>
      )}

      <ContactDialog
        open={dialogOpen}
        setOpen={setDialogOpen}
        contact={editing}
        onSaved={refresh}
      />

      <Dialog
        open={!!deleting}
        onOpenChange={(open) => !open && setDeleting(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Delete Contact</Trans>
            </DialogTitle>
            <DialogDescription>
              {deleting?.kind === 'handle' ? (
                <Trans>
                  This removes the contact from your address book. Sage keeps
                  what it verified about the handle, so any change since you
                  last sent to it is still flagged.
                </Trans>
              ) : (
                <Trans>This removes the contact from your address book.</Trans>
              )}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className='gap-2'>
            <Button variant='outline' onClick={() => setDeleting(null)}>
              <Trans>Cancel</Trans>
            </Button>
            <Button
              variant='destructive'
              onClick={() => deleting && remove(deleting)}
            >
              <Trans>Delete</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

interface ContactRowProps {
  contact: ContactRecord;
  verifying: boolean;
  onSend: () => void;
  onVerify: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

function ContactRow({
  contact,
  verifying,
  onSend,
  onVerify,
  onEdit,
  onDelete,
}: ContactRowProps) {
  const state = contactState(contact);
  const resolution = contact.resolution;
  const lastVerified = resolution
    ? formatTimestamp(resolution.last_verified_at, 'medium', 'short')
    : null;
  const verifiedHeight = resolution?.verified_height.toLocaleString();
  const changedFrom = resolution?.changed_from;
  const lastSent = resolution?.last_sent_address;
  const name = contact.name;

  return (
    <div className='flex flex-col gap-2 p-3'>
      <div className='flex items-start justify-between gap-2'>
        <div className='min-w-0 flex flex-col gap-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <span className='font-medium break-words'>{contact.name}</span>
            {contact.kind === 'handle' && (
              <span className='font-mono text-sm text-muted-foreground'>
                {contact.recipient}
              </span>
            )}
            <ContactStatusBadge state={state} />
          </div>
          {contact.note && (
            <div className='text-sm text-muted-foreground break-words'>
              {contact.note}
            </div>
          )}
        </div>
        <div className='flex items-center gap-1 shrink-0'>
          <Button
            variant='outline'
            size='icon'
            onClick={onSend}
            aria-label={t`Send to ${name}`}
            title={t`Send`}
          >
            <SendIcon className='h-4 w-4' aria-hidden='true' />
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='ghost'
                size='icon'
                aria-label={t`Actions for ${name}`}
              >
                <MoreVerticalIcon className='h-4 w-4' aria-hidden='true' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              {contact.kind === 'handle' && (
                <DropdownMenuItem onClick={onVerify} disabled={verifying}>
                  <RefreshCwIcon
                    className={`h-4 w-4 mr-2 ${verifying ? 'animate-spin' : ''}`}
                    aria-hidden='true'
                  />
                  <Trans>Verify Now</Trans>
                </DropdownMenuItem>
              )}
              <DropdownMenuItem onClick={onEdit}>
                <PencilIcon className='h-4 w-4 mr-2' aria-hidden='true' />
                <Trans>Edit</Trans>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={onDelete}
                className='text-destructive focus:text-destructive'
              >
                <TrashIcon className='h-4 w-4 mr-2' aria-hidden='true' />
                <Trans>Delete</Trans>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {contact.address && (
        <div className='flex items-center gap-2'>
          <div className='min-w-0 flex-1 text-xs'>
            {contact.kind === 'handle' && (
              <span className='text-muted-foreground'>
                <Trans>Last verified address</Trans>{' '}
              </span>
            )}
            <span className='font-mono break-all'>{contact.address}</span>
          </div>
          <CopyButton
            value={contact.address}
            className='h-7 w-7 shrink-0'
            aria-label={t`Copy address`}
          />
        </div>
      )}

      {resolution && (
        <div className='text-xs text-muted-foreground'>
          <Trans>
            Last verified {lastVerified}, at height {verifiedHeight}
          </Trans>
          {resolution.verification_reason && (
            <span> · {resolution.verification_reason}</span>
          )}
        </div>
      )}

      {lastSent && lastSent !== contact.address && !changedFrom && (
        <div className='text-xs text-muted-foreground'>
          <Trans>
            Last sent to <span className='font-mono break-all'>{lastSent}</span>
          </Trans>
        </div>
      )}

      {changedFrom && (
        <div className='flex items-start gap-1.5 text-xs text-destructive'>
          <AlertTriangleIcon
            className='h-3.5 w-3.5 mt-0.5 shrink-0'
            aria-hidden='true'
          />
          <span className='break-words'>
            <Trans>
              This handle changed. It previously resolved to{' '}
              <span className='font-mono break-all'>{changedFrom}</span>.
              You&apos;ll be asked to confirm the new address before sending to
              it.
            </Trans>
          </span>
        </div>
      )}
    </div>
  );
}
