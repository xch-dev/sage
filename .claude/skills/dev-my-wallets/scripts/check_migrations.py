"""Compare the migrations a Sage root's databases have applied with the ones this checkout builds.

Sage treats any migration error as an outdated database and offers to delete and resync it, so
this finds every case that would cause one before a dev build opens the databases:

- a database has applied a migration this checkout doesn't have, or a different version of one
  (sqlx compares SHA-384 checksums of the migration files)
- a database has a failed migration recorded
- this checkout has migrations the installed release doesn't, which the release would then fail on

It never writes to the databases or creates them. Opening one read-only can still create or
update its `-shm` file, as any SQLite reader does. Run it with `python3 -I`, from the repository
root, while no Sage instance is using the root.

Launching opens the active wallet's database, the shared databases and the apps database, so a
problem in one of those blocks the launch. Other wallets are only opened when the user switches to
them, so a problem in one of those is a warning.

Exit status: 0 if nothing will change, 1 if this build will apply migrations the release doesn't
have (ask the user first), 2 if launching would bring up the reset dialog (don't launch).
"""

import argparse
import hashlib
import re
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


def tag_exists(tag):
    result = subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", f"{tag}^{{commit}}"],
        capture_output=True,
    )
    return result.returncode == 0


def release_migrations(tag, directory):
    """Each migration version in a release tag, with its checksum. Empty if it has none there."""
    listing = subprocess.run(
        ["git", "ls-tree", "--name-only", f"{tag}:{directory}"],
        capture_output=True,
        text=True,
    )
    if listing.returncode != 0:
        # The directory doesn't exist in that release, such as the apps migrations before v0.13.0.
        return {}
    migrations = {}
    for name in listing.stdout.split():
        if not name.endswith(".sql"):
            continue
        contents = subprocess.run(
            ["git", "show", f"{tag}:{directory}/{name}"], check=True, capture_output=True
        ).stdout
        migrations[int(name.split("_", 1)[0])] = hashlib.sha384(contents).hexdigest()
    return migrations


def active_fingerprint(root):
    """The wallet Sage opens at launch: `fingerprint` in the `[global]` section of config.toml."""
    try:
        config = (root / "config.toml").read_text()
    except OSError:
        return None
    section = re.search(r"^\[global\]$(.*?)(?=^\[|\Z)", config, re.M | re.S)
    match = section and re.search(r"^fingerprint\s*=\s*(\d+)\s*$", section.group(1), re.M)
    return match.group(1) if match else None


def query(connection, sql):
    return connection.execute(sql).fetchall()


def applied_migrations(path):
    """
    The migrations a database has recorded, as {version: (success, checksum)}. An empty database
    has none. Raises `sqlite3.Error` if it can't be read.
    """
    uri = path.as_uri() + "?mode=ro"
    try:
        connection = sqlite3.connect(uri, uri=True)
        tables = query(connection, "select name from sqlite_master where type = 'table'")
    except sqlite3.OperationalError:
        # A database in WAL mode can't be opened read-only without its -shm file. Immutable mode
        # opens it, but ignores the WAL, so it's only used when there's no WAL to miss.
        if Path(f"{path}-wal").exists():
            raise
        connection = sqlite3.connect(uri + "&immutable=1", uri=True)
        tables = query(connection, "select name from sqlite_master where type = 'table'")

    if not tables:
        return {}
    if ("_sqlx_migrations",) not in tables:
        raise sqlite3.DatabaseError("it has tables, but no _sqlx_migrations table")

    rows = query(connection, "select version, success, lower(hex(checksum)) from _sqlx_migrations")
    return {version: (bool(success), checksum) for version, success, checksum in rows}


def problems_with(applied, migrations):
    """Why sqlx would refuse to run `migrations` on a database with `applied` recorded."""
    problems = []
    for version, (success, checksum) in sorted(applied.items()):
        if not success:
            problems.append(f"migration {version} is recorded as failed")
        elif version not in migrations:
            problems.append(f"migration {version} is applied, but the build doesn't have it")
        elif checksum != migrations[version]:
            problems.append(f"migration {version} differs from the build's")
    return problems


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--release", help="The installed release's tag, such as v0.13.0")
    args = parser.parse_args()

    release = args.release
    if release and not tag_exists(release):
        print(f"release comparison skipped: tag {release} not found (run git fetch --tags)")
        release = None

    sets = [
        (WALLET_MIGRATIONS, sorted((args.root / "wallets").glob("*/*.sqlite"))
         + sorted((args.root / "shared").glob("*.sqlite"))),
        (APPS_MIGRATIONS, [args.root / "sage-apps.sqlite3"]),
    ]

    active = active_fingerprint(args.root)
    print(f"active wallet: {active or 'none'}")

    def opened_at_launch(path):
        parts = path.relative_to(args.root).parts
        return parts[0] != "wallets" or parts[1] == active

    blocked = False
    will_migrate = False

    for directory, databases in sets:
        expected = checkout_migrations(directory)
        released = release_migrations(release, directory) if release else None
        # Migrations the release doesn't have in this form. Without a release to compare with,
        # every migration counts.
        unreleased = {
            version
            for version, checksum in expected.items()
            if released is None or released.get(version) != checksum
        }
        if release and unreleased:
            print(
                f"{directory}: {release} doesn't have {', '.join(map(str, sorted(unreleased)))} "
                "(added or changed since)"
            )

        for path in databases:
            name = path.relative_to(args.root)
            if path.exists():
                try:
                    applied = applied_migrations(path)
                except sqlite3.Error as error:
                    print(f"{name}: unreadable ({error})")
                    continue
            else:
                # Created at launch, or when a wallet on its network is opened.
                applied = {}

            problems = problems_with(applied, expected)
            if problems:
                details = "; ".join(problems)
                if released is not None and problems_with(applied, released):
                    details += f" (it fails in {release} too)"
                if opened_at_launch(path):
                    blocked = True
                    print(f"{name}: WOULD SHOW THE RESET DIALOG: {details}")
                else:
                    print(f"{name}: would show the reset dialog if switched to: {details}")
                continue

            pending = sorted(set(expected) - set(applied))
            new_to_release = [version for version in pending if version in unreleased]
            verb = "will be created with" if not path.exists() else "will apply"
            if new_to_release:
                will_migrate = True
                lacks = f", which {release} lacks" if release else ""
                print(f"{name}: {verb} {', '.join(map(str, pending))}{lacks}")
            elif pending:
                print(f"{name}: {verb} {', '.join(map(str, pending))}")
            else:
                print(f"{name}: up to date")

    sys.exit(2 if blocked else 1 if will_migrate else 0)


if __name__ == "__main__":
    main()
