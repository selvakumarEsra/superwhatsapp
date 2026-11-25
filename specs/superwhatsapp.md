---
id: SUPERWA-DOC
title: SuperWhatsapp, the product
owner: selvakumarEsra
priority: high
---

<!-- id: SUPERWA-DOC -->
# SuperWhatsapp: the fork becomes its own product

The fork of ZapFast with Znga becomes **SuperWhatsapp**: a fast desktop
WhatsApp client with a proactive AI agent (Znga). It lives in its own
repository, `selvakumarEsra/superwhatsapp`, with its own name, logo,
releases and website.

Non-goals: renaming internal identifiers. The Rust library stays `zapfast`,
and the data folders, keychain entries, bundle identifiers, Flatpak ID and
single-instance slot keep their names, so an upgrade keeps the linked
account, the chats and the archive key, and upstream merges stay easy.
SuperWhatsapp is not made by, affiliated with or endorsed by WhatsApp or
Meta; the product says so wherever it introduces itself.

<!-- id: REQ-SUPERWA-001 -->
## Everything a person sees MUST say SuperWhatsapp

Window titles, menus, the tray, notifications, dialogs, Settings › About,
both interfaces, the translations, installers, the app bundle and the
release files name the product SuperWhatsapp. The program is
`superwhatsapp`. History (release notes of earlier versions, the
changelog) and credit to upstream ZapFast stay as written.

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
- No interface text, translation or installer shows "ZapFast" except where
  it credits upstream or describes an earlier version.
<!-- id: REQ-SUPERWA-001.A2 -->
- `superwhatsapp --version` prints `superwhatsapp <version>`.
<!-- id: REQ-SUPERWA-001.A3 -->
- An existing install keeps its linked account, chats, settings and
  archive key after it is replaced by SuperWhatsapp.

<!-- id: REQ-SUPERWA-002 -->
## SuperWhatsapp MUST have its own logo

An original mark that reads as "super" plus a proactive AI agent, used for
the app icon on every platform, the tray, the website and the favicon. It
MUST NOT resemble WhatsApp's logo (no phone handset in a speech bubble, no
WhatsApp green as the main colour).

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
(and `superwhatsapp-preview-*` for the new interface), signs
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
  credits ZapFast under its MIT license.
