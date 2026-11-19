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

Launching opens the apps database, and the active wallet's database and shared database on the
active wallet's network, so a problem in one of those blocks the launch. Other databases are only
opened when the user switches wallet or network, so a problem in one of those is a warning.

Exit status: 0 if launching changes nothing the release depends on, 1 if it will apply migrations
the release doesn't have (ask the user first), 2 if it would bring up the reset dialog (don't
launch). Databases it doesn't open never raise the status, since only switching to them would
migrate them or bring up the dialog.
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


def toml_section(text, header):
    """The body of a `[header]` section, or of each `[[header]]` table in an array."""
    pattern = rf"^\[{re.escape(header)}\]$(.*?)(?=^\[|\Z)"
    return re.findall(pattern, text, re.M | re.S)


def toml_value(section, key):
    match = re.search(rf"^{re.escape(key)}\s*=\s*\"?([^\"\n]*?)\"?\s*$", section, re.M)
    return match.group(1) if match else None


def read(path):
    try:
        return path.read_text()
    except OSError:
        return ""


def launch_databases(root):
    """
    The active wallet's fingerprint, and the wallet and shared databases a launch opens for it.
    This follows `wallet_db_path` and `shared_db_path` in `crates/sage/src/sage.rs`: the wallet's
    network in wallets.toml, or else the default network. For a custom network, whose ID can differ
    from its name, the shared database's name is a guess.
    """
    config = read(root / "config.toml")
    global_sections = toml_section(config, "global")
    fingerprint = toml_value(global_sections[0], "fingerprint") if global_sections else None
    if fingerprint is None:
        return None, set()

    network = None
    for wallet in toml_section(read(root / "wallets.toml"), "[wallets]"):
        if toml_value(wallet, "fingerprint") == fingerprint:
            network = toml_value(wallet, "network")
            break
    if network is None:
        network_sections = toml_section(config, "network")
        network = toml_value(network_sections[0], "default_network") if network_sections else None
    if network is None:
        # The default in `sage-config`, used when config.toml doesn't set one.
        network = "mainnet"

    return fingerprint, {
        root / "wallets" / fingerprint / f"{network}.sqlite",
        root / "shared" / f"{network}.sqlite",
    }


def release_has_shared_databases(tag):
    """Whether a release has the per-network shared databases, which came after v0.13.0."""
    return (
        subprocess.run(
            ["git", "grep", "--quiet", "shared_db_path", tag, "--", "crates/sage/src/sage.rs"],
            capture_output=True,
        ).returncode
        == 0
    )


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

    active, opened = launch_databases(args.root)
    opened.add(args.root / "sage-apps.sqlite3")
    print(f"active wallet: {active or 'none'}")

    wallet_databases = set((args.root / "wallets").glob("*/*.sqlite"))
    wallet_databases |= set((args.root / "shared").glob("*.sqlite"))
    # The launch creates the active wallet's databases if they don't exist yet.
    wallet_databases |= {path for path in opened if path.name.endswith(".sqlite")}
    sets = [
        (WALLET_MIGRATIONS, sorted(wallet_databases)),
        (APPS_MIGRATIONS, [args.root / "sage-apps.sqlite3"]),
    ]

    # A release without shared databases never opens them, so they can't fail in it.
    shared_in_release = release is None or release_has_shared_databases(release)

    def compared_with_release(path):
        return release is not None and (
            shared_in_release or path.relative_to(args.root).parts[0] != "shared"
        )

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
                if compared_with_release(path) and problems_with(applied, released):
                    details += f" (it fails in {release} too)"
                if path in opened:
                    blocked = True
                    print(f"{name}: WOULD SHOW THE RESET DIALOG: {details}")
                else:
                    print(f"{name}: would show the reset dialog if opened: {details}")
                continue

            pending = sorted(set(expected) - set(applied))
            new_to_release = [
                version
                for version in pending
                if version in unreleased and (release is None or compared_with_release(path))
            ]
            verb = "will be created with" if not path.exists() else "will apply"
            if not pending:
                print(f"{name}: up to date")
                continue
            lacks = f", which {release} lacks" if release and new_to_release else ""
            # Only the databases the launch opens are migrated by it.
            when = "" if path in opened else " if opened"
            if new_to_release and path in opened:
                will_migrate = True
            print(f"{name}: {verb} {', '.join(map(str, pending))}{lacks}{when}")

    sys.exit(2 if blocked else 1 if will_migrate else 0)


if __name__ == "__main__":
    main()
