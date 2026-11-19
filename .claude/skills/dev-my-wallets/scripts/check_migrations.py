"""Compare the migrations a Sage root's databases have applied with the ones this checkout builds.

Sage treats any migration error as an outdated database and offers to delete and resync it, so
this finds every case that would cause one before a dev build opens the databases:

- a database has applied a migration this checkout doesn't have, or a different version of one
  (sqlx compares SHA-384 checksums of the migration files)
- a database has a failed migration recorded
- this checkout has migrations the installed release doesn't, which the release would then fail on

It only reads the databases. Run it with `python3 -I`, from the repository root, while no Sage
instance is using the root.

Exit status: 0 if nothing will change, 1 if this build will apply new migrations (ask the user
first), 2 if launching would bring up the reset dialog (don't launch).
"""

import argparse
import hashlib
import sqlite3
import subprocess
import sys
from pathlib import Path

DEFAULT_ROOT = Path.home() / "Library/Application Support/com.rigidnetwork.sage"
WALLET_MIGRATIONS = "migrations"
APPS_MIGRATIONS = "crates/sage-apps/src/db/migrations"


def checkout_migrations(directory):
    """Each migration version in the working tree, with the checksum sqlx records for it."""
    migrations = {}
    for path in sorted(Path(directory).glob("*.sql")):
        version = int(path.name.split("_", 1)[0])
        migrations[version] = hashlib.sha384(path.read_bytes()).hexdigest()
    return migrations


def release_migrations(tag, directory):
    """Each migration version in a release tag, with its checksum."""
    names = subprocess.run(
        ["git", "ls-tree", "--name-only", f"{tag}:{directory}"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.split()
    migrations = {}
    for name in names:
        if not name.endswith(".sql"):
            continue
        contents = subprocess.run(
            ["git", "show", f"{tag}:{directory}/{name}"], check=True, capture_output=True
        ).stdout
        migrations[int(name.split("_", 1)[0])] = hashlib.sha384(contents).hexdigest()
    return migrations


def applied_migrations(path):
    """The migrations a database has recorded, as {version: (success, checksum)}."""
    uri = path.as_uri() + "?mode=ro"
    try:
        connection = sqlite3.connect(uri, uri=True)
        rows = connection.execute(
            "select version, success, lower(hex(checksum)) from _sqlx_migrations"
        ).fetchall()
    except sqlite3.OperationalError:
        # A database in WAL mode can't be opened read-only without its -shm file. Immutable mode
        # opens it, but ignores the WAL, so it's only used when there's no WAL to miss.
        if Path(f"{path}-wal").exists():
            raise
        connection = sqlite3.connect(uri + "&immutable=1", uri=True)
        rows = connection.execute(
            "select version, success, lower(hex(checksum)) from _sqlx_migrations"
        ).fetchall()
    return {version: (bool(success), checksum) for version, success, checksum in rows}


def check_database(path, expected):
    """Problems that would make Sage reject the database, and migrations it would apply."""
    try:
        applied = applied_migrations(path)
    except sqlite3.OperationalError as error:
        return [f"unreadable ({error})"], []

    problems = []
    for version, (success, checksum) in sorted(applied.items()):
        if not success:
            problems.append(f"migration {version} is recorded as failed")
        elif version not in expected:
            problems.append(f"migration {version} is applied, but this checkout doesn't have it")
        elif checksum != expected[version]:
            problems.append(f"migration {version} differs from this checkout's")
    pending = sorted(set(expected) - set(applied))
    return problems, pending


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--release", help="The installed release's tag, such as v0.13.0")
    args = parser.parse_args()

    sets = [
        (WALLET_MIGRATIONS, sorted((args.root / "wallets").glob("*/*.sqlite"))
         + sorted((args.root / "shared").glob("*.sqlite"))),
        (APPS_MIGRATIONS, [args.root / "sage-apps.sqlite3"]),
    ]

    blocked = False
    will_migrate = False

    for directory, databases in sets:
        expected = checkout_migrations(directory)

        for path in databases:
            name = path.relative_to(args.root)
            if not path.exists():
                # Created at launch, with every migration applied from scratch.
                print(f"{name}: missing, will be created")
                continue
            problems, pending = check_database(path, expected)
            if problems:
                blocked = True
                print(f"{name}: WOULD SHOW THE RESET DIALOG: {'; '.join(problems)}")
            elif pending:
                will_migrate = True
                print(f"{name}: will apply {', '.join(map(str, pending))}")
            else:
                print(f"{name}: up to date")

        if args.release:
            released = release_migrations(args.release, directory)
            ahead = [
                version
                for version, checksum in sorted(expected.items())
                if released.get(version) != checksum
            ]
            if ahead:
                print(
                    f"{directory}: {args.release} doesn't have {', '.join(map(str, ahead))} "
                    "(added or changed since)"
                )

    sys.exit(2 if blocked else 1 if will_migrate else 0)


if __name__ == "__main__":
    main()
