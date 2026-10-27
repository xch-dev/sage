import { commands, ContactRecord, HandleRecord } from '@/bindings';
import { useWallet } from '@/contexts/WalletContext';
import { useDebounce } from '@/hooks/useDebounce';
import { useErrors } from '@/hooks/useErrors';
import {
  contactKindFilter,
  contactState,
  handleState,
  matchContact,
} from '@/lib/contacts';
import { toCustomError } from '@/lib/errors';
import { xchandlesEnabled } from '@/lib/features';
import { isHandleInput, isValidHandle } from '@/lib/handles';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { BookUserIcon, UserIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Button } from '../ui/button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '../ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '../ui/popover';
import { ContactStatusBadge } from './ContactStatusBadge';

const PICKER_LIMIT = 50;
const RESOLVED_HANDLE_LIMIT = 100;

/** The most handle contacts loaded to tell which resolved handles are saved. */
const HANDLE_CONTACT_LIMIT = 1000;

interface ContactPickerProps {
  /**
   * Called with what to enter as the recipient: the contact's address, or its
   * handle written as `@handle`. Handles go through the same recipient
   * pipeline as a typed `@handle`, so they're resolved, verified and checked
   * for changes before anything is sent.
   */
  onSelect: (recipient: string) => void;
}

/**
 * Picks a recipient from the address book, or from the handles Sage has
 * already verified. This only fills in the recipient field, so the field's
 * preview and the confirmation show the resolved address as usual.
 */
export function ContactPicker({ onSelect }: ContactPickerProps) {
  const { wallet } = useWallet();
  const { addError } = useErrors();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const debouncedSearch = useDebounce(search, 200);
  const [contacts, setContacts] = useState<ContactRecord[]>([]);
  const [handles, setHandles] = useState<HandleRecord[]>([]);
  const [savedHandles, setSavedHandles] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!open || !wallet) return;

    let cancelled = false;

    commands
      .getContacts({
        search: debouncedSearch.trim() || null,
        kind: contactKindFilter,
        offset: 0,
        limit: PICKER_LIMIT,
      })
      .then((data) => {
        if (!cancelled) setContacts(data.contacts);
      })
      .catch((error) => addError(toCustomError(error)));

    return () => {
      cancelled = true;
    };
  }, [open, wallet, debouncedSearch, addError]);

  useEffect(() => {
    if (!open || !wallet || !xchandlesEnabled) return;

    let cancelled = false;

    // Every handle contact, not just the ones on the current page of search
    // results, so that a saved handle is never offered as an unsaved one.
    commands
      .getContacts({
        search: null,
        kind: 'handle',
        offset: 0,
        limit: HANDLE_CONTACT_LIMIT,
      })
      .then((data) => {
        if (!cancelled) {
          setSavedHandles(
            new Set(
              data.contacts.flatMap((contact) =>
                contact.handle ? [contact.handle] : [],
              ),
            ),
          );
        }
      })
      .catch(() => {
        if (!cancelled) setSavedHandles(new Set());
      });

    // Only mappings which are verified and unexpired are offered. Networks
    // without XCHandles fail here, and simply have no handles to offer.
    commands
      .getHandles({
        filter: 'all',
        include_unverified: false,
        refresh: false,
        offset: 0,
        limit: RESOLVED_HANDLE_LIMIT,
      })
      .then((data) => {
        if (!cancelled) setHandles(data.handles);
      })
      .catch(() => {
        if (!cancelled) setHandles([]);
      });

    return () => {
      cancelled = true;
    };
  }, [open, wallet]);

  const query = debouncedSearch.trim().replace(/^@/, '').toLowerCase();
  const otherHandles = handles.filter(
    (record) =>
      !savedHandles.has(record.handle) &&
      (record.handle.includes(query) || record.address === query),
  );

  const select = (recipient: string) => {
    onSelect(recipient);
    setOpen(false);
    setSearch('');
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type='button'
          variant='ghost'
          size='sm'
          className='h-auto py-0.5 px-1.5 text-xs text-muted-foreground hover:text-foreground gap-1'
          aria-label={t`Insert a recipient from your contacts`}
        >
          <BookUserIcon className='h-3 w-3' aria-hidden='true' />
          <Trans>Contacts</Trans>
        </Button>
      </PopoverTrigger>
      <PopoverContent className='w-80 p-0' align='end'>
        <Command shouldFilter={false}>
          <CommandInput
            value={search}
            onValueChange={setSearch}
            placeholder={t`Search contacts`}
          />
          <CommandList>
            <CommandEmpty>
              <div className='px-2 text-sm text-muted-foreground'>
                <Trans>
                  No contacts found. You can add contacts on the Addresses page.
                </Trans>
              </div>
            </CommandEmpty>
            {contacts.length > 0 && (
              <CommandGroup heading={t`Contacts`}>
                {contacts.map((contact) => (
                  <CommandItem
                    key={`contact-${contact.id}`}
                    value={`contact-${contact.id}`}
                    onSelect={() => select(contact.recipient)}
                    className='flex flex-col items-start gap-0.5'
                  >
                    <div className='flex w-full items-center justify-between gap-2'>
                      <span className='truncate font-medium'>
                        {contact.name}
                      </span>
                      <ContactStatusBadge state={contactState(contact)} />
                    </div>
                    <span className='w-full truncate font-mono text-xs text-muted-foreground'>
                      {contact.recipient}
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            )}
            {otherHandles.length > 0 && (
              <CommandGroup heading={t`Resolved handles`}>
                {otherHandles.map((record) => (
                  <CommandItem
                    key={`handle-${record.handle}`}
                    value={`handle-${record.handle}`}
                    onSelect={() => select(`@${record.handle}`)}
                    className='flex flex-col items-start gap-0.5'
                  >
                    <div className='flex w-full items-center justify-between gap-2'>
                      <span className='truncate font-medium'>
                        @{record.handle}
                      </span>
                      <ContactStatusBadge state={handleState(record)} />
                    </div>
                    <span className='w-full truncate font-mono text-xs text-muted-foreground'>
                      {record.address}
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

/**
 * Names the contact a recipient field refers to, so that a pasted address or
 * handle which is already saved is recognizable. The handle preview below the
 * field still shows the address it resolves to.
 */
export function RecipientContact({ value }: { value: string }) {
  const recipient = useDebounce(value.trim(), 300);
  const [contact, setContact] = useState<ContactRecord | null>(null);

  const isHandle = isHandleInput(recipient);
  const searchable =
    recipient.length > 0 &&
    (isHandle ? xchandlesEnabled && isValidHandle(recipient) : true);

  useEffect(() => {
    if (!searchable) {
      setContact(null);
      return;
    }

    let cancelled = false;

    commands
      .getContacts({
        search: recipient,
        kind: contactKindFilter,
        offset: 0,
        limit: 10,
      })
      .then((data) => {
        if (!cancelled) {
          setContact(matchContact(data.contacts, recipient) ?? null);
        }
      })
      .catch(() => {
        if (!cancelled) setContact(null);
      });

    return () => {
      cancelled = true;
    };
  }, [recipient, searchable]);

  if (!contact || contact.recipient !== value.trim()) return null;

  const name = contact.name;

  return (
    <div className='flex items-center gap-1.5 text-xs text-muted-foreground'>
      <UserIcon className='h-3.5 w-3.5 shrink-0' aria-hidden='true' />
      <span className='truncate'>
        <Trans>
          Contact: <span className='font-medium text-foreground'>{name}</span>
        </Trans>
      </span>
    </div>
  );
}
