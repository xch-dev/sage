import { ContactRecord } from '@/bindings';
import { ContactStatusBadge } from '@/components/contacts/ContactStatusBadge';
import { Button } from '@/components/ui/button';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { useAddressContacts } from '@/hooks/useAddressContacts';
import { handleTrust } from '@/lib/addressHandles';
import { contactState } from '@/lib/contacts';
import { cn } from '@/lib/utils';
import { plural, t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { AlertTriangleIcon, UserIcon } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

/**
 * Whether a contact's label needs a warning. Address contacts are saved
 * addresses, so they never do. Handle contacts only label the address their
 * handle was last verified as resolving to, which may be out of date, or may
 * have changed hands since the user last used it.
 */
function needsWarning(contact: ContactRecord): boolean {
  return (
    contact.kind === 'handle' &&
    (!contact.resolution || handleTrust(contact.resolution) !== 'verified')
  );
}

export interface AddressContactsProps {
  address: string | null | undefined;
  /** A smaller label, for tables and dense lists. */
  compact?: boolean;
  className?: string;
}

/**
 * Labels an address with the names it's saved as in the address book. Every
 * contact for the address is listed in a popover, with how far it can be
 * relied on. The address itself is always shown by the caller, since a
 * contact name never replaces it.
 */
export function AddressContacts({
  address,
  compact = false,
  className,
}: AddressContactsProps) {
  const contacts = useAddressContacts(address);

  if (!address || contacts.length === 0) return null;

  const [primary, ...others] = contacts;
  const name = primary.name;
  const count = others.length;
  const warning = contacts.some(needsWarning);

  return (
    <Popover>
      <PopoverTrigger asChild>
        <button
          type='button'
          className={cn(
            'inline-flex max-w-full items-center gap-1 rounded-md border bg-background font-medium leading-none hover:bg-accent focus:outline-none focus-visible:ring-2 focus-visible:ring-ring',
            compact
              ? 'max-w-[12rem] px-1 py-0.5 text-xs'
              : 'px-1.5 py-1 text-xs',
            warning
              ? 'border-dashed border-amber-500/70 text-amber-700 dark:text-amber-400'
              : 'text-foreground',
            className,
          )}
          title={t`Contact: ${name}`}
          aria-label={
            count > 0
              ? plural(count, {
                  one: `Contact: ${name}, and # more contact. Show all contacts for this address`,
                  other: `Contact: ${name}, and # more contacts. Show all contacts for this address`,
                })
              : t`Contact: ${name}. Show contact details`
          }
          onClick={(event) => event.stopPropagation()}
        >
          {warning ? (
            <AlertTriangleIcon className='h-3.5 w-3.5 shrink-0' aria-hidden />
          ) : (
            <UserIcon className='h-3.5 w-3.5 shrink-0' aria-hidden />
          )}
          <span className='truncate'>{name}</span>
          {count > 0 && (
            <span className='shrink-0 text-muted-foreground'>+{count}</span>
          )}
        </button>
      </PopoverTrigger>
      <PopoverContent
        className='w-80 space-y-3 p-3'
        align='start'
        onClick={(event) => event.stopPropagation()}
      >
        <div className='text-sm font-medium'>
          {contacts.length > 1 ? (
            <Trans>Saved contacts for this address</Trans>
          ) : (
            <Trans>Saved contact for this address</Trans>
          )}
        </div>

        <div className='max-h-72 space-y-2 overflow-y-auto'>
          {contacts.map((contact) => (
            <ContactDetails key={contact.id} contact={contact} />
          ))}
        </div>

        <OpenContactsButton />
      </PopoverContent>
    </Popover>
  );
}

/** Only rendered once the popover is open, inside the app's router. */
function OpenContactsButton() {
  const navigate = useNavigate();

  return (
    <Button
      variant='outline'
      size='sm'
      className='h-7 w-full text-xs'
      onClick={() => navigate('/addresses?tab=contacts')}
    >
      <Trans>Open Contacts</Trans>
    </Button>
  );
}

function ContactDetails({ contact }: { contact: ContactRecord }) {
  const handle = contact.handle;

  return (
    <div className='space-y-1 rounded-md border p-2 text-xs'>
      <div className='flex items-center justify-between gap-2'>
        <span className='min-w-0 truncate font-medium'>{contact.name}</span>
        <ContactStatusBadge state={contactState(contact)} />
      </div>

      {handle ? (
        <div className='text-muted-foreground'>
          {needsWarning(contact) ? (
            <Trans>
              Saved as @{handle}. It may not resolve here anymore, so it will be
              verified again before you send to it.
            </Trans>
          ) : (
            <Trans>Saved as @{handle}, which resolves to this address</Trans>
          )}
        </div>
      ) : (
        <div className='text-muted-foreground'>
          <Trans>Saved as this address</Trans>
        </div>
      )}

      {contact.note && (
        <div className='break-words text-muted-foreground'>{contact.note}</div>
      )}
    </div>
  );
}
