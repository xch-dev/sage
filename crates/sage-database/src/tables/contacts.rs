use chia_wallet_sdk::prelude::*;
use sqlx::SqliteExecutor;
use unicode_normalization::UnicodeNormalization;

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ContactKind {
    Address = 0,
    Handle = 1,
}

impl TryFrom<i64> for ContactKind {
    type Error = DatabaseError;

    fn try_from(value: i64) -> Result<Self> {
        Ok(match value {
            0 => Self::Address,
            1 => Self::Handle,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

/// What a contact names. This is the contact's identity, and doesn't change.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContactTarget {
    /// A raw address, by its puzzle hash.
    Address(Bytes32),
    /// An `XCHandles` handle, without the leading `@`. The address it resolves to is only ever
    /// read from the `xchandles` table.
    Handle(String),
}

impl ContactTarget {
    pub fn kind(&self) -> ContactKind {
        match self {
            Self::Address(..) => ContactKind::Address,
            Self::Handle(..) => ContactKind::Handle,
        }
    }

    fn columns(&self) -> (Option<&[u8]>, Option<&str>) {
        match self {
            Self::Address(puzzle_hash) => (Some(puzzle_hash.as_ref()), None),
            Self::Handle(handle) => (None, Some(handle.as_str())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactRow {
    pub id: u32,
    pub name: String,
    pub note: Option<String>,
    pub target: ContactTarget,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Which contacts to list.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ContactQuery<'a> {
    /// Only contacts whose name, note or handle contains this text, ignoring ASCII case.
    pub search: Option<&'a str>,
    /// Also match contacts for this puzzle hash: address contacts for it, and handle contacts
    /// whose stored mapping in `registry_launcher_id` resolves to it.
    pub puzzle_hash: Option<Bytes32>,
    pub registry_launcher_id: Option<Bytes32>,
    pub kind: Option<ContactKind>,
}

impl Database {
    pub async fn contact(&self, id: u32) -> Result<Option<ContactRow>> {
        contact(&self.pool, id).await
    }

    /// A page of contacts sorted by name, and the total number matching.
    pub async fn contacts_page(
        &self,
        query: ContactQuery<'_>,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<ContactRow>, u32)> {
        contacts_page(&self.pool, query, limit, offset).await
    }

    /// Every contact, sorted by name.
    pub async fn all_contacts(&self) -> Result<Vec<ContactRow>> {
        all_contacts(&self.pool).await
    }

    /// Contacts which name the given puzzle hash: address contacts for it, and handle contacts
    /// whose mapping in the registry is verified, unexpired by `now`, and resolves to it. Whether
    /// such a mapping is still recent enough to rely on is left to the caller.
    pub async fn contacts_by_puzzle_hash(
        &self,
        registry_launcher_id: Option<Bytes32>,
        puzzle_hash: Bytes32,
        now: u64,
    ) -> Result<Vec<ContactRow>> {
        contacts_by_puzzle_hash(&self.pool, registry_launcher_id, puzzle_hash, now).await
    }

    pub async fn delete_contact(&self, id: u32) -> Result<bool> {
        delete_contact(&self.pool, id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn contact(&mut self, id: u32) -> Result<Option<ContactRow>> {
        contact(&mut *self.tx, id).await
    }

    pub async fn contact_by_target(
        &mut self,
        target: &ContactTarget,
    ) -> Result<Option<ContactRow>> {
        contact_by_target(&mut *self.tx, target).await
    }

    /// The contact whose name looks the same as this one, as compared by [`contact_name_key`].
    pub async fn contact_by_name(&mut self, name: &str) -> Result<Option<ContactRow>> {
        contact_by_name(&mut *self.tx, name).await
    }

    pub async fn insert_contact(
        &mut self,
        name: &str,
        note: Option<&str>,
        target: &ContactTarget,
        now: u64,
    ) -> Result<u32> {
        insert_contact(&mut *self.tx, name, note, target, now).await
    }

    pub async fn update_contact(
        &mut self,
        id: u32,
        name: &str,
        note: Option<&str>,
        now: u64,
    ) -> Result<bool> {
        update_contact(&mut *self.tx, id, name, note, now).await
    }
}

/// Whether a character is skipped when comparing names, since it doesn't change how a name
/// looks on its own: zero-width joiners and non-joiners, variation selectors, tag characters,
/// directional marks, the combining grapheme joiner and soft hyphens. These are needed by
/// emoji and some scripts, so they're allowed in names, but never make a name distinct.
fn is_ignorable(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'
            | '\u{034F}'
            | '\u{061C}'
            | '\u{180B}'..='\u{180F}'
            | '\u{200C}'..='\u{200F}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{E0000}'..='\u{E007F}'
            | '\u{E0100}'..='\u{E01EF}'
    )
}

/// A contact's name in the form it's stored: Unicode NFC, so that a name is stored the same
/// way whether its accents were typed as separate combining marks or not.
pub fn normalize_contact_name(name: &str) -> String {
    name.nfc().collect()
}

/// The form of a contact's name used to tell whether two names look the same: lowercased,
/// with runs of whitespace collapsed to one space, ignorable characters removed, and in
/// Unicode NFC, so that combining marks are composed and in a consistent order. Two contacts
/// can't have names with the same key.
pub fn contact_name_key(name: &str) -> String {
    let mut key = String::with_capacity(name.len());
    let mut space = false;

    for c in name.chars().filter(|&c| !is_ignorable(c)) {
        if c.is_whitespace() {
            space = !key.is_empty();
            continue;
        }

        if space {
            key.push(' ');
            space = false;
        }

        key.extend(c.to_lowercase());
    }

    key.nfc().collect()
}

macro_rules! contact_row {
    ($row:expr) => {{
        let row = $row;
        let target = match ContactKind::try_from(row.kind)? {
            ContactKind::Address => ContactTarget::Address(
                row.puzzle_hash
                    .ok_or(DatabaseError::InvalidAddress)?
                    .convert()?,
            ),
            ContactKind::Handle => {
                ContactTarget::Handle(row.handle.ok_or(DatabaseError::InvalidEnumVariant)?)
            }
        };

        Ok(ContactRow {
            id: row.id.convert()?,
            name: row.name,
            note: row.note,
            target,
            created_at: row.created_at.convert()?,
            updated_at: row.updated_at.convert()?,
        })
    }};
}

/// Escapes text for a `LIKE` pattern using `\` as the escape character.
fn like_pattern(search: &str) -> String {
    let mut pattern = String::with_capacity(search.len() + 2);
    pattern.push('%');

    for char in search.chars() {
        if matches!(char, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(char);
    }

    pattern.push('%');
    pattern
}

async fn contact(conn: impl SqliteExecutor<'_>, id: u32) -> Result<Option<ContactRow>> {
    sqlx::query!("SELECT * FROM contacts WHERE id = ?", id)
        .fetch_optional(conn)
        .await?
        .map(|row| contact_row!(row))
        .transpose()
}

async fn contact_by_target(
    conn: impl SqliteExecutor<'_>,
    target: &ContactTarget,
) -> Result<Option<ContactRow>> {
    let (puzzle_hash, handle) = target.columns();

    sqlx::query!(
        "SELECT * FROM contacts WHERE puzzle_hash = ? OR handle = ?",
        puzzle_hash,
        handle
    )
    .fetch_optional(conn)
    .await?
    .map(|row| contact_row!(row))
    .transpose()
}

async fn contact_by_name(conn: impl SqliteExecutor<'_>, name: &str) -> Result<Option<ContactRow>> {
    let key = contact_name_key(name);

    sqlx::query!("SELECT * FROM contacts WHERE name_key = ?", key)
        .fetch_optional(conn)
        .await?
        .map(|row| contact_row!(row))
        .transpose()
}

async fn contacts_page(
    conn: &sqlx::SqlitePool,
    query: ContactQuery<'_>,
    limit: u32,
    offset: u32,
) -> Result<(Vec<ContactRow>, u32)> {
    let search = query.search.map(like_pattern);
    let puzzle_hash = query.puzzle_hash.as_ref().map(AsRef::<[u8]>::as_ref);
    let registry_launcher_id = query
        .registry_launcher_id
        .as_ref()
        .map(AsRef::<[u8]>::as_ref);
    let kind = query.kind.map(|kind| kind as u8);
    let unfiltered = search.is_none() && puzzle_hash.is_none();

    // The total is counted separately, so that it's still correct for a page past the end.
    let total = sqlx::query_scalar!(
        r"
        SELECT COUNT(*) FROM contacts
        WHERE (? IS NULL OR kind = ?)
            AND (
                ?
                OR name LIKE ? ESCAPE '\'
                OR note LIKE ? ESCAPE '\'
                OR handle LIKE ? ESCAPE '\'
                OR puzzle_hash = ?
                OR handle IN (
                    SELECT handle FROM xchandles
                    WHERE registry_launcher_id = ? AND resolved_puzzle_hash = ?
                )
            )
        ",
        kind,
        kind,
        unfiltered,
        search,
        search,
        search,
        puzzle_hash,
        registry_launcher_id,
        puzzle_hash
    )
    .fetch_one(conn)
    .await?;

    let rows = sqlx::query!(
        r"
        SELECT * FROM contacts
        WHERE (? IS NULL OR kind = ?)
            AND (
                ?
                OR name LIKE ? ESCAPE '\'
                OR note LIKE ? ESCAPE '\'
                OR handle LIKE ? ESCAPE '\'
                OR puzzle_hash = ?
                OR handle IN (
                    SELECT handle FROM xchandles
                    WHERE registry_launcher_id = ? AND resolved_puzzle_hash = ?
                )
            )
        ORDER BY name COLLATE NOCASE ASC, id ASC
        LIMIT ? OFFSET ?
        ",
        kind,
        kind,
        unfiltered,
        search,
        search,
        search,
        puzzle_hash,
        registry_launcher_id,
        puzzle_hash,
        limit,
        offset
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| contact_row!(row))
    .collect::<Result<_>>()?;

    Ok((rows, total.try_into()?))
}

async fn all_contacts(conn: impl SqliteExecutor<'_>) -> Result<Vec<ContactRow>> {
    sqlx::query!("SELECT * FROM contacts ORDER BY name COLLATE NOCASE ASC, id ASC")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| contact_row!(row))
        .collect()
}

async fn contacts_by_puzzle_hash(
    conn: impl SqliteExecutor<'_>,
    registry_launcher_id: Option<Bytes32>,
    puzzle_hash: Bytes32,
    now: u64,
) -> Result<Vec<ContactRow>> {
    let registry_launcher_id = registry_launcher_id.as_ref().map(AsRef::<[u8]>::as_ref);
    let puzzle_hash = puzzle_hash.as_ref();
    let now: i64 = now.try_into()?;

    sqlx::query!(
        "
        SELECT * FROM contacts
        WHERE puzzle_hash = ? OR handle IN (
            SELECT handle FROM xchandles
            WHERE registry_launcher_id = ?
                AND resolved_puzzle_hash = ?
                AND status = 0
                AND expiration > MAX(?, verified_timestamp)
        )
        ORDER BY name COLLATE NOCASE ASC, id ASC
        ",
        puzzle_hash,
        registry_launcher_id,
        puzzle_hash,
        now
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| contact_row!(row))
    .collect()
}

async fn insert_contact(
    conn: impl SqliteExecutor<'_>,
    name: &str,
    note: Option<&str>,
    target: &ContactTarget,
    now: u64,
) -> Result<u32> {
    let kind = target.kind() as u8;
    let (puzzle_hash, handle) = target.columns();
    let key = contact_name_key(name);
    let now: i64 = now.try_into()?;

    let id = sqlx::query_scalar!(
        "
        INSERT INTO contacts (
            name, name_key, note, kind, puzzle_hash, handle, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        ",
        name,
        key,
        note,
        kind,
        puzzle_hash,
        handle,
        now,
        now
    )
    .fetch_one(conn)
    .await?;

    id.convert()
}

async fn update_contact(
    conn: impl SqliteExecutor<'_>,
    id: u32,
    name: &str,
    note: Option<&str>,
    now: u64,
) -> Result<bool> {
    let key = contact_name_key(name);
    let now: i64 = now.try_into()?;

    let result = sqlx::query!(
        "UPDATE contacts SET name = ?, name_key = ?, note = ?, updated_at = ? WHERE id = ?",
        name,
        key,
        note,
        now,
        id
    )
    .execute(conn)
    .await?;

    Ok(result.rows_affected() > 0)
}

async fn delete_contact(conn: impl SqliteExecutor<'_>, id: u32) -> Result<bool> {
    let result = sqlx::query!("DELETE FROM contacts WHERE id = ?", id)
        .execute(conn)
        .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, migrate};

    use super::*;

    async fn database() -> anyhow::Result<Database> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        migrate!("../../migrations").run(&pool).await?;
        Ok(Database::new(pool))
    }

    #[tokio::test]
    async fn migration_enforces_identity_and_duplicates() -> anyhow::Result<()> {
        let db = database().await?;
        let mut tx = db.tx().await?;

        let alice = ContactTarget::Address(Bytes32::new([1; 32]));
        let bob = ContactTarget::Handle("bob".to_string());

        let id = tx.insert_contact("Alice", None, &alice, 1).await?;
        tx.insert_contact("Bob", Some("note"), &bob, 1).await?;

        // The schema backs up the endpoint's checks: one contact per recipient, and no two
        // names which look the same.
        assert!(tx.insert_contact("Other", None, &alice, 1).await.is_err());
        assert!(tx.insert_contact("Other", None, &bob, 1).await.is_err());
        assert!(
            tx.insert_contact("ALICE", None, &ContactTarget::Handle("x".into()), 1)
                .await
                .is_err()
        );

        assert!(
            tx.insert_contact("Al\u{200D}ice", None, &ContactTarget::Handle("y".into()), 1)
                .await
                .is_err()
        );

        assert_eq!(tx.contact_by_name("alice").await?.unwrap().id, id);
        assert_eq!(tx.contact_by_name(" ALICE\u{FE0F}").await?.unwrap().id, id);
        assert_eq!(tx.contact_by_target(&alice).await?.unwrap().name, "Alice");
        tx.commit().await?;

        // A contact names exactly one kind of recipient.
        let invalid = sqlx::query(
            "INSERT INTO contacts (name, name_key, kind, puzzle_hash, handle, created_at, updated_at)
            VALUES ('Mixed', 'mixed', 0, x'01', 'mixed', 1, 1)",
        )
        .execute(&db.pool)
        .await;
        assert!(invalid.is_err());

        assert!(db.delete_contact(id).await?);
        assert!(!db.delete_contact(id).await?);
        assert_eq!(db.all_contacts().await?.len(), 1);

        Ok(())
    }

    #[test]
    fn name_keys_ignore_what_cant_be_seen() {
        assert_eq!(contact_name_key("Alice"), "alice");
        assert_eq!(contact_name_key("ÉLODIE"), "élodie");
        assert_eq!(contact_name_key("Alice  \u{00A0}Smith"), "alice smith");
        assert_eq!(contact_name_key("Bob 🐱\u{200D}"), "bob 🐱");
        assert_eq!(contact_name_key("Bob 🐱\u{FE0F}"), "bob 🐱");
        assert_eq!(contact_name_key("Alice (work)\u{FE0F}"), "alice (work)");
        assert_eq!(contact_name_key("\u{200D}"), "");
        // Accents typed as combining marks, in either order, look the same as precomposed ones.
        assert_eq!(contact_name_key("Jose\u{0301}"), contact_name_key("José"));
        assert_eq!(
            contact_name_key("a\u{0323}\u{0302}"),
            contact_name_key("a\u{0302}\u{0323}")
        );
        assert_eq!(
            contact_name_key("a\u{0302}\u{034F}\u{0323}"),
            contact_name_key("a\u{0323}\u{0302}")
        );
        // Emoji keep their visible parts, so different emoji stay different.
        assert_ne!(
            contact_name_key("\u{1F3F3}\u{FE0F}\u{200D}\u{1F308}"),
            contact_name_key("\u{1F3F3}")
        );
    }

    #[test]
    fn like_patterns_escape_wildcards() {
        assert_eq!(like_pattern("al"), "%al%");
        assert_eq!(like_pattern("50%_\\"), "%50\\%\\_\\\\%");
    }
}
