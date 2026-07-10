# Distribution

This project is a native Rust/iced desktop application. The first distribution
target should keep packaging close to Cargo and avoid adding a web app bundler.

## Recommended baseline

Use `cargo-packager` as the primary packager.

It supports the target formats needed for the first public builds:

- macOS: `.app`, `.dmg`
- Linux: `.AppImage`, `.deb`, pacman package/PKGBUILD
- Windows: `.msi` through WiX Toolset, plus optional NSIS `.exe`

The main tradeoff is that `cargo-packager` is still marked public preview by
CrabNebula, so release builds should be tested on each target OS before tagging.
For this project that is still a good fit because it covers all major desktop
installer formats from one Rust-oriented configuration.

## Required project metadata

Before enabling release packaging, add or confirm:

- Stable app identifier: `io.github.<owner>.vrcx-but-in-rust` or another reverse-DNS ID.
- Product name: `VRCX-BIR` or `VRCX But In Rust`.
- License metadata in `Cargo.toml` matching `LICENCE`.
- Short package description.
- Repository/homepage once the project has a canonical URL.
- Icons:
  - Linux: PNG, preferably 32/128/256/512px.
  - Windows: `.ico`.
  - macOS: `.icns`.
- Linux desktop entry category, likely `Network` or `Utility`.

## Local commands

Install the packager:

```sh
cargo install cargo-packager --locked
```

Build a release binary:

```sh
cargo build --release
```

Package from the repository root:

```sh
cargo packager --release
```

The packager does not build by default unless configured with a
`before-packaging-command`, so either keep the explicit build step or configure
the build command in `packager.toml`. This repository uses `packager.toml`.

Useful format-specific commands:

```sh
cargo packager --release --formats app
CI=true cargo packager --release --formats app --formats dmg
cargo packager --release --formats appimage --formats deb --formats pacman
cargo packager --release --formats wix
```

Use `CI=true` for local DMG testing if Finder AppleScript times out while
prettifying the DMG window.

After generating a macOS DMG, patch its volume icon with:

```sh
scripts/set-dmg-volume-icon.sh target/dist/*.dmg assets/icons/vrcx-bir-volume.icns
```

## Per-platform notes

### macOS DMG

Build on macOS. Create both `.app` and `.dmg`.

For local testing, unsigned packages are acceptable. For public distribution,
expect to add Developer ID signing and notarization; otherwise Gatekeeper will
warn or block users depending on their settings.

### Linux AppImage

Build AppImage on Linux. Prefer an older Ubuntu LTS runner/container for better
glibc compatibility with older user systems.

AppImage needs a desktop entry and icon to feel native. Test on at least Ubuntu,
Fedora, and an Arch-based distro because this app uses a native GUI stack and may
pull graphics/windowing libraries.

### Linux distribution packages

Start with `.deb` and pacman output from `cargo-packager`.

If RPM becomes a hard requirement, add a second Linux packaging layer:

- `nFPM` for shared `.deb`, `.rpm`, `.apk`, Arch, and related package formats
  from one YAML config.
- Or `cargo-generate-rpm` for a Rust/Cargo-focused `.rpm` path.

For repository-grade distro packages, system dependencies and desktop metadata
should be explicit instead of relying only on bundled artifacts.

### Windows MSI

Build MSI on Windows. `cargo-packager` uses WiX Toolset for `.msi` output.

For public release, add code signing later. Unsigned MSI installers are useful
for CI artifacts and tester builds, but Windows SmartScreen warnings are expected
until reputation/signing is in place.

## Suggested release matrix

Use native CI runners instead of heavy cross-compilation for the first pass:

- `macos-latest`: build `.app` and `.dmg`
- `ubuntu-22.04` or older supported LTS: build `.AppImage`, `.deb`, pacman package
- `windows-latest`: build `.msi`

Each release job should:

1. Check formatting/lints/tests.
2. Build `cargo build --release`.
3. Run `cargo packager --release`.
4. Upload generated artifacts.

## Current setup

The repository includes `packager.toml` with:

- app identifier: `com.drapnard.vrcxbir`
- product name: `VRCX But In Rust`
- package name: `vrcx-but-in-rust`
- MIT license file: `LICENCE`
- final PNG, ICO, and ICNS icons in `assets/icons`
- GitHub Actions packaging workflow in `.github/workflows/package.yml`

The app icon uses `assets/icons/vrcx-bir.icns` on macOS and
`assets/icons/vrcx-bir.png`/`assets/icons/vrcx-bir.ico` for Linux/Windows. The
DMG mounted volume icon uses `assets/icons/vrcx-bir-volume.icns`.
