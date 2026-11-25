---
title: What is SuperWhatsapp?
description: What SuperWhatsapp and its AI agent Znga do for you, what it supports, and its current limitations.
redirect_from:
  - /what-is-fastsapp/
  - /what-is-zapfast/
nav_order: 0
---

## SuperWhatsapp in one minute

SuperWhatsapp is a desktop app for WhatsApp on macOS, Windows and Linux, with
**Znga**, a proactive AI agent, built in. It links to your WhatsApp account as
a companion device, the same way WhatsApp's own desktop app does, so your
phone keeps working as before and your chats appear on your computer.

Where other desktop clients wait for you to read everything, SuperWhatsapp
works ahead of you:

1. **It sorts what matters.** Every new message is scored on your computer.
   Questions, deadlines, plans and people writing again and again go to
   Znga's **Needs you** list and can raise a notification of their own.
2. **It catches you up.** A **briefing** sums up what happened and what needs
   you across all your chats, on demand or when you come back.
3. **It drafts, you decide.** Znga drafts replies in your voice, rewrites a
   message friendlier or shorter, translates it, and turns one instruction
   into a personal message for several people. Nothing is sent until you
   click Send.
4. **It remembers.** Promises you make ("I'll send it tomorrow") come back as
   reminders, and the facts people share are there when you need them.

[Meet Znga](/znga/) to see each of these in detail.

Under the hood SuperWhatsapp is a native app written in Rust, with no browser
engine inside. It is built on [ZapFast](https://github.com/crmne/zapfast) by
Carmine Paolino and connects through
[whatsapp-rust](https://github.com/oxidezap/whatsapp-rust). It opens in under
a second and stays light on memory: [see the measurements](/benchmarks/).

<img class="VPImage dark" src="{{ '/screenshot.png' | relative_url }}" alt="SuperWhatsapp with Znga's Today panel beside a chat" width="1800" height="1157">
<img class="VPImage light" src="{{ '/screenshot-light.png' | relative_url }}" alt="SuperWhatsapp in its light theme, showing a conversation with photos, voice messages and reactions" width="1800" height="1157">

## Also in SuperWhatsapp

- **Favourites.** The heart on the left opens your favourite chats and
  groups, and the Forward dialog has a Favourites tab.
- **Insights.** A dashboard of how you use WhatsApp, worked out on your
  computer: trends, busiest hours, streaks, top chats, reply times and the
  people you have drifted from. Reading what was said (most used words and
  emoji) is optional and off until you turn it on.
- **Keep.** Messages deleted for everyone, and the versions that edits
  replace, stay in your local archive under Insights, and every attachment is
  saved as it arrives.
- **Pin as many chats as you like.** Pins beyond WhatsApp's limit stay on
  this computer, so your phone's pins are never pushed out.

## What it does

<img class="VPImage" src="{{ '/assets/images/product/chat.png' | relative_url }}" alt="A SuperWhatsapp chat with a photo, voice messages, a reaction and a link preview, beside the chat list" width="1800" height="1157">

- **Stores your chats.** SuperWhatsapp links as a companion device and stores
  messages in an encrypted SQLite file per account. History remains after
  restart, and older messages are fetched from your phone as you scroll.
- **Sends common message types.** Send formatted text, replies, edits,
  reactions, forwards, pictures, files, stickers, GIFs, and recorded voice
  messages. You can add captions to attachments and send one photo as a
  WhatsApp view-once message.
- **Makes calls (beta).** Place and answer one-to-one voice and video calls, with
  mute, camera and a choice of microphone, speaker and camera, and see every
  call, group calls included, in its chat and on the Calls page.
  [Calls](/using-superwhatsapp/#calls).
- **Plays media in the chat.** Voice messages, videos, round video messages,
  GIFs, and animated WebP and supported Lottie stickers play in place. H.264 videos in MP4 files play
  after downloading to the local cache; other formats open in your system
  player. [Download limits and playback controls](/using-superwhatsapp/#videos-and-photos).
- **Uses interactive messages.** Business templates show their text, images,
  and options. Reply buttons and simple lists send the selected response with
  a quote, web links open in your browser, and copy-code buttons use the clipboard. [See examples and limitations](/using-superwhatsapp/#interactive-messages).
- **Uses consistent names.** Address-book names take priority over public
  WhatsApp profile names for chats, mentions, replies, and notifications.
- **Runs in the background.** Closing the window keeps SuperWhatsapp in the system
  tray. Notifications can show the chat picture and open the chat at the
  message they announced. Supported desktops show the unread count on the app
  icon in the taskbar or the macOS Dock; on Windows, the count appears while the window
  has a taskbar button. Muting a chat also mutes it on your phone.
- **Keeps several numbers.** Link more than one WhatsApp number and switch
  between them in one window.
- **Keeps chats private on this computer.** The archive is encrypted with a
  key in your OS keyring, locked chats stay hidden behind a local code, and an
  optional app lock hides the whole window behind a password.
- **Copies message text.** Select part of a message or copy across messages
  with the time, date, and sender included.

## What it does not do yet

These are current implementation limits, not permanent product exclusions.
Feature requests can be discussed within the project's
[product boundaries](https://github.com/selvakumarEsra/superwhatsapp/blob/main/CONTRIBUTING.md#before-opening-an-issue).
SuperWhatsapp does not currently support:

- Creating call links and screen sharing; status posts, communities and
  publishing to channels. Group calls work, as an experiment (see the guide).
- Playing videos outside the supported H.264 MP4 format in the app; they open
  in your system player. Attachments over 64 MiB download only when you click
  them, never automatically.
- Interactive forms, payments, shopping flows, carousel selections, or forwarding
  interactive messages. Use these in WhatsApp Web or on your phone. Embedded
  videos, documents, and templates without readable text
  also need another client.

When reporting [an issue](https://github.com/selvakumarEsra/superwhatsapp/issues), include
what happened, what you expected, and when it happened. This helps match the
problem to the log in the state directory.

## Account safety

SuperWhatsapp is an **unofficial** client. Using it may be against WhatsApp's terms
of service. It uses WhatsApp's companion-device protocol, sends normal
receipts, and does not automate or send messages in bulk: Znga only drafts,
and every message to someone else needs your click. WhatsApp may still
restrict accounts that use unofficial clients. Use an official client if you
cannot accept that risk.

## Prior art

SuperWhatsapp connects through
[whatsapp-rust](https://github.com/oxidezap/whatsapp-rust), which grew out
of the [whatsmeow](https://github.com/tulir/whatsmeow) lineage. WhatsApp
Web defines the companion-device model. SuperWhatsapp is built on
[ZapFast](https://github.com/crmne/zapfast) by Carmine Paolino (MIT license),
which uses [fastframe](https://fastframe.dev) and [egui](https://www.egui.rs).

SuperWhatsapp is an independent project, not affiliated with, endorsed by or
sponsored by WhatsApp LLC or Meta Platforms, Inc. WhatsApp is a trademark of WhatsApp LLC.
