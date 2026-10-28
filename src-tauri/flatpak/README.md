# Standalone Flatpak

The x86_64 Flatpak repackages the DEB produced by the Ubuntu 22.04 Linux build.
It uses the GNOME 50 runtime and is distributed as a `.flatpak` release asset.

## Build

Install the usual [development prerequisites](../../README.md#development),
plus `flatpak`, `flatpak-builder`, and `dpkg-deb` (`dpkg` on Arch Linux).
Install the runtime and SDK once:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.gnome.Platform//50 org.gnome.Sdk//50
```

Build the DEB on Ubuntu 22.04, then package it:

```sh
pnpm install --frozen-lockfile
pnpm exec tauri build --bundles deb -- --locked
pnpm package:flatpak target/release/bundle/deb/*.deb
```

The packaging command can also run on another Linux distribution with a DEB
copied from that Ubuntu build. A binary built on a newer distribution may
require libraries newer than the Flatpak runtime provides; the package checks
reject unresolved libraries and symbols before producing a bundle.

Output: `target/release/bundle/flatpak/Sage_<version>_x64.flatpak`.
The version comes from the input DEB. Build files stay under `target/flatpak/`.
The Linux x64 CI job packages its existing DEB and uploads the Flatpak alongside
the other artifacts, including tagged GitHub releases.

## Install and update

```sh
flatpak install --user ./Sage_<version>_x64.flatpak
flatpak run com.rigidnetwork.sage
```

Flatpak downloads the GNOME runtime from Flathub if needed. Sage itself does
not need a Flathub listing. Install the next downloaded bundle with the same
command to update; this distribution does not configure an application update
repository.

Wallet data is stored under `~/.var/app/com.rigidnetwork.sage/`, separately from
native installations. Import wallets through Sage's UI when needed. The sandbox
allows networking and graphics; file import/export uses desktop portals.

## NVIDIA and Wayland

Sage sets `__NV_DISABLE_EXPLICIT_SYNC=1` inside its Flatpak to avoid WebKitGTK
exiting with Wayland Error 71 on NVIDIA. This uses Flatpak's environment settings.
To test without the workaround:

```sh
flatpak run --env=__NV_DISABLE_EXPLICIT_SYNC=0 com.rigidnetwork.sage
```

See [Tauri's Linux graphics guidance](https://v2.tauri.app/develop/debug/linux-graphics/).
Remove the setting after confirming an updated runtime/driver works without it.

## Checks

Packaging checks library and symbol resolution inside the GNOME runtime.
To rerun this check:

```sh
pnpm check:flatpak-package target/flatpak/build
```

Before releasing, install the bundle and test with a disposable wallet: restart
persistence, peer syncing, built-in apps, file import/export, external links,
WalletConnect, and Wayland/X11 rendering.
