---
id: WEBUI-DOC
title: Tauri 2 + Solid.js interface — replacing egui
owner: selvakumarEsra
priority: high
version: 1
---

<!-- id: WEBUI-DOC -->
# Tauri 2 + Solid.js interface — replacing egui

ZapFast's interface moves from egui to a Tauri 2 shell with a Solid.js,
TypeScript and Tailwind front end, built with Vite, the stack OpenTrader
uses. The goal is a look and feel that matches WhatsApp Desktop for
macOS pixel for pixel: its layout, spacing, typography, colours, motion
and interactions.

The egui interface is removed, not kept alongside. The Rust code that is
not interface (the WhatsApp worker, archive, accounts, media, calls audio,
Znga, extras logic) stays and becomes the core library the
Tauri shell links. Everything ZapFast does today must work in the new
interface before it ships as a release.

Layout in the repository, as in OpenTrader:

- `src/` — the core library crate `zapfast` (no egui).
- `src-tauri/` — the Tauri 2 app: window, IPC commands, events, tray,
  updater, single instance, deep links.
- `web/` — the Solid.js front end (`web/src/`), Tailwind styles, Vite
  config, front-end tests.

**Decisions taken with this spec.** These override earlier rules:

- AGENTS.md's "no browser engine" boundary no longer applies to this fork.
  The system web view (WebView2 on Windows, WKWebView on macOS, WebKitGTK
  on Linux) draws the interface. Every other product boundary stays:
  WhatsApp only, no automation, no telemetry, no hosted backend, nothing
  for one platform only.
- The fork stops merging upstream's interface changes. Upstream fixes to
  the core (worker, archive, protocol, media) are still merged.
- The fork rule "no widgets inside upstream panels" ends with egui. Fork
  features (Znga, extras) become ordinary parts of the web
  interface, each in its own folder under `web/src/features/`.

Non-goals: a hosted web app or a browser tab version (the interface runs
only inside the Tauri window, never served over the network); a mobile
build; copying WhatsApp's logos, trademarks, sounds or proprietary
artwork. The look is matched with original assets drawn to the same
size and position (icons, the bundled Noto emoji, the wallpaper pattern)
and the same system font WhatsApp Desktop uses.

Mini-apps are removed from ZapFast with this move (the `miniapps`
feature, `src/miniapps/` and specs/miniapps.md); they remain in the
v0.21.1-beta.5 release and in git history.

How this spec is used: one document for the whole move, as decided, with
one requirement per area so each can be built and checked on its own.
Tests do not exist yet, so no requirement declares `verifies:` until its
tests are written; each area's tests are listed in its acceptance
criteria.

<!-- id: REQ-WEBUI-001 -->
## The core MUST be a library with no interface code, and egui MUST be removed

`src/` builds as the library crate with no dependency on egui, eframe,
egui_extras, winit or the fastframe interface crates (fastframe crates
with no interface, such as file writes, fonts data or power events, may
stay). `src/ui/`, `src/demo.rs`'s egui harness and every `frame_ui` path
are deleted. What `src/app.rs` did beyond drawing (applying `Action`s and
`Event`s to account state, timers, notifications, read receipts) moves to
a core module, `src/state/`, that the Tauri shell drives.

implementations:
  - src/state/mod.rs:State
  - src/state/mod.rs:State.apply
  - src-tauri/src/main.rs:main

## Acceptance
<!-- id: REQ-WEBUI-001.A1 -->
- `cargo tree -p zapfast` lists no egui, eframe, egui_extras or winit.
<!-- id: REQ-WEBUI-001.A2 -->
- `cargo test -p zapfast` runs the core's tests (archive, worker
  translation, Znga, extras logic) with no window and no web
  view.
<!-- id: REQ-WEBUI-001.A3 -->
- `cargo tauri build` produces the app from `src-tauri/` on Linux,
  Windows and macOS.
<!-- id: REQ-WEBUI-001.A4 -->
- No existing user data changes: the same `state/accounts/<id>/` and
  `cache/accounts/<id>/` folders, archive keys and settings files are
  opened by the new app as they are, and an account linked in the egui
  build stays linked.

<!-- id: REQ-WEBUI-002 -->
## The interface MUST talk to the core only through typed commands and events

The core keeps the state; the front end shows it and asks for changes.
The front end calls typed Tauri commands that become `model::Action`s
(the same rule as today: views never change state directly). The core
pushes typed events with the state the front end needs (chat list
changes, a conversation's messages, account status, dialogs, Znga and
Znga updates). Types are generated from Rust into
`web/src/bindings.ts` (tauri-specta), so the two sides cannot drift.
Large lists are sent as windows and changes, not whole on every update.

implementations:
  - src-tauri/src/commands.rs:dispatch
  - src-tauri/src/events.rs:Bridge
  - web/src/bindings.ts:commands

## Acceptance
<!-- id: REQ-WEBUI-002.A1 -->
- Every user action in the interface goes through a generated command;
  the front end has no other way to reach the core.
<!-- id: REQ-WEBUI-002.A2 -->
- Changing a command's or event's Rust type without regenerating the
  bindings fails the build (`cargo test` checks the bindings are current).
<!-- id: REQ-WEBUI-002.A3 -->
- An account with 5,000 chats and a conversation with 50,000 messages
  stay responsive: the front end receives only the visible window plus a
  margin, and a new message sends a single change, not the whole list.
<!-- id: REQ-WEBUI-002.A4 -->
- Events from an account not on screen update only that account's state,
  as today (no composer, dialog, playback or read-state changes).

<!-- id: REQ-WEBUI-003 -->
## The web view MUST stay private and closed to everything but the core

The web view loads only the bundled front end. A strict Content Security
Policy blocks remote scripts, styles, frames and connections; link
previews, avatars, media and stickers reach the page only through a
custom protocol the core serves from the account's own cache. Tauri's
capabilities allow only ZapFast's own commands and the plugins it uses.
No other program can call the commands. Message text, numbers, keys and
QR payloads are never logged from either side.

implementations:
  - src-tauri/tauri.conf.json:app
  - src-tauri/src/protocol.rs:serve

## Acceptance
<!-- id: REQ-WEBUI-003.A1 -->
- With the CSP in place, an `<img>`, `fetch`, `<script>` or `<iframe>`
  pointing at any remote address is refused; a link in a message opens
  in the system browser, never in the window.
<!-- id: REQ-WEBUI-003.A2 -->
- The media protocol serves files only from the active account's cache
  and download folders; a path outside them, or another account's
  folder, returns an error.
<!-- id: REQ-WEBUI-003.A3 -->
- No developer tools in release builds; no local HTTP server or socket is
  opened for the interface.
<!-- id: REQ-WEBUI-003.A4 -->
- A message containing HTML or script text is shown as text, never run
  or rendered as markup (message formatting is applied by building
  elements, not by inserting HTML).

<!-- id: REQ-WEBUI-004 -->
## The interface MUST match WhatsApp Desktop for macOS pixel for pixel

The reference is the WhatsApp Desktop app for macOS, captured on a fixed
date and kept as the baseline until it is deliberately re-captured.
Every screen ZapFast shares with WhatsApp Desktop matches it pixel for
pixel at the same window size: the labelled sidebar, chat list, conversation, composer, info panel, menus,
dialogs, settings and the empty "no chat open" pane. That covers
geometry (widths, heights, paddings, radii, borders, shadows), colours
in light and dark, the font stack, sizes, weights and line heights,
bubble shapes and tails, tick and time styling, hover, pressed, focus
and selected states, and motion (durations and easing). All values live
as design tokens in `web/src/styles/tokens.css`; components use only
tokens, never literal values.

Where ZapFast has something WhatsApp Desktop lacks (Znga, Insights,
Keep, Favourites in the sidebar, several accounts, the app lock), it is drawn
with the same tokens and component patterns so it looks native to the
same design.

How the baseline is taken: the person captures WhatsApp Desktop
themselves, from their own logged-in Mac. The first capture (2026-11-24)
is the dark theme, chat list and an open one-to-one chat, in a window of
1288×806 points; the light theme and the remaining screens are captured
the same way before they are matched [needs review]. Only measurements (computed sizes, colours,
fonts) go into `tokens.css`; the captures contain personal chats, so
they stay outside the repository and outside pull requests. Pixel
comparisons use the demo archive arranged like the capture (same number
of rows, message kinds and lengths), so text differs but structure does
not.

implementations:
  - web/src/styles/tokens.css:root
  - web/src/App.tsx:App

## Acceptance
<!-- id: REQ-WEBUI-004.A1 -->
- For each captured screen, at the capture's window size and theme, every
  element's box (position and size) is within 1 px of the baseline.
<!-- id: REQ-WEBUI-004.A2 -->
- Every colour in the token file equals the baseline's computed colour
  exactly, in both themes.
<!-- id: REQ-WEBUI-004.A3 -->
- Font family, size, weight and line height of every text style equal
  the baseline's.
<!-- id: REQ-WEBUI-004.A4 -->
- Hover, pressed, selected and keyboard-focus states of list rows,
  buttons, menu items and bubbles match the baseline's.
<!-- id: REQ-WEBUI-004.A5 -->
- Panel slides, menu fades and the scroll-to-bottom button use the
  baseline's durations and easing; with "reduce motion" set they appear
  without animation.
<!-- id: REQ-WEBUI-004.A6 -->
- No WhatsApp logo, trademark, icon file, sound or wallpaper artwork is
  bundled; icons and the wallpaper pattern are original and drawn to the
  baseline's size and position.
<!-- id: REQ-WEBUI-004.A7 -->
- No component contains a literal colour, size or duration; a lint
  check fails the build on one.
<!-- id: REQ-WEBUI-004.A8 -->
- Below the width where WhatsApp Desktop hides its sidebar or stacks its
  panels, ZapFast does the same, and Back returns to the list.

<!-- id: REQ-WEBUI-005 -->
## Linking, accounts and the app lock MUST work as they do today

Linking by QR code and by phone-number code, several linked accounts with
the switcher on our avatar, adding and removing accounts, and the app
lock screen.

implementations:
  - web/src/features/login/Login.tsx:Login
  - web/src/features/accounts/Switcher.tsx:Switcher
  - web/src/features/lock/Lock.tsx:Lock

## Acceptance
<!-- id: REQ-WEBUI-005.A1 -->
- A new account links by QR and by pairing code; the QR refreshes as
  WhatsApp rotates it.
<!-- id: REQ-WEBUI-005.A2 -->
- Switching accounts shows only that account's chats, and unread dots
  show for the others.
<!-- id: REQ-WEBUI-005.A3 -->
- With the app lock on, nothing of any chat is drawn until it unlocks.
<!-- id: REQ-WEBUI-005.A4 -->
- Removing an account deletes its folders and keyring entry as today.
<!-- id: REQ-WEBUI-005.A5 -->
- When the archive's keyring key is locked or missing, linking stops with
  the same explanation as today; no new, empty archive is created.

<!-- id: REQ-WEBUI-006 -->
## The chat list MUST offer everything it does today

Search (chats, contacts and messages), the filters (All, Unread,
Private, Favourites, Groups, Channels) and labels, pinned chats, typing
and recording states, unread counts and mentions, mute, archive, the
locked folder, Starred, the compact avatar column when the list is
hidden, the chat menu (pin, mute, archive, mark read or unread, favourite,
labels, lock, block, delete, clear, export), and drafts shown in rows.

As on WhatsApp Desktop for macOS, the full list (All, no search) is grouped
under collapsible headings: **Favourites** first, holding the favourite
chats in the phone's favourites order as one-line rows, then **Chats**,
holding every other chat with the pinned ones on top. Any filter or search
shows one ungrouped list.

implementations:
  - web/src/features/chats/ChatList.tsx:ChatList
  - web/src/features/chats/ChatRow.tsx:ChatRow

## Acceptance
<!-- id: REQ-WEBUI-006.A1 -->
- 5,000 chats scroll at the display's refresh rate (virtualised rows).
<!-- id: REQ-WEBUI-006.A2 -->
- Each filter, label and folder shows the same chats as the egui build
  for the same archive.
<!-- id: REQ-WEBUI-006.A3 -->
- Every chat menu entry performs its action and the row updates without
  reloading the list.
<!-- id: REQ-WEBUI-006.A4 -->
- With All chosen and no search, the list shows a Favourites heading over
  the favourite chats and a Chats heading over the rest; a favourite chat
  appears only under Favourites. Clicking a heading folds or unfolds its
  chats, and the fold is remembered on this computer. With no favourites,
  the Favourites heading is not shown.
<!-- id: REQ-WEBUI-006.A5 -->
- With any other filter, or while searching, the list is not grouped.

<!-- id: REQ-WEBUI-007 -->
## The conversation MUST show every message kind and action

Text with WhatsApp formatting, links with previews, images, videos, GIFs,
round videos, voice notes, audio, documents, stickers (still, animated
and Lottie), contacts, locations and live locations, polls, events,
view-once, motion photos, deleted and edited messages, system messages,
replies and quotes, reactions, forwarded labels, ticks and times, date
separators, unread separator, and "load older" from the archive and the
phone. Actions: reply, react, forward (with the Favourites tab), copy,
star, pin, edit, delete for me and for everyone, message info, select
several, search in the chat with the date filter, jump to a reply.

implementations:
  - web/src/features/conversation/Conversation.tsx:Conversation
  - web/src/features/conversation/Message.tsx:Message

## Acceptance
<!-- id: REQ-WEBUI-007.A1 -->
- The demo archive (every kind above) renders each message kind; a
  front-end test checks each kind against a stored expectation.
<!-- id: REQ-WEBUI-007.A2 -->
- A 50,000-message conversation opens at the newest message in under
  300 ms and scrolls without dropped frames; older pages load as the top
  is reached without the view jumping.
<!-- id: REQ-WEBUI-007.A3 -->
- Right-to-left and mixed-direction text, emoji and non-Latin text
  display and select correctly.
<!-- id: REQ-WEBUI-007.A4 -->
- Each action listed reaches the core as its `Action` and the message
  updates in place.

<!-- id: REQ-WEBUI-008 -->
## The composer MUST offer everything it does today

Multi-line text with formatting shortcuts, emoji and `:shortcode`
completion, mentions in groups, the plus menu (photos and videos,
documents, camera, contact, poll, event, location), pasting and dropping
images and files with the attachment preview and captions, the image
editor, the emoji, GIF and sticker pickers with search, voice recording
with the recording bar, replies and edits, and drafts kept per chat.

implementations:
  - web/src/features/composer/Composer.tsx:Composer
  - web/src/features/picker/Picker.tsx:Picker

## Acceptance
<!-- id: REQ-WEBUI-008.A1 -->
- Enter sends, Shift+Enter adds a line, as today; the shortcut line is
  shown under the composer.
<!-- id: REQ-WEBUI-008.A2 -->
- A pasted image opens the attachment preview; sending it sends one
  image with its caption.
<!-- id: REQ-WEBUI-008.A3 -->
- A draft typed in one chat is still there after switching chats and
  after restarting.
<!-- id: REQ-WEBUI-008.A4 -->
- Voice recording uses the core's audio capture (not the web view's
  microphone permission) and sends a voice note.
<!-- id: REQ-WEBUI-008.A5 -->
- A file too large to send, or one WhatsApp refuses, shows the same
  error as today and the draft and attachment stay.

<!-- id: REQ-WEBUI-009 -->
## Media MUST play and preview as today, decoded by the core

Images open in the viewer with zoom, pan and gallery; videos, GIFs, round
videos and motion photos play inline and in the viewer; voice notes play
with speed control and the waveform; downloads, automatic download rules
and the download folder work as today. Decoding stays in the core where
the web view cannot play a format on every platform (for example Opus
voice notes on WebKitGTK); the core then hands the page a playable stream
or frames through the media protocol.

implementations:
  - src-tauri/src/protocol.rs:serve
  - web/src/features/media/Viewer.tsx:Viewer
  - web/src/features/media/VoiceNote.tsx:VoiceNote

## Acceptance
<!-- id: REQ-WEBUI-009.A1 -->
- Every sample image, video, GIF, voice note, round video and motion
  photo in the demo archive plays on Linux, Windows and macOS.
<!-- id: REQ-WEBUI-009.A2 -->
- Voice note speed (1×, 1.5×, 2×) keeps pitch, as today.
<!-- id: REQ-WEBUI-009.A3 -->
- A media file is read only from the active account's cache (REQ-WEBUI-003.A2).
<!-- id: REQ-WEBUI-009.A4 -->
- A file that cannot be decoded shows its download card with an error,
  never a blank or broken element.

<!-- id: REQ-WEBUI-010 -->
## Calls MUST keep working, with audio and camera in the core

Incoming and outgoing voice and video calls, the call screen and the
call strip, muting, device pickers, the camera preview and the Calls
history. Audio capture and playback and camera capture stay in the core,
as today; the web view shows video frames the core sends and never asks
for the microphone or camera itself. [needs review] Calls are a work in
progress upstream; this requirement holds them to today's behaviour, not
more.

implementations:
  - web/src/features/calls/CallScreen.tsx:CallScreen
  - web/src/features/calls/CallHistory.tsx:CallHistory

## Acceptance
<!-- id: REQ-WEBUI-010.A1 -->
- An incoming call rings, can be answered and declined, and shows the
  other side's video when it is a video call.
<!-- id: REQ-WEBUI-010.A2 -->
- No web-view permission prompt for the microphone or camera appears.

<!-- id: REQ-WEBUI-011 -->
## Info panels and dialogs MUST offer everything they do today

Contact and group info (media, docs and links tabs, groups in common,
participants, group admin, disappearing messages, block, report,
encryption), message info, new chat and new group, the Forward dialog
with All and Favourites tabs (REQ-EXTRAS-015), labels, chat export,
blocking, polls, locations, broadcast lists, and the keyboard-shortcut
list.

implementations:
  - web/src/features/info/InfoPanel.tsx:InfoPanel
  - web/src/features/dialogs/Forward.tsx:Forward

## Acceptance
<!-- id: REQ-WEBUI-011.A1 -->
- Each dialog of the egui build exists and performs the same actions.
<!-- id: REQ-WEBUI-011.A2 -->
- Dialogs trap focus, close with Escape and return focus to what opened
  them.
<!-- id: REQ-WEBUI-011.A3 -->
- An action the core refuses (for example, admin rights lost) shows the
  core's error and leaves the dialog open.

<!-- id: REQ-WEBUI-012 -->
## Settings and desktop integration MUST work on every platform

Settings (appearance, wallpaper, notifications, privacy, storage and
downloads, proxy, language, app lock, shortcuts, about and updates);
notifications with replies where the platform allows; the tray icon and
its menu; the dock or taskbar badge; start at login; one running
instance; opening `zapfast <number>` and WhatsApp links; the native menu
on macOS; the custom title bar where ZapFast draws one today. Updates use
the Tauri updater against the fork's releases, signed with the fork's
key.

implementations:
  - web/src/features/settings/Settings.tsx:Settings
  - src-tauri/src/desktop.rs:setup

## Acceptance
<!-- id: REQ-WEBUI-012.A1 -->
- Each setting of the egui build exists, keeps its stored value from the
  egui build, and takes effect without a restart where it does today.
<!-- id: REQ-WEBUI-012.A2 -->
- Starting ZapFast a second time brings the first window forward instead
  of opening another.
<!-- id: REQ-WEBUI-012.A3 -->
- An installed beta.5 build updates to the first Tauri release, and the
  Tauri build accepts only updates signed with the fork's key.
<!-- id: REQ-WEBUI-012.A4 -->
- Tray, notifications, badge and autostart work on Linux, Windows and
  macOS, or fall back as they do today where a platform lacks one.

<!-- id: REQ-WEBUI-013 -->
## Znga and the extras MUST move into the web interface

The Znga panel (Today, Drafts, Ask, Setup with the audit log, promises,
people, catch-up, calendar, voice notes, coach, personas), Favourites on
the rail, Insights (all tabs) and Keep (Deleted & edited with Copy,
edit highlights and Explain). Their contracts (specs/znga.md,
specs/znga-advanced.md, specs/extras.md) are unchanged; only how they
are drawn changes.

implementations:
  - web/src/features/znga/Znga.tsx:Znga
  - web/src/features/insights/Insights.tsx:Insights

## Acceptance
<!-- id: REQ-WEBUI-013.A1 -->
- Every acceptance criterion of the three fork specs that names the
  interface passes against the web interface.
<!-- id: REQ-WEBUI-013.A2 -->
- Text Znga or Insights shows from messages or a model is shown as text;
  it can never inject markup or script into the page.

<!-- id: REQ-WEBUI-014 -->
## The interface MUST be accessible and keyboard-first

Every control has an accessible name and role (ARIA where HTML lacks
one), the reading order matches the visual order, and every keyboard
shortcut of the egui build (src/ui/keys.rs) works. Focus is always
visible when moved by keyboard.

implementations:
  - web/src/lib/keys.ts:shortcuts

## Acceptance
<!-- id: REQ-WEBUI-014.A1 -->
- VoiceOver, NVDA and Orca read the chat list, a conversation and the
  composer, including unread counts and message status.
<!-- id: REQ-WEBUI-014.A2 -->
- Each shortcut in the shortcut list works and is listed.
<!-- id: REQ-WEBUI-014.A3 -->
- An automated accessibility check (axe) on each screen of the demo
  reports no serious violations.

<!-- id: REQ-WEBUI-015 -->
## The interface MUST stay translated

The existing gettext catalogs (`assets/i18n/`) translate the web
interface; plural forms and right-to-left languages work as today.

implementations:
  - web/src/lib/i18n.ts:t

## Acceptance
<!-- id: REQ-WEBUI-015.A1 -->
- Switching the language in Settings changes every string without a
  restart.
<!-- id: REQ-WEBUI-015.A2 -->
- Arabic and Hebrew lay the window out right to left.

<!-- id: REQ-WEBUI-016 -->
## The new interface MUST stay fast and light

ZapFast's reason to exist is speed, so the move carries budgets.
[needs review] The numbers below are measured on a mid-range laptop with
an account of 5,000 chats and 500,000 archived messages, in a release
build.

implementations:
  - web/src/lib/perf.ts:mark

## Acceptance
<!-- id: REQ-WEBUI-016.A1 -->
- From launch to the chat list drawn with real data: under 1.5 s.
<!-- id: REQ-WEBUI-016.A2 -->
- Opening another chat: under 100 ms to its newest messages.
<!-- id: REQ-WEBUI-016.A3 -->
- Memory after an hour of normal use: under 500 MB for the whole app.
<!-- id: REQ-WEBUI-016.A4 -->
- Typing in the composer shows each character within one frame.
<!-- id: REQ-WEBUI-016.A5 -->
- A release is not published while any budget above is missed on the
  reference machine.

<!-- id: REQ-WEBUI-017 -->
## Packaging and releases MUST cover the same platforms as today

Linux (x86_64 and aarch64 `.tar.gz`, Flatpak), Windows (x64 and arm64
MSI and zip), macOS (universal DMG), built by the Release workflow from a
`v*` tag on `main`, with `checksums.txt` signed by the fork's key as
today. Linux packages declare the WebKitGTK dependency.

implementations:
  - .github/workflows/release.yml:jobs
  - src-tauri/tauri.conf.json:bundle

## Acceptance
<!-- id: REQ-WEBUI-017.A1 -->
- The Release workflow attaches the same set of files as for
  v0.21.1-beta.5.
<!-- id: REQ-WEBUI-017.A2 -->
- Each package starts and links an account on a clean machine of its
  platform.
<!-- id: REQ-WEBUI-017.A3 -->
- A tag not on `main` builds nothing, as today.

<!-- id: REQ-WEBUI-018 -->
## A demo mode and tests MUST cover the new interface

The offline demo (sample chats, every message kind, no connection) runs
in the Tauri app (`--demo`, `--demo-page`), so screens can be checked and
screenshotted without an account. Front-end tests (Vitest with Solid
Testing Library) cover components; end-to-end tests drive the demo build
(WebdriverIO with tauri-driver) through the main flows. CI runs the core
tests, front-end tests and end-to-end tests on every push.

implementations:
  - src/state/demo.rs:populate
  - web/vitest.config.ts:default

## Acceptance
<!-- id: REQ-WEBUI-018.A1 -->
- `--demo` opens the sample chats with no network access.
<!-- id: REQ-WEBUI-018.A2 -->
- End-to-end tests send a message, reply, react, forward to a favourite,
  open media and change the theme in the demo, on Linux in CI.
<!-- id: REQ-WEBUI-018.A3 -->
- The core's existing tests that do not draw the interface still pass.
