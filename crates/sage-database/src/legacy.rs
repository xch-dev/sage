//! Moving the address book and handle mappings from wallet databases to the shared database.
//!
//! They used to be stored in each wallet's database. Now every wallet on a network shares
//! them, so when a wallet is opened, anything still in its own database is merged into the
//! shared database and then removed from the wallet's. Merging is idempotent, so if it's
//! interrupted after the shared database is written, it's safe to run again.

use crate::{
    Database, MAX_CONTACT_NAME_LENGTH, Result, SharedDatabase, SharedDatabaseTx, XchandleRow,
};

/// The most names tried for a contact whose name is already taken, before it's skipped.
const MAX_RENAME_ATTEMPTS: usize = 100;

/// What was moved from a wallet's database to the shared database.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LegacyImport {
    /// Handle mappings added to the shared database, or merged into one already there.
    pub xchandles: usize,
    /// Contacts added to the shared database.
    pub contacts: usize,
    /// Contacts added under a different name, as `(old, new)`, because another contact
    /// already had a name that looks the same.
    pub renamed: Vec<(String, String)>,
    /// Contacts skipped because the shared database already has a contact for the same
    /// recipient.
    pub duplicates: usize,
}

/// Merges two mappings of the same handle, one from the shared database and one from a
/// wallet's database.
///
/// The more recently verified mapping is kept. The other one is treated as an earlier
/// verification of it, the same way a new verification is applied to a stored mapping: if
/// the handle resolved somewhere else in it, or it had a pending change, that becomes a
/// pending change, so that a change the user hasn't seen in either wallet is never lost.
/// The address last sent to is kept from either.
pub fn merge_xchandle(a: &XchandleRow, b: &XchandleRow) -> XchandleRow {
    let (newer, older) =
        if (b.last_verified_at, b.verified_height) > (a.last_verified_at, a.verified_height) {
            (b, a)
        } else {
            (a, b)
        };

    let mut merged = newer.clone();

    if merged.previous_resolved_puzzle_hash.is_none() {
        let (launcher_id, puzzle_hash) = match older.previous_resolved_puzzle_hash {
            Some(pending) => (older.previous_resolved_launcher_id, pending),
            None => (Some(older.resolved_launcher_id), older.resolved_puzzle_hash),
        };

        if puzzle_hash != merged.resolved_puzzle_hash {
            merged.previous_resolved_launcher_id = launcher_id;
            merged.previous_resolved_puzzle_hash = Some(puzzle_hash);
        }
    }

    merged.last_sent_puzzle_hash = newer.last_sent_puzzle_hash.or(older.last_sent_puzzle_hash);

    merged
}

impl SharedDatabase {
    /// Moves the address book and handle mappings in a wallet's database to this one, and
    /// removes them from the wallet's.
    ///
    /// Handle mappings are merged with [`merge_xchandle`]. Contacts for a recipient which
    /// already has one are skipped, since the existing contact is kept. A contact whose name
    /// looks the same as an existing contact's is added with a number after its name.
    pub async fn import_legacy(&self, wallet: &Database) -> Result<LegacyImport> {
        let xchandles = wallet.legacy_xchandles().await?;
        let contacts = wallet.legacy_contacts().await?;

        let mut import = LegacyImport::default();

        if xchandles.is_empty() && contacts.is_empty() {
            return Ok(import);
        }

        let mut tx = self.tx().await?;

        for legacy in xchandles {
            let row = match tx
                .xchandle(legacy.registry_launcher_id, &legacy.handle)
                .await?
            {
                Some(existing) => merge_xchandle(&existing, &legacy),
                None => legacy,
            };

            tx.upsert_xchandle(&row).await?;
            import.xchandles += 1;
        }

        for contact in contacts {
            if tx.contact_by_target(&contact.target).await?.is_some() {
                import.duplicates += 1;
                continue;
            }

            let Some(name) = available_name(&mut tx, &contact.name).await? else {
                import.duplicates += 1;
                continue;
            };

            tx.insert_contact(
                &name,
                contact.note.as_deref(),
                &contact.target,
                contact.created_at,
            )
            .await?;

            if name != contact.name {
                import.renamed.push((contact.name, name));
            }

            import.contacts += 1;
        }

        tx.commit().await?;

        // If this fails, everything is merged again next time, which changes nothing.
        wallet.clear_legacy_address_book().await?;

        Ok(import)
    }
}

/// A name for a contact which doesn't look the same as any existing contact's: the name
/// itself, or the name followed by " (2)", " (3)" and so on, shortened to fit if needed.
async fn available_name(tx: &mut SharedDatabaseTx<'_>, name: &str) -> Result<Option<String>> {
    if tx.contact_by_name(name).await?.is_none() {
        return Ok(Some(name.to_string()));
    }

    for number in 2..=MAX_RENAME_ATTEMPTS + 1 {
        let suffix = format!(" ({number})");
        let base: String = name
            .chars()
            .take(MAX_CONTACT_NAME_LENGTH - suffix.chars().count())
            .collect();
        let candidate = format!("{}{suffix}", base.trim_end());

        if tx.contact_by_name(&candidate).await?.is_none() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::prelude::*;
    use sqlx::{SqlitePool, migrate};

    use super::*;
    use crate::{ContactTarget, XchandleStatus};

    const REGISTRY: Bytes32 = Bytes32::new([9; 32]);
    const A: Bytes32 = Bytes32::new([1; 32]);
    const B: Bytes32 = Bytes32::new([2; 32]);
    const C: Bytes32 = Bytes32::new([3; 32]);

    async fn pool() -> anyhow::Result<SqlitePool> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        migrate!("../../migrations").run(&pool).await?;
        Ok(pool)
    }

    fn row(handle: &str, resolved: Bytes32, last_verified_at: u64) -> XchandleRow {
        XchandleRow {
            registry_launcher_id: REGISTRY,
            handle: handle.to_string(),
            handle_hash: Bytes32::new([4; 32]),
            owner_launcher_id: Bytes32::new([5; 32]),
            resolved_launcher_id: Bytes32::new([6; 32]),
            resolved_puzzle_hash: resolved,
            expiration: 2_000_000_000,
            slot_coin_id: Bytes32::new([7; 32]),
            slot_counter: 1,
            slot_left_handle_hash: Bytes32::default(),
            slot_right_handle_hash: Bytes32::default(),
            slot_parent_id: Bytes32::default(),
            slot_parent_parent_id: Bytes32::default(),
            slot_parent_inner_puzzle_hash: Bytes32::default(),
            slot_confirmation_height: 10,
            resolved_coin_id: Bytes32::default(),
            resolved_confirmation_height: 10,
            indexed_height: 10,
            verified_height: 20,
            verified_timestamp: 1_000,
            last_verified_at,
            status: XchandleStatus::Verified,
            status_reason: None,
            previous_resolved_launcher_id: None,
            previous_resolved_puzzle_hash: None,
            last_sent_puzzle_hash: None,
        }
    }

    #[test]
    fn merging_keeps_the_newer_mapping() {
        let older = row("alice", A, 100);
        let mut newer = row("alice", A, 200);
        newer.status = XchandleStatus::Stale;

        assert_eq!(merge_xchandle(&older, &newer), newer);
        assert_eq!(merge_xchandle(&newer, &older), newer);
    }

    #[test]
    fn merging_flags_a_change_seen_in_only_one_wallet() {
        // One wallet last knew the handle at A, and the other has since verified it at B.
        let older = row("alice", A, 100);
        let newer = row("alice", B, 200);

        let merged = merge_xchandle(&older, &newer);
        assert_eq!(merged.resolved_puzzle_hash, B);
        assert_eq!(merged.previous_resolved_puzzle_hash, Some(A));
        assert_eq!(
            merged.previous_resolved_launcher_id,
            Some(older.resolved_launcher_id)
        );
    }

    #[test]
    fn merging_keeps_pending_changes() {
        // A change from A to B wasn't acknowledged in the older wallet. The newer wallet sees
        // the handle at C, so the user last knew it at A.
        let mut older = row("alice", B, 100);
        older.previous_resolved_puzzle_hash = Some(A);
        let newer = row("alice", C, 200);
        assert_eq!(
            merge_xchandle(&older, &newer).previous_resolved_puzzle_hash,
            Some(A)
        );

        // If it went back to A, there's nothing left to acknowledge.
        let newer = row("alice", A, 200);
        assert_eq!(
            merge_xchandle(&older, &newer).previous_resolved_puzzle_hash,
            None
        );

        // The newer wallet's own pending change is kept as it is.
        let mut newer = row("alice", C, 200);
        newer.previous_resolved_puzzle_hash = Some(B);
        assert_eq!(
            merge_xchandle(&older, &newer).previous_resolved_puzzle_hash,
            Some(B)
        );
    }

    #[test]
    fn merging_keeps_the_address_last_sent_to() {
        let mut older = row("alice", A, 100);
        older.last_sent_puzzle_hash = Some(A);
        let newer = row("alice", A, 200);
        assert_eq!(
            merge_xchandle(&older, &newer).last_sent_puzzle_hash,
            Some(A)
        );

        let mut newer = row("alice", A, 200);
        newer.last_sent_puzzle_hash = Some(B);
        assert_eq!(
            merge_xchandle(&older, &newer).last_sent_puzzle_hash,
            Some(B)
        );
    }

    #[test]
    fn merging_is_idempotent() {
        let mut older = row("alice", B, 100);
        older.previous_resolved_puzzle_hash = Some(A);
        older.last_sent_puzzle_hash = Some(A);
        let newer = row("alice", C, 200);

        let merged = merge_xchandle(&older, &newer);
        assert_eq!(merge_xchandle(&merged, &older), merged);
        assert_eq!(merge_xchandle(&merged, &merged), merged);
    }

    #[tokio::test]
    async fn importing_moves_everything_and_can_run_again() -> anyhow::Result<()> {
        let wallet = Database::new(pool().await?);
        let other_wallet = Database::new(pool().await?);
        let shared = SharedDatabase::new(pool().await?);

        // Two wallets saved contacts and resolved handles before they were shared.
        let alice = ContactTarget::Address(A);
        let bob = ContactTarget::Handle("bob".to_string());
        let carol = ContactTarget::Address(C);

        save_legacy(
            &wallet,
            &[("Alice", &alice), ("Bob", &bob)],
            &[row("bob", A, 100)],
        )
        .await?;

        // The same recipient under another name, and another recipient under a name that's
        // already taken.
        save_legacy(
            &other_wallet,
            &[("Alice's address", &alice), ("ALICE", &carol)],
            &[row("bob", B, 200), row("dave", C, 150)],
        )
        .await?;

        let import = shared.import_legacy(&wallet).await?;
        assert_eq!(
            import,
            LegacyImport {
                xchandles: 1,
                contacts: 2,
                renamed: vec![],
                duplicates: 0,
            }
        );

        let import = shared.import_legacy(&other_wallet).await?;
        assert_eq!(
            import,
            LegacyImport {
                xchandles: 2,
                contacts: 1,
                renamed: vec![("ALICE".to_string(), "ALICE (2)".to_string())],
                duplicates: 1,
            }
        );

        let names: Vec<_> = shared
            .all_contacts()
            .await?
            .into_iter()
            .map(|contact| (contact.name, contact.target))
            .collect();
        assert_eq!(
            names,
            [
                ("Alice".to_string(), alice.clone()),
                ("ALICE (2)".to_string(), carol.clone()),
                ("Bob".to_string(), bob.clone()),
            ]
        );

        // The other wallet verified @bob more recently, at B, so the first wallet's A is
        // kept as a pending change.
        let merged = shared.xchandle(REGISTRY, "bob").await?.unwrap();
        assert_eq!(merged.resolved_puzzle_hash, B);
        assert_eq!(merged.previous_resolved_puzzle_hash, Some(A));
        assert!(shared.xchandle(REGISTRY, "dave").await?.is_some());

        // Nothing is left in the wallets, so opening them again changes nothing.
        for db in [&wallet, &other_wallet] {
            assert!(db.legacy_contacts().await?.is_empty());
            assert!(db.legacy_xchandles().await?.is_empty());
            assert_eq!(shared.import_legacy(db).await?, LegacyImport::default());
        }

        Ok(())
    }

    #[tokio::test]
    async fn an_interrupted_import_can_run_again() -> anyhow::Result<()> {
        let wallet = Database::new(pool().await?);
        let shared = SharedDatabase::new(pool().await?);

        let alice = ContactTarget::Address(A);
        let mut mapping = row("bob", B, 100);
        mapping.previous_resolved_puzzle_hash = Some(A);

        save_legacy(
            &wallet,
            &[("Alice", &alice)],
            std::slice::from_ref(&mapping),
        )
        .await?;
        shared.import_legacy(&wallet).await?;

        // As if Sage was closed after writing the shared database, but before clearing the
        // wallet's.
        save_legacy(
            &wallet,
            &[("Alice", &alice)],
            std::slice::from_ref(&mapping),
        )
        .await?;

        let import = shared.import_legacy(&wallet).await?;
        assert_eq!(import.contacts, 0);
        assert_eq!(import.duplicates, 1);
        assert_eq!(shared.all_contacts().await?.len(), 1);
        assert_eq!(shared.xchandle(REGISTRY, "bob").await?.unwrap(), mapping);

        Ok(())
    }

    #[tokio::test]
    async fn renamed_contacts_fit_the_name_limit() -> anyhow::Result<()> {
        let shared = SharedDatabase::new(pool().await?);
        let long = "a".repeat(MAX_CONTACT_NAME_LENGTH);

        let mut tx = shared.tx().await?;
        tx.insert_contact(&long, None, &ContactTarget::Address(A), 1)
            .await?;
        let name = available_name(&mut tx, &long).await?.unwrap();
        tx.commit().await?;

        assert_eq!(name.chars().count(), MAX_CONTACT_NAME_LENGTH);
        assert!(name.ends_with("a (2)"));

        Ok(())
    }

    /// The wallet's database, through the queries Sage used to run against it before the
    /// address book was shared. The tables are the same in both.
    fn as_legacy(wallet: &Database) -> SharedDatabase {
        SharedDatabase::new(wallet.pool.clone())
    }

    /// Saves contacts and handle mappings in a wallet's database, as Sage did before they
    /// were shared.
    async fn save_legacy(
        wallet: &Database,
        contacts: &[(&str, &ContactTarget)],
        mappings: &[XchandleRow],
    ) -> anyhow::Result<()> {
        let legacy = as_legacy(wallet);
        let mut tx = legacy.tx().await?;

        for (name, target) in contacts {
            tx.insert_contact(name, None, target, 1).await?;
        }

        for mapping in mappings {
            tx.upsert_xchandle(mapping).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
