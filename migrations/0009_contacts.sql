/*
 * The address book.
 *
 * Every wallet has its own database for each network, so contacts are already scoped to a
 * single wallet and network, and are never shared between them.
 *
 * A contact is a name for exactly one recipient, which is either a raw address (kind 0,
 * stored as its puzzle hash) or an XCHandles handle (kind 1, stored without the leading `@`).
 * The recipient is the contact's stable identity and can't be changed once the contact has
 * been created. For a handle, only the handle is stored here: the address it resolves to,
 * whether that is verified, the address it resolved to before a change, and the address last
 * sent to all come from the xchandles table, so they're never copied or allowed to drift.
 *
 * Duplicates: a recipient can belong to at most one contact, and no two names can look the
 * same, so that two entries in a recipient picker can't be confused. Names are compared by
 * name_key, which is the name lowercased, with whitespace collapsed and characters that
 * don't change how it looks, like zero-width joiners and variation selectors, removed.
 */
CREATE TABLE contacts (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL COLLATE NOCASE,
  name_key TEXT NOT NULL UNIQUE,
  note TEXT,
  kind INTEGER NOT NULL,
  puzzle_hash BLOB UNIQUE,
  handle TEXT UNIQUE,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  CHECK (
    (kind = 0 AND puzzle_hash IS NOT NULL AND handle IS NULL)
    OR (kind = 1 AND handle IS NOT NULL AND puzzle_hash IS NULL)
  )
);
