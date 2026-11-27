---
title: Privacy and security
description: What SuperWhatsapp and its AI agent Znga store, where your messages go, and the controls you have.
nav_order: 5
---

SuperWhatsapp is built so that your messages stay yours. This page sets out
what is stored, what leaves your computer, and how to control it.

## Your messages stay on your computer

- **End to end.** SuperWhatsapp links to your account as a companion device,
  like WhatsApp's own desktop app, and uses WhatsApp's end-to-end encryption.
- **Encrypted archive.** Your message history is kept in an encrypted
  database on your computer. Its key lives in your system's keychain (the
  macOS Keychain, the Windows Credential Manager or the Secret Service on
  Linux). [Details](/settings-and-files/#archive-encryption).
- **Locked chats and the app lock.** Hide chats behind a code, or the whole
  window behind a password, on this computer.
- **No accounts, no tracking.** SuperWhatsapp has no account of its own, no
  analytics and no telemetry.

## What reaches the network

- **WhatsApp**, to send and receive your messages, calls and media.
- **GitHub**, once a day, to check for a new version, if **Check for
  updates** is on. Updates are only installed after their signature checks
  out.
- **Link previews and GIF search**, when you write a link or search for a
  GIF.
- **The AI engine you choose for Znga**, and only after you turn Znga on.

Every one of these can go through a proxy, including Tor. See
[Settings and files](/settings-and-files/).

## Znga and your data

Znga is off until you turn it on, and it is built to need as little as
possible:

| | |
|---|---|
| **Needs you** alerts | Scored on your computer, with no AI model. Nothing is sent. |
| **Briefings, drafts, Ask** | Sent to the engine you chose. With a local model (Ollama, LM Studio) nothing leaves your computer. |
| **Cloud engines** | Ask for your consent before reading anything. |
| **Locked and excluded chats** | Never read. |
| **Memory** | Encrypted on your computer. Forget any fact from the Memory tab. |
| **Style profile** | Statistics about how you write and your favourite emoji, never your messages. |
| **Sending** | Znga only drafts. A message to anyone else needs your click. |

Each action Znga takes passes a guard that checks it against your settings
and records it in the Sentinel log on the Memory tab.

## Account safety

SuperWhatsapp is an unofficial client. It uses WhatsApp's companion-device
protocol, sends normal receipts and never sends automated or bulk messages,
but using an unofficial client may be against WhatsApp's terms of service,
and WhatsApp may restrict accounts that use one. Use WhatsApp's own apps if
you cannot accept that risk.

## Open source

SuperWhatsapp is open source under the MIT license, so anyone can check what
it does. The code is on
[GitHub](https://github.com/selvakumarEsra/superwhatsapp). Found a security
problem? Please report it privately through GitHub's
[security advisories](https://github.com/selvakumarEsra/superwhatsapp/security/advisories/new).
