use anyhow::Result;
use axum::http::StatusCode;
use chia_wallet_sdk::utils::Address;
use sage_api::{
    AddContact, ContactEntry, ContactKind, ContactRecord, ContactSkipReason, DeleteContact,
    DeleteWallet, ExportContacts, GetContact, GetContacts, GetHandles, HandleFilterMode,
    HandleStatus, HandleVerification, ImportContacts, Login, LookupAddressContacts,
    SetNetworkOverride, UpdateContact,
};
use sage_database::{XchandleRow, XchandleStatus};

use super::{
    TestApp,
    recipients::{
        address, confirm, error_body, funded_app, now, puzzle_hash, send, store_handle,
        stored_handle,
    },
};

fn add(name: &str, recipient: &str) -> AddContact {
    AddContact {
        name: name.to_string(),
        recipient: recipient.to_string(),
        note: None,
    }
}

fn search(search: Option<&str>, kind: Option<ContactKind>) -> GetContacts {
    GetContacts {
        search: search.map(str::to_string),
        kind,
        offset: 0,
        limit: 50,
    }
}

fn names(contacts: &[ContactRecord]) -> Vec<&str> {
    contacts
        .iter()
        .map(|contact| contact.name.as_str())
        .collect()
}

async fn contacts(app: &TestApp) -> Result<Vec<ContactRecord>> {
    Ok(app.get_contacts(search(None, None)).await?.contacts)
}

async fn contact(app: &TestApp, name: &str) -> Result<ContactRecord> {
    Ok(contacts(app)
        .await?
        .into_iter()
        .find(|contact| contact.name == name)
        .expect("missing contact"))
}

fn assert_status(error: &anyhow::Error, status: StatusCode) {
    let message = error.to_string();
    assert!(
        message.contains(&format!("status {}", status.as_u16())),
        "{message}"
    );
}

#[tokio::test]
async fn test_contact_crud_and_search() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    let alice = app
        .add_contact(AddContact {
            note: Some("  From the meetup  ".to_string()),
            ..add("  Alice ", &address(1))
        })
        .await?
        .contact;
    assert_eq!(alice.name, "Alice");
    assert_eq!(alice.note.as_deref(), Some("From the meetup"));
    assert_eq!(alice.kind, ContactKind::Address);
    assert_eq!(alice.recipient, address(1));
    assert_eq!(alice.address.as_deref(), Some(address(1).as_str()));
    assert!(alice.resolution.is_none());

    // A handle contact is distinguishable, and isn't looked up when it's added.
    let bob = app.add_contact(add("Bob", " @bob ")).await?.contact;
    assert_eq!(bob.kind, ContactKind::Handle);
    assert_eq!(bob.recipient, "@bob");
    assert_eq!(bob.handle.as_deref(), Some("bob"));
    assert_eq!(bob.address, None);
    assert!(bob.resolution.is_none());

    app.add_contact(add("carol", &address(3))).await?;

    let all = app.get_contacts(search(None, None)).await?;
    assert_eq!(names(&all.contacts), ["Alice", "Bob", "carol"]);
    assert_eq!(all.total, 3);

    let by_name = app.get_contacts(search(Some("ALI"), None)).await?;
    assert_eq!(names(&by_name.contacts), ["Alice"]);

    let by_note = app.get_contacts(search(Some("meetup"), None)).await?;
    assert_eq!(names(&by_note.contacts), ["Alice"]);

    let by_handle = app.get_contacts(search(Some("@bo"), None)).await?;
    assert_eq!(names(&by_handle.contacts), ["Bob"]);

    let by_address = app.get_contacts(search(Some(&address(3)), None)).await?;
    assert_eq!(names(&by_address.contacts), ["carol"]);

    let by_hex = app
        .get_contacts(search(Some(&hex::encode(puzzle_hash(1))), None))
        .await?;
    assert_eq!(names(&by_hex.contacts), ["Alice"]);

    // LIKE wildcards are matched literally.
    let wildcard = app.get_contacts(search(Some("%"), None)).await?;
    assert!(wildcard.contacts.is_empty());

    let handles = app
        .get_contacts(search(None, Some(ContactKind::Handle)))
        .await?;
    assert_eq!(names(&handles.contacts), ["Bob"]);

    let page = app
        .get_contacts(GetContacts {
            offset: 1,
            limit: 1,
            ..search(None, None)
        })
        .await?;
    assert_eq!(names(&page.contacts), ["Bob"]);
    assert_eq!(page.total, 3);

    // Renaming keeps the contact's identity and recipient.
    let renamed = app
        .update_contact(UpdateContact {
            id: alice.id,
            name: "Alice Smith".to_string(),
            note: None,
        })
        .await?
        .contact;
    assert_eq!(renamed.id, alice.id);
    assert_eq!(renamed.recipient, address(1));
    assert_eq!(renamed.note, None);
    assert_eq!(
        app.get_contact(GetContact { id: alice.id })
            .await?
            .contact
            .name,
        "Alice Smith"
    );

    // Changing only the case of its own name is allowed.
    app.update_contact(UpdateContact {
        id: alice.id,
        name: "alice smith".to_string(),
        note: None,
    })
    .await?;

    app.delete_contact(DeleteContact { id: bob.id }).await?;
    assert_eq!(names(&contacts(&app).await?), ["alice smith", "carol"]);

    for error in [
        app.get_contact(GetContact { id: bob.id })
            .await
            .unwrap_err(),
        app.delete_contact(DeleteContact { id: bob.id })
            .await
            .unwrap_err(),
        app.update_contact(UpdateContact {
            id: bob.id,
            name: "Bob".to_string(),
            note: None,
        })
        .await
        .unwrap_err(),
    ] {
        assert_status(&error, StatusCode::NOT_FOUND);
    }

    Ok(())
}

#[tokio::test]
async fn test_contact_duplicates_and_validation() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    let alice = app.add_contact(add("Alice", &address(1))).await?.contact;
    let bob = app.add_contact(add("Bob", "@bob")).await?.contact;

    // A recipient can only belong to one contact, however it's written.
    let error = app
        .add_contact(add("Alice 2", &hex::encode(puzzle_hash(1))))
        .await
        .unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);

    let error = app
        .add_contact(add("Alice 2", &address(1)))
        .await
        .unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);
    assert!(
        error
            .to_string()
            .contains("already saved as the contact Alice")
    );

    let error = app.add_contact(add("Bobby", "@bob")).await.unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);

    // No two names can look the same, so two picker entries can't be confused. That covers
    // case, and characters which don't change how a name looks.
    for name in ["ALICE", "Al\u{200D}ice", "Alice\u{FE0F}", " alice "] {
        let error = app.add_contact(add(name, &address(2))).await.unwrap_err();
        assert_status(&error, StatusCode::BAD_REQUEST);
        assert!(error.to_string().contains("Alice"), "{name:?}: {error}");
    }

    // Accents typed as combining marks look the same as precomposed ones, and are found by
    // the same search.
    app.add_contact(add("Jos\u{00E9}", &address(3))).await?;
    let error = app
        .add_contact(add("Jose\u{0301}", &address(4)))
        .await
        .unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);
    let found = app.get_contacts(search(Some("Jose\u{0301}"), None)).await?;
    assert_eq!(found.total, 1);

    let error = app
        .update_contact(UpdateContact {
            id: bob.id,
            name: "alice".to_string(),
            note: None,
        })
        .await
        .unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);

    let mainnet = Address::new(puzzle_hash(5), "xch".to_string()).encode()?;

    for (name, recipient) in [
        ("", address(5).as_str()),
        ("Line\nbreak", address(5).as_str()),
        // Looks the same as an existing contact's name.
        ("Alice\u{200B}", address(5).as_str()),
        ("Mainnet", mainnet.as_str()),
        ("Bad handle", "@AB"),
        ("Not an address", "alice"),
    ] {
        let error = app.add_contact(add(name, recipient)).await.unwrap_err();
        assert_status(&error, StatusCode::BAD_REQUEST);
    }

    assert_eq!(
        names(&contacts(&app).await?),
        ["Alice", "Bob", "Jos\u{00E9}"]
    );
    assert_eq!(contact(&app, "Alice").await?.id, alice.id);

    Ok(())
}

#[tokio::test]
async fn test_handle_contact_history_comes_from_the_store() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    app.add_contact(add("Alice", "@alice")).await?;

    // Until the handle is resolved, there's nothing to show for it.
    assert!(contact(&app, "Alice").await?.resolution.is_none());

    store_handle(&app, "alice", puzzle_hash(1), |_| {}).await?;

    let alice = contact(&app, "Alice").await?;
    let resolution = alice.resolution.expect("missing resolution");
    assert_eq!(alice.address.as_deref(), Some(address(1).as_str()));
    assert_eq!(resolution.status, HandleStatus::Verified);
    assert_eq!(resolution.verification, HandleVerification::Verified);
    // Read from the store, so never presented as freshly verified.
    assert!(resolution.cached);

    // The mapping changed: the contact keeps the old address as history, from the store.
    store_handle(&app, "alice", puzzle_hash(2), |row| {
        row.previous_resolved_puzzle_hash = Some(puzzle_hash(1));
        row.last_sent_puzzle_hash = Some(puzzle_hash(1));
    })
    .await?;

    let resolution = contact(&app, "Alice").await?.resolution.unwrap();
    assert_eq!(resolution.address, address(2));
    assert_eq!(resolution.status, HandleStatus::Changed);
    assert_eq!(
        resolution.previous_address.as_deref(),
        Some(address(1).as_str())
    );
    assert_eq!(
        resolution.changed_from.as_deref(),
        Some(address(1).as_str())
    );
    assert_eq!(
        resolution.last_sent_address.as_deref(),
        Some(address(1).as_str())
    );

    // Deleting the contact keeps the mapping and its history, so adding it again can't hide
    // the change.
    let id = contact(&app, "Alice").await?.id;
    app.delete_contact(DeleteContact { id }).await?;
    assert!(
        stored_handle(&app, "alice")
            .await?
            .previous_resolved_puzzle_hash
            .is_some()
    );

    app.add_contact(add("Alice", "@alice")).await?;
    let resolution = contact(&app, "Alice").await?.resolution.unwrap();
    assert_eq!(resolution.status, HandleStatus::Changed);

    // Old, stale, unverified and expired mappings show their state.
    for (edit, verification) in [
        (
            Box::new(|row: &mut XchandleRow| {
                row.last_verified_at -= 2 * 24 * 60 * 60;
            }) as Box<dyn FnOnce(&mut XchandleRow)>,
            HandleVerification::Outdated,
        ),
        (
            Box::new(|row: &mut XchandleRow| {
                row.status = XchandleStatus::Stale;
            }),
            HandleVerification::Stale,
        ),
        (
            Box::new(|row: &mut XchandleRow| {
                row.status = XchandleStatus::Unverified;
                row.status_reason = Some("Handle not found".to_string());
            }),
            HandleVerification::Unverified,
        ),
        (
            Box::new(|row: &mut XchandleRow| {
                row.expiration = now() - 1;
            }),
            HandleVerification::Expired,
        ),
    ] {
        store_handle(&app, "alice", puzzle_hash(2), edit).await?;
        let resolution = contact(&app, "Alice").await?.resolution.unwrap();
        assert_eq!(resolution.verification, verification);
        assert!(resolution.cached);
    }

    Ok(())
}

#[tokio::test]
async fn test_changed_handle_contact_requires_acknowledgement() -> Result<()> {
    let mut app = funded_app().await?;

    store_handle(&app, "alice", puzzle_hash(2), |row| {
        row.previous_resolved_puzzle_hash = Some(puzzle_hash(1));
    })
    .await?;

    let alice = app.add_contact(add("Alice", "@alice")).await?.contact;

    // Sending to the contact goes through the recipient pipeline, which refuses the changed
    // handle until the new address is confirmed.
    let error = app
        .send_xch(send(&alice.recipient, vec![], true))
        .await
        .unwrap_err();
    let (status, body) = error_body(&error);
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["recipients"][0]["kind"], "changed");
    assert_eq!(body["recipients"][0]["previous_address"], address(1));

    app.send_xch(send(
        &alice.recipient,
        vec![confirm("alice", address(2))],
        true,
    ))
    .await?;
    app.wait_for_coins().await;

    let resolution = contact(&app, "Alice").await?.resolution.unwrap();
    assert_eq!(resolution.status, HandleStatus::Verified);
    assert_eq!(resolution.changed_from, None);
    assert_eq!(
        resolution.last_sent_address.as_deref(),
        Some(address(2).as_str())
    );

    Ok(())
}

#[tokio::test]
async fn test_lookup_address_contacts() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    app.add_contact(add("Alice", &address(1))).await?;
    app.add_contact(add("Alice's handle", "@alice")).await?;
    app.add_contact(add("Bob", "@bob")).await?;
    app.add_contact(add("Carol", "@carol")).await?;
    app.add_contact(add("Dave", "@dave")).await?;
    app.add_contact(add("Erin", "@erin")).await?;
    app.add_contact(add("Frank", "@frank")).await?;

    store_handle(&app, "alice", puzzle_hash(1), |_| {}).await?;
    store_handle(&app, "erin", puzzle_hash(1), |row| {
        row.status = XchandleStatus::Stale;
    })
    .await?;
    // A handle which changed hands labels its new address, marked as changed, and no
    // longer labels the address it used to resolve to.
    store_handle(&app, "frank", puzzle_hash(2), |row| {
        row.previous_resolved_puzzle_hash = Some(puzzle_hash(1));
    })
    .await?;
    // Mappings which can't be relied on never label an address.
    store_handle(&app, "bob", puzzle_hash(1), |row| {
        row.status = XchandleStatus::Unverified;
    })
    .await?;
    store_handle(&app, "carol", puzzle_hash(1), |row| row.expiration = 1).await?;
    // Outdated mappings do, and are marked as outdated.
    store_handle(&app, "dave", puzzle_hash(2), |row| {
        row.last_verified_at -= 2 * 24 * 60 * 60;
    })
    .await?;

    let results = app
        .lookup_address_contacts(LookupAddressContacts {
            addresses: vec![
                address(1),
                hex::encode(puzzle_hash(2)),
                format!("0x{}", hex::encode(puzzle_hash(9))),
                address(1),
            ],
        })
        .await?
        .results;

    assert_eq!(results.len(), 4);
    assert_eq!(results[0].address, address(1));
    assert_eq!(names(&results[0].contacts), ["Alice", "Alice's handle"]);
    assert_eq!(names(&results[1].contacts), ["Dave", "Frank"]);
    assert_eq!(
        results[1].contacts[0]
            .resolution
            .as_ref()
            .unwrap()
            .verification,
        HandleVerification::Outdated
    );
    assert_eq!(
        results[1].contacts[1]
            .resolution
            .as_ref()
            .unwrap()
            .changed_from
            .as_deref(),
        Some(address(1).as_str())
    );
    assert!(results[2].contacts.is_empty());
    assert_eq!(names(&results[3].contacts), ["Alice", "Alice's handle"]);

    // Handles aren't addresses, and other networks' addresses are rejected.
    let mainnet = Address::new(puzzle_hash(1), "xch".to_string()).encode()?;
    for address in ["@alice", mainnet.as_str()] {
        let error = app
            .lookup_address_contacts(LookupAddressContacts {
                addresses: vec![address.to_string()],
            })
            .await
            .unwrap_err();
        assert_status(&error, StatusCode::BAD_REQUEST);
    }

    let error = app
        .lookup_address_contacts(LookupAddressContacts {
            addresses: vec![address(1); 1001],
        })
        .await
        .unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn test_get_handles_contacts_filter() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    store_handle(&app, "alice", puzzle_hash(1), |_| {}).await?;
    store_handle(&app, "bob", puzzle_hash(2), |_| {}).await?;
    app.add_contact(add("Bob", "@bob")).await?;
    app.add_contact(add("Carol", "@carol")).await?;

    let request = |filter| GetHandles {
        filter,
        include_unverified: true,
        refresh: false,
        search: None,
        offset: 0,
        limit: 50,
    };

    // Every resolved handle is listed, whether or not it's a contact.
    let all = app.get_handles(request(HandleFilterMode::All)).await?;
    assert_eq!(all.total, 2);

    // Only resolved handles which are contacts. Carol was never resolved.
    let contacts = app.get_handles(request(HandleFilterMode::Contacts)).await?;
    assert_eq!(contacts.total, 1);
    assert_eq!(contacts.handles[0].handle, "bob");

    Ok(())
}

#[tokio::test]
async fn test_export_and_import_contacts() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    app.add_contact(AddContact {
        note: Some("note".to_string()),
        ..add("Alice", &address(1))
    })
    .await?;
    app.add_contact(add("Bob", "@bob")).await?;

    let export = app.export_contacts(ExportContacts {}).await?;
    assert_eq!(export.network_id, "testnet11");
    assert_eq!(
        export.contacts,
        [
            ContactEntry {
                name: "Alice".to_string(),
                recipient: address(1),
                note: Some("note".to_string()),
            },
            ContactEntry {
                name: "Bob".to_string(),
                recipient: "@bob".to_string(),
                note: None,
            },
        ]
    );

    let bob = contact(&app, "Bob").await?.id;
    app.delete_contact(DeleteContact { id: bob }).await?;

    let entry = |name: &str, recipient: &str| ContactEntry {
        name: name.to_string(),
        recipient: recipient.to_string(),
        note: None,
    };

    let mut entries = export.contacts.clone();
    entries.extend([
        // Renaming an existing contact through an import isn't possible.
        entry("Mallory", &address(1)),
        // Neither is reusing a name for another recipient.
        entry("alice", &address(7)),
        entry("Bad", "@x"),
        entry("Carol", &address(3)),
        // A duplicate within the file is skipped too.
        entry("Carol again", &address(3)),
    ]);

    let imported = app
        .import_contacts(ImportContacts {
            network_id: "testnet11".to_string(),
            contacts: entries,
        })
        .await?;

    assert_eq!(names(&imported.added), ["Bob", "Carol"]);
    assert_eq!(
        imported
            .skipped
            .iter()
            .map(|skipped| (skipped.entry.name.as_str(), skipped.reason))
            .collect::<Vec<_>>(),
        [
            ("Alice", ContactSkipReason::DuplicateRecipient),
            ("Mallory", ContactSkipReason::DuplicateRecipient),
            ("alice", ContactSkipReason::DuplicateName),
            ("Bad", ContactSkipReason::Invalid),
            ("Carol again", ContactSkipReason::DuplicateRecipient),
        ]
    );
    assert_eq!(names(&contacts(&app).await?), ["Alice", "Bob", "Carol"]);
    assert_eq!(contact(&app, "Alice").await?.note.as_deref(), Some("note"));

    // Contacts from another network would point somewhere else here.
    let error = app
        .import_contacts(ImportContacts {
            network_id: "mainnet".to_string(),
            contacts: vec![entry("Dave", "@dave")],
        })
        .await
        .unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn test_contacts_and_handles_are_shared_per_network() -> Result<()> {
    let mut app = TestApp::new().await?;

    let first = app.setup_bls(0).await?;
    app.add_contact(add("Alice", &address(1))).await?;
    app.add_contact(add("Bob", "@bob")).await?;
    store_handle(&app, "bob", puzzle_hash(2), |row| {
        row.previous_resolved_puzzle_hash = Some(puzzle_hash(3));
    })
    .await?;

    // Every wallet on the network has the same address book, and the same history for each
    // handle, including a change which hasn't been acknowledged yet.
    let second = app.setup_bls(0).await?;
    assert_eq!(names(&contacts(&app).await?), ["Alice", "Bob"]);
    let bob = contact(&app, "Bob").await?.resolution.unwrap();
    assert_eq!(bob.address, address(2));
    assert_eq!(bob.changed_from.as_deref(), Some(address(3).as_str()));

    // So a contact can't be saved twice by switching wallets.
    let error = app
        .add_contact(add("Alice", &address(4)))
        .await
        .unwrap_err();
    assert_status(&error, StatusCode::BAD_REQUEST);

    // Changes in one wallet are seen in the others.
    app.add_contact(add("Carol", &address(5))).await?;
    let alice = contact(&app, "Alice").await?.id;
    app.delete_contact(DeleteContact { id: alice }).await?;

    app.login(Login { fingerprint: first }).await?;
    assert_eq!(names(&contacts(&app).await?), ["Bob", "Carol"]);

    // Each network has its own address book and handles.
    app.set_network_override(SetNetworkOverride {
        fingerprint: first,
        name: Some("mainnet".to_string()),
    })
    .await?;
    app.login(Login { fingerprint: first }).await?;
    assert!(contacts(&app).await?.is_empty());
    app.add_contact(add("Mainnet", &mainnet_address(1)?))
        .await?;

    app.set_network_override(SetNetworkOverride {
        fingerprint: first,
        name: None,
    })
    .await?;
    app.login(Login { fingerprint: first }).await?;
    assert_eq!(names(&contacts(&app).await?), ["Bob", "Carol"]);

    // Deleting a wallet leaves the address book for the other wallets.
    app.delete_wallet(DeleteWallet {
        fingerprint: first,
        delete_key: true,
    })
    .await?;
    app.login(Login {
        fingerprint: second,
    })
    .await?;
    assert_eq!(names(&contacts(&app).await?), ["Bob", "Carol"]);
    assert_eq!(
        stored_handle(&app, "bob").await?.resolved_puzzle_hash,
        puzzle_hash(2)
    );

    Ok(())
}

fn mainnet_address(byte: u8) -> Result<String> {
    Ok(Address::new(puzzle_hash(byte), "xch".to_string()).encode()?)
}
