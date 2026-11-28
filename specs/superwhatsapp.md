---
id: SUPERWA-DOC
title: SuperWhatsapp, the product
owner: selvakumarEsra
priority: high
---

<!-- id: SUPERWA-DOC -->
# SuperWhatsapp: an independent product

**SuperWhatsapp** is a fast desktop WhatsApp client with a proactive AI agent
(Znga). It is an independent project in its own repository,
`selvakumarEsra/superwhatsapp`, with its own name, logo, releases, website and
roadmap, and presents no earlier project as its parent.

Internal names say SuperWhatsapp too (REQ-SUPERWA-005..008): the crates,
data folders, keychain entries, environment, Flatpak ID and packaging. The
older names survive in exactly one place, the one-time migration that moves
an older install over, plus the macOS bundle ID for one more release. The
fork no longer merges upstream interface changes, so easy upstream merges
are no longer a goal.
SuperWhatsapp is not made by, affiliated with or endorsed by WhatsApp or
Meta; the product says so wherever it introduces itself.

<!-- id: REQ-SUPERWA-001 -->
## Everything a person sees MUST say SuperWhatsapp

Window titles, menus, the tray, notifications, dialogs, Settings › About,
both interfaces, the translations, installers, the app bundle and the
release files name the product SuperWhatsapp. The program is
`superwhatsapp`. Nothing a person sees names or links to an earlier project;
the only exception is the copyright notice the MIT license requires in
`LICENSE`, kept while that code remains.

implementations:
  - src/znga/update.rs:CONFIG
  - src/ui/settings.rs:about
  - web/src/features/settings/Settings.tsx:Settings
  - src-tauri/tauri.conf.json:productName

verifies:
  - src/updates.rs:update_config_is_valid
  - src/znga/update.rs:updates_come_from_the_fork_with_its_own_key
  - tests/update_flags.rs:update_receipt_and_error_are_taken_off_the_command_line

## Acceptance
<!-- id: REQ-SUPERWA-001.A1 -->
- No interface text, translation, installer, package metadata, README or
  website page shows "ZapFast", "FastsApp" or links to their sites or
  repositories.
<!-- id: REQ-SUPERWA-001.A2 -->
- `superwhatsapp --version` prints `superwhatsapp <version>`.
<!-- id: REQ-SUPERWA-001.A3 -->
- An existing install keeps its linked account, chats, settings and
  archive key after it is replaced by SuperWhatsapp.

<!-- id: REQ-SUPERWA-002 -->
## SuperWhatsapp MUST have its own logo

An original mark that reads as "super" plus a proactive AI agent, used for
the app icon on every platform, the tray, the website and the favicon. Its
colours follow WhatsApp's greens (`#25D366` to `#128C7E`) so the app feels
at home beside WhatsApp, but its shape MUST NOT copy WhatsApp's logo: no
phone handset in a round speech bubble.

implementations:
  - packaging/icons/superwhatsapp.svg
  - packaging/macos/icon-1024.svg

verifies:
  - src/util.rs:icon_is_opaque_in_the_middle_and_clear_at_the_corners
  - src/util.rs:the_tray_template_cuts_the_bubble_out_of_the_disc
  - src/util.rs:the_shipped_icon_draws_without_svg_filters

## Acceptance
<!-- id: REQ-SUPERWA-002.A1 -->
- The egui app, the Tauri app, the Linux desktop entry, the macOS bundle,
  the Windows executable and the website all show the new mark.
<!-- id: REQ-SUPERWA-002.A2 -->
- The mark stays legible at 16 px (tray, favicon).

<!-- id: REQ-SUPERWA-003 -->
## Releases MUST come from the SuperWhatsapp repository

The Release workflow builds from a `v*` tag on `main` of
`selvakumarEsra/superwhatsapp`, names its files `superwhatsapp-<tag>-<target>`
(and never publishes `superwhatsapp-preview-*` packages), signs
`checksums.txt` with the fork's update key, and publishes a draft for
prereleases. Installed copies update from this repository only, accept the
earlier names `zapfast` and `fastsapp` as their own, and trust only the
fork's key. Packages for third-party stores (AUR, Homebrew, winget) are not
published from this repository until they are set up under the new name.

implementations:
  - .github/workflows/release.yml:jobs
  - native-packages.yaml
  - src/znga/update.rs:REPOSITORY

## Acceptance
<!-- id: REQ-SUPERWA-003.A1 -->
- A tag on `main` produces macOS, Windows (x64, arm64), Linux (x86_64,
  aarch64) and Flatpak files named `superwhatsapp-*`, plus `checksums.txt`
  and its signature.
<!-- id: REQ-SUPERWA-003.A2 -->
- A tag not on `main` builds nothing.
<!-- id: REQ-SUPERWA-003.A3 -->
- The workflow can also run by hand to build every file without
  publishing, to check a change to it.
<!-- id: REQ-SUPERWA-003.A4 -->
- No job publishes to AUR, Homebrew or winget under upstream's names.

<!-- id: REQ-SUPERWA-004 -->
## The website MUST present SuperWhatsapp as a product

The docs site, published on GitHub Pages from this repository, leads with
what SuperWhatsapp does for a person (speed, privacy, the proactive AI
agent), shows it, and covers download, setup, every feature, Znga, privacy,
settings and files, and the FAQ. Download links point at this
repository's latest release files.

implementations:
  - docs/index.md
  - docs/_config.yml
  - .github/workflows/docs.yml:jobs

## Acceptance
<!-- id: REQ-SUPERWA-004.A1 -->
- The home page states the product, its three main benefits and a download
  button within the first screen, on desktop and phone widths.
<!-- id: REQ-SUPERWA-004.A2 -->
- Every guide and reference page names the product SuperWhatsapp and links
  only to pages and files that exist.
<!-- id: REQ-SUPERWA-004.A3 -->
- The site says SuperWhatsapp is not affiliated with WhatsApp or Meta, and
  that it is open source under the MIT license.

<!-- id: REQ-SUPERWA-005 -->
## An existing install MUST move to the SuperWhatsapp folders and keys once

When SuperWhatsapp starts and its own data folder does not exist yet, it
moves the data of an older install (the `me.paolino.zapfast` folders on
macOS, `zapfast` on Linux, `paolino\zapfast` on Windows, and the still older
`fastsapp` and `fastwhatsapp` ones) into its own folders before anything
opens a file, and copies each encrypted archive's key to its keychain entry
under the new name and folder. The archive is the only copy of the history,
so a folder moves only when every key it needs was copied and read back; if
one cannot be, nothing moves, the old install stays as it was and the start
fails with the reason. A copy of an older install that is still running is
asked first, through its own single-instance slot, to show its window, and
the new start then exits, so nothing moves while it runs. Once the new
folder exists the migration never runs again and the older folders are
never read or written: when both exist, the new ones are used unchanged,
the old ones are left alone and the log warns once. Old keychain entries are
left in place, so a damaged move can be recovered. Saved Znga API keys move
the same way the first time they are read.

implementations:
  - src/migration.rs:AppDirs.adopt_previous_names
  - src/archive/encryption.rs:move_archive_key
  - src/archive/encryption.rs:key_for
  - src/znga/keys.rs:stored
  - src/single_instance.rs:acquire_after_previous
  - src/autostart.rs:move_previous_entry

verifies:
  - src/migration.rs:an_older_install_moves_with_its_archive_keys
  - src/migration.rs:a_missing_key_moves_nothing
  - src/migration.rs:the_new_folders_win_over_old_ones
  - src/migration.rs:the_move_runs_only_while_superwhatsapp_has_no_folder

## Acceptance
<!-- id: REQ-SUPERWA-005.A1 -->
- After upgrading from 0.21.1-beta.10 or beta.11, Settings shows the archive
  under the SuperWhatsapp folder (on macOS
  `~/Library/Application Support/cc.superwhatsapp`), the account is still
  linked and the history opens.
<!-- id: REQ-SUPERWA-005.A2 -->
- When an archive's key cannot be read, no folder moves and the old install
  still opens with an older SuperWhatsapp.
<!-- id: REQ-SUPERWA-005.A3 -->
- When both old and new folders exist, the new ones are used unchanged and
  nothing in the old ones is read, written or deleted.
<!-- id: REQ-SUPERWA-005.A4 -->
- A second start after a successful move does not look at the old folders.

<!-- id: REQ-SUPERWA-006 -->
## A new install MUST use only SuperWhatsapp names

A computer that never had an older install gets only SuperWhatsapp folders
(`cc.superwhatsapp` on macOS, `superwhatsapp` on Linux,
`cc\superwhatsapp` on Windows), keychain entries under `cc.superwhatsapp`,
the log file `superwhatsapp.log`, and no folder, file or keychain entry with
an older name is created. The same holds after a migration: nothing creates
an old-named folder again.

implementations:
  - src/paths.rs:AppDirs.discover
  - src/archive/encryption.rs:SERVICE

verifies:
  - src/migration.rs:a_new_install_uses_only_superwhatsapp_folders

## Acceptance
<!-- id: REQ-SUPERWA-006.A1 -->
- On a fresh account, after linking, no `zapfast`, `fastsapp` or `me.paolino`
  folder exists and the keychain holds no `rocks.zapfast` or
  `me.paolino.zapfast` entry.
<!-- id: REQ-SUPERWA-006.A2 -->
- After a migration, using every feature (chats, calls, Znga, themes,
  favourites, insights, keep, autostart) creates nothing under an old name.

<!-- id: REQ-SUPERWA-007 -->
## Internal names MUST say SuperWhatsapp

The Rust crates are `superwhatsapp` and `superwhatsapp-desktop`, the
environment variables start with `SUPERWHATSAPP_`, temporary files, the log
and log targets say superwhatsapp, the Linux Flatpak ID is
`cc.superwhatsapp.SuperWhatsapp`, and the packaging recipes (Flatpak, Arch,
RPM, winget, Homebrew, Omarchy) are named superwhatsapp. The older folder,
keychain, login item and program names appear only in the one migration
module of REQ-SUPERWA-005 (`src/migration.rs`), which the keychain, login
item and updater code take them from, and in the user documentation that
explains the move. What else stays, because it identifies something outside
this repository: the `fastsapp` single-instance wire name older copies
answer on, and the package names (`fastsapp`) that packages replace; the
macOS bundle ID until REQ-SUPERWA-008 completes; the upstream update
configuration of builds without the `znga` feature (`src/updates.rs`); the
libraries SuperWhatsapp is built on, their repositories and upstream issue
numbers; the copyright notice the MIT license requires; and the
`ZAPFAST_UPDATE_SIGNING_KEY` and `ZAPFAST_GIPHY_KEY` repository secrets,
which the release workflow accepts after the `SUPERWHATSAPP_` ones until
they are renamed in GitHub.

implementations:
  - Cargo.toml:package
  - src-tauri/Cargo.toml:package

## Acceptance
<!-- id: REQ-SUPERWA-007.A1 -->
- Outside the listed exceptions, a search of the repository for "zapfast"
  (any case) finds nothing, and "me.paolino.zapfast" appears only in
  `src/migration.rs` and the user documentation.
<!-- id: REQ-SUPERWA-007.A2 -->
- The build, the test suite and the release workflow pass under the new
  names, with the same tests passing as before the rename.

<!-- id: REQ-SUPERWA-008 -->
## The macOS bundle ID MUST move to cc.superwhatsapp over two releases

An installed copy refuses an update whose bundle ID it does not know, so
the first release with the internal rename keeps the bundle ID
`me.paolino.fastsapp` but accepts `cc.superwhatsapp` in updates, and the
release after that ships as `cc.superwhatsapp`. macOS asks again for
camera, microphone and notification permission after the switch.

implementations:
  - src/znga/update.rs:CONFIG
  - packaging/macos/Info.plist

## Acceptance
<!-- id: REQ-SUPERWA-008.A1 -->
- A copy of the first release updates in place to a bundle with ID
  `cc.superwhatsapp`.
<!-- id: REQ-SUPERWA-008.A2 -->
- The first release itself still has bundle ID `me.paolino.fastsapp`, so
  0.21.1-beta.10 and beta.11 update to it in place.
