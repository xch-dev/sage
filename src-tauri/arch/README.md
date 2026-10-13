# Arch Linux package

On Arch Linux (x86_64), install `base-devel`, then build as a normal user:

```bash
cd src-tauri/arch
makepkg -s
sudo pacman -U "$(makepkg --packagelist)"
```

`makepkg -si` builds and installs in one step. The recipe requires the complete
Sage checkout and builds its current working tree, including uncommitted changes.
To build a release, check out its tag first. The version comes from
`src-tauri/tauri.conf.json`; makepkg updates `pkgver` and resets `pkgrel` to `1`
when the app version changes. Alphabetic prerelease labels such as `-rc.1` become
`rc.1` so Arch orders them before the final release.

From the repository root, after `pnpm install`, the equivalent command is:

```bash
pnpm package:arch
sudo pacman -U target/release/bundle/arch/sage-wallet-*.pkg.tar.zst
```

This uses the same recipe and writes the package under `target/release/bundle/arch`,
alongside the other Tauri bundles. Additional arguments go to makepkg, for example
`pnpm package:arch --noconfirm`. Use the specific package filename if the output
directory contains several versions.

The recipe uses the existing Tauri build and its `beforeBuildCommand` to build
the frontend and built-in apps. Rustup and pnpm select the versions pinned in
`rust-toolchain.toml` and `package.json`. Preparation downloads dependencies;
Cargo and pnpm use their normal caches, and Rust output stays in `target/`.
GCC from `base-devel` is the supported C compiler. AWS-LC 0.34 needs a scoped
compiler flag for its old `memchr` wrapper with glibc 2.44; recheck it when
updating that dependency.

Arch is an entry in the existing CI build matrix, using an Arch container and
a non-root makepkg user. The recipe's `check()` hook validates the desktop entry
and launcher syntax and runs the working-tree version and launcher tests. CI
uploads the package and attaches it to `v*` releases. Run
`node --test scripts/package-arch.test.mjs` on Arch to run those tests separately.

Launch **Sage** from the application menu or run `sage-tauri`. The package contains
the wallet, launcher, desktop entry, icons, and built-in apps. Tauri loads the
built-in apps from `/usr/lib/Sage/builtin-apps` (case sensitive).

On Wayland, the launcher defaults `__NV_DISABLE_EXPLICIT_SYNC` to `1` to avoid
the NVIDIA/WebKitGTK startup failure reported as "Error 71 (Protocol error)".
This [Tauri workaround](https://v2.tauri.app/develop/debug/linux-graphics/) keeps
hardware acceleration enabled. Override it with
`__NV_DISABLE_EXPLICIT_SYNC=0 sage-tauri` when testing newer drivers or WebKitGTK.

Sage sizes its main webview to GTK's content area when using native Wayland,
keeping bottom controls such as Logout visible below the title bar. This uses
the active GTK backend; X11 and XWayland retain Tauri's existing resize behavior.

Increment `pkgrel` for packaging changes within an app version. ARM builds,
AUR publication, and a pacman repository are not configured.
