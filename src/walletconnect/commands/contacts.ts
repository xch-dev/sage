import { commands, ContactRecord } from '@/bindings';
import { refreshAddressLabels } from '@/lib/addressContacts';
import { xchandlesEnabled } from '@/lib/features';
import { isHandleInput } from '@/lib/handles';
import { t } from '@lingui/core/macro';
import { Params, Return } from '../commands';
import { toRecord } from './xchandles';

type WalletConnectContact = Return<'chia_addContact'>;

function toContact(contact: ContactRecord): WalletConnectContact {
  return {
    id: contact.id,
    name: contact.name,
    note: contact.note,
    kind: contact.kind,
    recipient: contact.recipient,
    handle: contact.handle,
    address: contact.address,
    resolution: contact.resolution ? toRecord(contact.resolution) : null,
    createdAt: contact.created_at,
    updatedAt: contact.updated_at,
  };
}

export async function handleGetContacts(
  params: Params<'chia_getContacts'>,
): Promise<Return<'chia_getContacts'>> {
  const data = await commands.getContacts({
    search: params?.search ?? null,
    // Builds without XCHandles only show address contacts, so sites only get
    // what the user can see.
    kind: xchandlesEnabled ? null : 'address',
    offset: params?.offset ?? 0,
    limit: params?.limit ?? 50,
  });

  return {
    contacts: data.contacts.map(toContact),
    total: data.total,
  };
}

export async function handleAddContact(
  params: Params<'chia_addContact'>,
): Promise<Return<'chia_addContact'>> {
  if (!xchandlesEnabled && isHandleInput(params.recipient)) {
    throw new Error(t`XCHandles is not enabled in this build of Sage`);
  }

  const data = await commands.addContact({
    name: params.name,
    recipient: params.recipient,
    note: params.note ?? null,
  });

  refreshAddressLabels();

  return toContact(data.contact);
}
