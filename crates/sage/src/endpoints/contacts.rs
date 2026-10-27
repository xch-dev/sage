//! The address book.
//!
//! A contact names a raw address or an `XCHandles` handle. For a handle, only the handle is
//! stored with the contact: the address it resolves to and its history (the address it
//! changed from and the address last sent to) are read from the verified mapping store, so
//! sending to a handle contact goes through the recipient pipeline like any other `@handle`,
//! including its change confirmation. Nothing here makes a network request.

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{prelude::*, utils::Address};
use sage_api::{
    AddContact, AddContactResponse, AddressContactsRecord, ContactEntry, ContactKind,
    ContactRecord, ContactSkipReason, DeleteContact, DeleteContactResponse, ExportContacts,
    ExportContactsResponse, GetContact, GetContactResponse, GetContacts, GetContactsResponse,
    ImportContacts, ImportContactsResponse, LookupAddressContacts, LookupAddressContactsResponse,
    SkippedContact, UpdateContact, UpdateContactResponse,
};
use sage_database::{
    ContactQuery, ContactRow, ContactTarget, DatabaseTx, contact_name_key, normalize_contact_name,
};
use sage_wallet::{Handle, HandleResolution, XchandlesError};

use crate::{Error, Result, Sage};

/// The longest a contact's name may be, in characters.
pub const MAX_CONTACT_NAME_LENGTH: usize = 64;

/// The longest a contact's note may be, in characters.
pub const MAX_CONTACT_NOTE_LENGTH: usize = 1000;

/// The most contacts which can be imported at once.
pub const MAX_IMPORT_CONTACTS: usize = 10_000;

/// The most addresses `lookup_address_contacts` accepts at once.
const MAX_LOOKUP_ADDRESSES: usize = 1000;

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

/// Whether a character changes how text around it is displayed without being
/// visible itself: bidirectional overrides, embeddings and isolates.
fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Whether a character is invisible, or reorders the text around it, and has no
/// use in a name. Joiners and variation selectors are needed by emoji and some
/// scripts, so they're allowed, and are instead ignored when names are compared
/// (see `contact_name_key`).
fn is_hidden(c: char) -> bool {
    is_bidi_control(c)
        || matches!(
            c,
            '\u{115F}'
                | '\u{1160}'
                | '\u{17B4}'
                | '\u{17B5}'
                | '\u{180E}'
                | '\u{200B}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{2060}'..='\u{2065}'
                | '\u{206A}'..='\u{206F}'
                | '\u{2800}'
                | '\u{3164}'
                | '\u{FEFF}'
                | '\u{FFA0}'
                | '\u{FFF0}'..='\u{FFFB}'
                | '\u{FDD0}'..='\u{FDEF}'
                | '\u{1BCA0}'..='\u{1BCA3}'
                | '\u{1D173}'..='\u{1D17A}'
                | '\u{E0080}'..='\u{E00FF}'
                | '\u{E01F0}'..='\u{E0FFF}'
        )
        || (c as u32) & 0xFFFE == 0xFFFE
}

/// Trims a contact's name, and checks that it's one line of a reasonable length,
/// without invisible characters which could make it look like another name.
fn contact_name(name: &str) -> Result<String> {
    let name = normalize_contact_name(name.trim());

    if name.is_empty() {
        return Err(Error::InvalidContact("the name is empty".to_string()));
    }

    if name.chars().count() > MAX_CONTACT_NAME_LENGTH {
        return Err(Error::InvalidContact(format!(
            "the name is longer than {MAX_CONTACT_NAME_LENGTH} characters"
        )));
    }

    if name.chars().any(char::is_control) {
        return Err(Error::InvalidContact(
            "the name contains control characters".to_string(),
        ));
    }

    if name.chars().any(is_hidden) {
        return Err(Error::InvalidContact(
            "the name contains invisible characters".to_string(),
        ));
    }

    if contact_name_key(&name).is_empty() {
        return Err(Error::InvalidContact(
            "the name has no visible characters".to_string(),
        ));
    }

    Ok(name)
}

/// Trims a contact's note, treating an empty note as none. Notes can have more
/// than one line, but no other control characters, or characters which reorder
/// the text around them.
fn contact_note(note: Option<&str>) -> Result<Option<String>> {
    let Some(note) = note
        .map(|note| normalize_contact_name(note.trim()))
        .filter(|note| !note.is_empty())
    else {
        return Ok(None);
    };

    if note.chars().count() > MAX_CONTACT_NOTE_LENGTH {
        return Err(Error::InvalidContact(format!(
            "the note is longer than {MAX_CONTACT_NOTE_LENGTH} characters"
        )));
    }

    if note
        .chars()
        .any(|c| (c.is_control() && c != '\n' && c != '\t') || is_bidi_control(c))
    {
        return Err(Error::InvalidContact(
            "the note contains control characters".to_string(),
        ));
    }

    Ok(Some(note))
}

/// A validated contact, ready to be stored.
struct NewContact {
    name: String,
    note: Option<String>,
    target: ContactTarget,
}

/// The existing contact a new contact would duplicate, if any. A recipient can only belong to
/// one contact, and no two names can look the same (see `contact_name_key`).
async fn find_duplicate(
    tx: &mut DatabaseTx<'_>,
    contact: &NewContact,
) -> Result<Option<(ContactSkipReason, ContactRow)>> {
    if let Some(existing) = tx.contact_by_target(&contact.target).await? {
        return Ok(Some((ContactSkipReason::DuplicateRecipient, existing)));
    }

    if let Some(existing) = tx.contact_by_name(&contact.name).await? {
        return Ok(Some((ContactSkipReason::DuplicateName, existing)));
    }

    Ok(None)
}

impl Sage {
    pub async fn get_contacts(&self, req: GetContacts) -> Result<GetContactsResponse> {
        let wallet = self.wallet()?;
        let registry_launcher_id = self.contact_registry();

        // Names and notes are stored composed, so the search is too, to match them however
        // its accents were typed.
        let search = req
            .search
            .as_deref()
            .map(|search| normalize_contact_name(search.trim()))
            .filter(|search| !search.is_empty());
        let search = search.as_deref();

        // An address or puzzle hash matches the contacts which name it, and `@handle` matches
        // by the handle.
        let puzzle_hash = search.and_then(|search| self.parse_lookup_address(search).ok());
        let text = search.map(|search| search.strip_prefix('@').unwrap_or(search));

        let (rows, total) = wallet
            .db
            .contacts_page(
                ContactQuery {
                    search: text,
                    puzzle_hash,
                    registry_launcher_id,
                    kind: req.kind.map(|kind| match kind {
                        ContactKind::Address => sage_database::ContactKind::Address,
                        ContactKind::Handle => sage_database::ContactKind::Handle,
                    }),
                },
                req.limit,
                req.offset,
            )
            .await?;

        let now = unix_now();
        let mut contacts = Vec::with_capacity(rows.len());

        for row in rows {
            contacts.push(self.contact_record(row, now).await?);
        }

        Ok(GetContactsResponse { contacts, total })
    }

    pub async fn get_contact(&self, req: GetContact) -> Result<GetContactResponse> {
        let wallet = self.wallet()?;

        let row = wallet
            .db
            .contact(req.id)
            .await?
            .ok_or(Error::MissingContact(req.id))?;

        Ok(GetContactResponse {
            contact: self.contact_record(row, unix_now()).await?,
        })
    }

    pub async fn add_contact(&self, req: AddContact) -> Result<AddContactResponse> {
        let wallet = self.wallet()?;

        let contact = self.new_contact(&ContactEntry {
            name: req.name,
            recipient: req.recipient,
            note: req.note,
        })?;

        let now = unix_now();
        let mut tx = wallet.db.tx().await?;

        match find_duplicate(&mut tx, &contact).await? {
            Some((ContactSkipReason::DuplicateName, existing)) => {
                return Err(Error::DuplicateContactName {
                    name: existing.name,
                    id: existing.id,
                });
            }
            Some((_, existing)) => {
                return Err(Error::DuplicateContactRecipient {
                    recipient: self.contact_recipient(&contact.target)?,
                    name: existing.name,
                    id: existing.id,
                });
            }
            None => {}
        }

        let id = tx
            .insert_contact(&contact.name, contact.note.as_deref(), &contact.target, now)
            .await?;

        tx.commit().await?;

        self.get_contact(GetContact { id })
            .await
            .map(|response| AddContactResponse {
                contact: response.contact,
            })
    }

    pub async fn update_contact(&self, req: UpdateContact) -> Result<UpdateContactResponse> {
        let wallet = self.wallet()?;
        let name = contact_name(&req.name)?;
        let note = contact_note(req.note.as_deref())?;

        let mut tx = wallet.db.tx().await?;

        if tx.contact(req.id).await?.is_none() {
            return Err(Error::MissingContact(req.id));
        }

        if let Some(existing) = tx.contact_by_name(&name).await?
            && existing.id != req.id
        {
            return Err(Error::DuplicateContactName {
                name: existing.name,
                id: existing.id,
            });
        }

        tx.update_contact(req.id, &name, note.as_deref(), unix_now())
            .await?;
        tx.commit().await?;

        self.get_contact(GetContact { id: req.id })
            .await
            .map(|response| UpdateContactResponse {
                contact: response.contact,
            })
    }

    /// Deletes a contact. A handle's mapping and history stay in the store, so adding the
    /// contact again can't hide a change.
    pub async fn delete_contact(&self, req: DeleteContact) -> Result<DeleteContactResponse> {
        let wallet = self.wallet()?;

        if !wallet.db.delete_contact(req.id).await? {
            return Err(Error::MissingContact(req.id));
        }

        Ok(DeleteContactResponse {})
    }

    pub async fn lookup_address_contacts(
        &self,
        req: LookupAddressContacts,
    ) -> Result<LookupAddressContactsResponse> {
        let wallet = self.wallet()?;
        let registry_launcher_id = self.contact_registry();

        if req.addresses.len() > MAX_LOOKUP_ADDRESSES {
            return Err(Error::TooManyAddresses(MAX_LOOKUP_ADDRESSES));
        }

        let puzzle_hashes = req
            .addresses
            .iter()
            .map(|address| self.parse_lookup_address(address))
            .collect::<Result<Vec<_>>>()?;

        let now = unix_now();
        let mut contacts = HashMap::new();

        for &puzzle_hash in &puzzle_hashes {
            if contacts.contains_key(&puzzle_hash) {
                continue;
            }

            let mut records = Vec::new();

            for row in wallet
                .db
                .contacts_by_puzzle_hash(registry_launcher_id, puzzle_hash, now)
                .await?
            {
                records.push(self.contact_record(row, now).await?);
            }

            contacts.insert(puzzle_hash, records);
        }

        Ok(LookupAddressContactsResponse {
            results: req
                .addresses
                .into_iter()
                .zip(puzzle_hashes)
                .map(|(address, puzzle_hash)| AddressContactsRecord {
                    address,
                    contacts: contacts[&puzzle_hash].clone(),
                })
                .collect(),
        })
    }

    pub async fn export_contacts(&self, _req: ExportContacts) -> Result<ExportContactsResponse> {
        let wallet = self.wallet()?;

        let contacts = wallet
            .db
            .all_contacts()
            .await?
            .into_iter()
            .map(|row| {
                Ok(ContactEntry {
                    recipient: self.contact_recipient(&row.target)?,
                    name: row.name,
                    note: row.note,
                })
            })
            .collect::<Result<_>>()?;

        Ok(ExportContactsResponse {
            network_id: self.network_id(),
            contacts,
        })
    }

    /// Adds exported contacts. Existing contacts are never changed, so an imported file
    /// can't rename or replace a contact the user already trusts. Entries which conflict with
    /// an existing contact, or with an earlier entry, are skipped and reported.
    pub async fn import_contacts(&self, req: ImportContacts) -> Result<ImportContactsResponse> {
        let wallet = self.wallet()?;

        if req.network_id != self.network_id() {
            return Err(Error::ContactNetworkMismatch(req.network_id));
        }

        if req.contacts.len() > MAX_IMPORT_CONTACTS {
            return Err(Error::InvalidContact(format!(
                "at most {MAX_IMPORT_CONTACTS} contacts can be imported at once"
            )));
        }

        let now = unix_now();
        let mut tx = wallet.db.tx().await?;
        let mut added = Vec::new();
        let mut skipped = Vec::new();

        for entry in req.contacts {
            let Ok(contact) = self.new_contact(&entry) else {
                skipped.push(SkippedContact {
                    entry,
                    reason: ContactSkipReason::Invalid,
                    existing_id: None,
                });
                continue;
            };

            if let Some((reason, existing)) = find_duplicate(&mut tx, &contact).await? {
                skipped.push(SkippedContact {
                    entry,
                    reason,
                    existing_id: Some(existing.id),
                });
                continue;
            }

            added.push(
                tx.insert_contact(&contact.name, contact.note.as_deref(), &contact.target, now)
                    .await?,
            );
        }

        tx.commit().await?;

        let mut records = Vec::with_capacity(added.len());

        for id in added {
            if let Some(row) = wallet.db.contact(id).await? {
                records.push(self.contact_record(row, now).await?);
            }
        }

        Ok(ImportContactsResponse {
            added: records,
            skipped,
        })
    }

    /// The registry handle contacts are resolved against, if this network supports
    /// `XCHandles`.
    fn contact_registry(&self) -> Option<Bytes32> {
        self.network()
            .xchandles()
            .map(|config| config.registry_launcher_id)
    }

    /// Validates a contact. Handles are only checked for syntax, and only accepted on networks
    /// which support `XCHandles`. Addresses must be for the current network.
    fn new_contact(&self, entry: &ContactEntry) -> Result<NewContact> {
        let name = contact_name(&entry.name)?;
        let note = contact_note(entry.note.as_deref())?;
        let recipient = entry.recipient.trim();

        let target = match Handle::from_input(recipient) {
            Some(handle) => {
                let handle = handle.map_err(XchandlesError::from)?;

                if self.contact_registry().is_none() {
                    return Err(XchandlesError::UnsupportedNetwork.into());
                }

                ContactTarget::Handle(handle.as_str().to_string())
            }
            None => ContactTarget::Address(self.parse_address(recipient.to_string())?),
        };

        Ok(NewContact { name, note, target })
    }

    fn contact_recipient(&self, target: &ContactTarget) -> Result<String> {
        Ok(match target {
            ContactTarget::Address(puzzle_hash) => {
                Address::new(*puzzle_hash, self.network().prefix()).encode()?
            }
            ContactTarget::Handle(handle) => format!("@{handle}"),
        })
    }

    async fn contact_record(&self, row: ContactRow, now: u64) -> Result<ContactRecord> {
        let wallet = self.wallet()?;
        let recipient = self.contact_recipient(&row.target)?;

        let (kind, handle, address, resolution) = match row.target {
            ContactTarget::Address(..) => {
                (ContactKind::Address, None, Some(recipient.clone()), None)
            }
            ContactTarget::Handle(handle) => {
                let resolution = match self.contact_registry() {
                    Some(registry_launcher_id) => wallet
                        .db
                        .xchandle(registry_launcher_id, &handle)
                        .await?
                        .map(|row| {
                            self.handle_record_at(&HandleResolution { row, cached: true }, now)
                        })
                        .transpose()?,
                    None => None,
                };

                (
                    ContactKind::Handle,
                    Some(handle),
                    resolution.as_ref().map(|record| record.address.clone()),
                    resolution,
                )
            }
        };

        Ok(ContactRecord {
            id: row.id,
            name: row.name,
            note: row.note,
            kind,
            recipient,
            handle,
            address,
            resolution,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_and_validated() {
        assert_eq!(contact_name("  Alice ").unwrap(), "Alice");
        // Names are stored composed, however their accents were typed.
        assert_eq!(contact_name("Jose\u{0301}").unwrap(), "Jos\u{00E9}");
        assert!(contact_name("   ").is_err());
        assert!(contact_name("a\nb").is_err());
        assert!(contact_name(&"a".repeat(MAX_CONTACT_NAME_LENGTH)).is_ok());
        assert!(contact_name(&"a".repeat(MAX_CONTACT_NAME_LENGTH + 1)).is_err());
    }

    #[test]
    fn names_cant_hide_characters() {
        // Each of these hides a character, or reorders the name.
        for name in [
            "Alice\u{200B}",
            "Alice\u{FEFF}",
            "Alice\u{2060}",
            "\u{202E}ecilA",
            "Alice\u{2067}",
            "Alice\u{2800}",
            "Alice\u{3164}",
            "Alice\u{E0080}",
            "Alice\u{1BCA0}",
            "Alice\u{2028}Smith",
            "\u{200B}",
            // Only ignorable characters, so nothing visible.
            "\u{200D}\u{FE0F}",
        ] {
            assert!(contact_name(name).is_err(), "{name:?} was accepted");
        }

        // Emoji and scripts which need joiners or selectors are allowed. Names which only
        // differ by these are rejected as duplicates instead.
        for name in [
            "Zoë",
            "Ali 🐱",
            "محمد",
            "Mom \u{2764}\u{FE0F}",
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} family",
            "\u{1F3F3}\u{FE0F}\u{200D}\u{1F308}",
            "\u{2764}\u{FE0F}\u{200D}\u{1F525}",
            "\u{1F575}\u{FE0F}\u{200D}\u{2642}\u{FE0F}",
            "1\u{FE0F}\u{20E3}",
            "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}",
            "\u{1F469}\u{1F3FD}\u{200D}\u{1F4BB}",
            "\u{0645}\u{06CC}\u{200C}\u{062E}\u{0648}\u{0627}\u{0647}\u{0645}",
            "\u{0915}\u{094D}\u{200D}\u{0937}",
            "\u{0DC1}\u{0DCA}\u{200D}\u{0DBB}\u{0DD3}",
            "山田\u{3000}太郎",
        ] {
            assert!(contact_name(name).is_ok(), "{name:?} was rejected");
        }
    }

    #[test]
    fn empty_notes_are_cleared() {
        assert_eq!(contact_note(None).unwrap(), None);
        assert_eq!(contact_note(Some("  ")).unwrap(), None);
        assert_eq!(contact_note(Some(" hi ")).unwrap().as_deref(), Some("hi"));
        assert!(contact_note(Some(&"a".repeat(MAX_CONTACT_NOTE_LENGTH + 1))).is_err());
        assert!(contact_note(Some("line one\nline\ttwo")).is_ok());
        assert!(contact_note(Some("pay \u{202E}1234")).is_err());
        assert!(contact_note(Some("a\u{7}b")).is_err());
    }
}
