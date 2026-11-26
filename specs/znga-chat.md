---
id: ZCHAT-DOC
title: Znga as a chat member and a proactive agent
owner: selvakumarEsra
priority: high
---

<!-- id: ZCHAT-DOC -->
# Znga as a chat member and a proactive agent

Znga stops being a separate window. It becomes **a member of the chat
list**: a chat named "Znga" where everything between the person and the AI
happens as a conversation, and where Znga speaks first when something needs
the person. Its configuration moves into Settings.

This is built in the Tauri interface only (specs/tauri-ui.md). The egui
Znga panel (`src/znga/ui.rs`) is not changed; it is deleted with the rest
of egui (REQ-WEBUI-001). The Znga logic that is not drawing (engines,
runner, triage, briefings, drafts, memory, promises, Sentinel) stays in the
core and is exposed to the web interface through typed commands and events
(REQ-WEBUI-002).

What moves where, from the old panel's tabs:

| Old tab | New home |
|---|---|
| Today (Needs you, briefings, catch-ups, plans, voice notes, promises, Stay close) | Messages Znga posts in its chat |
| Drafts (twin replies, check-ins, Update your people) | Draft messages in the Znga chat, each with Send, Edit in chat and Discard |
| Drafts (Suggest reply, Catch me up, Find plans, rewrite, translate for the open chat) | A Znga menu in every chat's composer |
| Ask | Writing to Znga in its chat |
| Memory and Setup | Settings › Znga |

House rules this keeps (specs/znga.md, specs/znga-advanced.md, AGENTS.md):
Znga is opt-in; it never reads locked or excluded chats; **it never sends a
message to anyone but the person without the person's click**; every action
passes the Sentinel and is logged; message text never reaches the log; no
WhatsApp logos or artwork (Znga's avatar is SuperWhatsapp's own sparkle).

Non-goals: Znga sending on its own; a Znga chat that syncs to WhatsApp's
servers or to the phone (the existing opt-in copy to "Message yourself",
REQ-ZNGA-006, stays as it is); answering in "Message yourself" changes
nothing (REQ-ZNGA-012 to 014 stay); Znga in the egui interface.

<!-- id: REQ-ZCHAT-001 -->
## Znga MUST appear in the chat list as a local chat

When Znga is turned on, each linked account's chat list shows a chat named
**Znga**, with SuperWhatsapp's sparkle as its avatar, kept above the other
chats. It exists only in SuperWhatsapp: its messages are stored in Znga's
encrypted memory on this computer, per account, and are never sent to
WhatsApp. It shows an unread count for messages Znga posted since the
person last opened it. Turning Znga off hides the chat; turning it on again
shows its history. The ✨ button on the left rail opens this chat.

implementations:
  - src/znga/thread.rs:Thread
  - src/znga/store.rs:Store
  - src-tauri/src/znga.rs:row
  - web/src/features/chats/ChatList.tsx:ChatList

verifies:
  - src/znga/store.rs:the_znga_chat_file_holds_no_readable_text
  - src/znga/thread.rs:unasked_messages_fold_within_ten_minutes
  - src/znga/chat.rs:nothing_is_posted_with_znga_off

## Acceptance
<!-- id: REQ-ZCHAT-001.A1 -->
- With Znga on, the chat list's first row is "Znga" with the sparkle avatar,
  above pinned chats, on every linked account.
<!-- id: REQ-ZCHAT-001.A2 -->
- With Znga off, no Znga row is shown and no Znga message is posted.
<!-- id: REQ-ZCHAT-001.A3 -->
- No Znga chat message is ever handed to the WhatsApp worker: a test that
  posts, answers and restarts finds no outgoing WhatsApp message.
<!-- id: REQ-ZCHAT-001.A4 -->
- After a restart the Znga chat shows the same messages, read from the
  encrypted store; the store file holds no readable message text.
<!-- id: REQ-ZCHAT-001.A5 -->
- Messages Znga posts while the chat is closed raise its unread count;
  opening the chat clears it.
<!-- id: REQ-ZCHAT-001.A6 -->
- Clicking ✨ on the left rail opens the Znga chat; no separate Znga window
  exists in the Tauri interface.

<!-- id: REQ-ZCHAT-002 -->
## Every request to the AI and its answer MUST appear in the Znga chat

What the person writes in the Znga chat is a question or instruction to
Znga, and Znga's answer appears below it as a message, streaming as the
engine writes it and stoppable (REQ-ZNGA-009 Ask, REQ-ZNGA-015 streaming).
Work started elsewhere (a composer tool in another chat, a briefing, a
voice-note summary) also appears in the Znga chat, as a short line saying
what was asked and for which chat, followed by the result. The existing Ask
history carries over as the start of the Znga chat.

implementations:
  - src/znga/thread.rs:Thread.post
  - src/znga/chat.rs:send_message
  - web/src/features/znga/ZngaChat.tsx:ZngaChat

verifies:
  - src/znga/chat.rs:questions_are_answered_in_the_chat_and_errors_say_what_to_do
  - src/znga/chat.rs:suggest_reply_goes_to_the_chat_and_the_composer
  - src/znga/thread.rs:restored_messages_come_before_newer_ones

## Acceptance
<!-- id: REQ-ZCHAT-002.A1 -->
- Writing "Do I owe anyone a reply?" in the Znga chat shows it as the
  person's message, then Znga's answer as it streams; Stop ends it and keeps
  what was written.
<!-- id: REQ-ZCHAT-002.A2 -->
- Using Suggest reply in a chat with Ada adds "Suggest a reply to Ada" and
  the draft to the Znga chat, and the same draft to the composer of the chat
  with Ada.
<!-- id: REQ-ZCHAT-002.A3 -->
- An engine error appears in the Znga chat as Znga's message saying what
  failed and what to do (for example, start Ollama), not as a silent
  failure.
<!-- id: REQ-ZCHAT-002.A4 -->
- After upgrading, the person's earlier Ask questions and answers are the
  first messages of the Znga chat, in order.

<!-- id: REQ-ZCHAT-003 -->
## What Znga finds MUST arrive as messages the person can act on

Everything the old Today and Drafts tabs listed arrives as a message in the
Znga chat, with its actions as buttons on the message:

| Message | Actions |
|---|---|
| Needs you: who, the message, why it was picked | Open chat, Draft reply, Done |
| Briefing: what happened, what needs you | Copy |
| Draft (twin reply, check-in, update for one person) | Send, Edit in chat, Discard |
| Catch-up of one chat | Open chat |
| Plan found in a chat | Add to calendar, Dismiss |
| Long voice note | Summarise (then the summary and transcript) |
| Promise due | Done, Not a promise |
| A close friend drifting away | Draft an opener |

A message whose action is taken shows the outcome ("Sent", "Added to
calendar") and its buttons are disabled.

implementations:
  - src/znga/thread.rs:Card
  - src/znga/chat.rs:card_action
  - src-tauri/src/znga.rs:chat_view
  - web/src/features/znga/ZngaCard.tsx:ZngaCard

verifies:
  - src/znga/chat.rs:tell_makes_a_draft_and_sends_only_on_the_click

## Acceptance
<!-- id: REQ-ZCHAT-003.A1 -->
- Each row of the table appears as a message with exactly its listed
  actions, and each action does what the old panel's button did.
<!-- id: REQ-ZCHAT-003.A2 -->
- Send on a draft sends it to that chat only after the click, and the Znga
  message then reads "Sent"; pressing Send again does nothing.
<!-- id: REQ-ZCHAT-003.A3 -->
- Edit in chat opens the person's chat with the draft in its composer and
  sends nothing.
<!-- id: REQ-ZCHAT-003.A4 -->
- An action on a chat of another linked account says to switch accounts,
  as the panel did, instead of acting on the wrong account.

<!-- id: REQ-ZCHAT-004 -->
## Every chat's composer MUST offer Znga's tools for that chat

The composer of every chat (not the Znga chat) has a Znga button opening a
menu: Suggest reply, Catch me up, Find plans, Friendlier, Shorter, More
formal, Fix spelling and Translate to a chosen language. Rewrites replace
the composer's text only after the person accepts them; Undo puts the
original back.

implementations:
  - web/src/features/composer/Composer.tsx:Composer
  - src/znga/chat.rs:tool

verifies:
  - src/znga/chat.rs:rewrites_are_offered_and_need_text

## Acceptance
<!-- id: REQ-ZCHAT-004.A1 -->
- With Znga on, every chat's composer shows the Znga button; with Znga off,
  or in a locked or excluded chat, it does not.
<!-- id: REQ-ZCHAT-004.A2 -->
- Rewrite and Translate are disabled while the composer is empty.
<!-- id: REQ-ZCHAT-004.A3 -->
- Accepting a rewrite replaces the composer text; Undo restores the
  original; nothing is sent by either.

<!-- id: REQ-ZCHAT-005 -->
## Znga MUST speak first when something needs the person

Znga posts in its chat on its own, without being asked, for:

- **Needs you**: a new message that scores as needing the person
  (REQ-ZNGA-004).
- **Welcome back**: a briefing when the person returns after being away
  (REQ-ZNGA-005).
- **Digests**: a morning and an evening digest of the day, at times the
  person sets. Off by default. [needs review: default times 08:00 and
  20:00]
- **Reminders**: a promise coming due, a plan found in a chat, a close
  friend drifting away.

Each kind can be switched off in Settings › Znga. Znga stays quiet during
the person's quiet hours, except for Needs you, and never posts more than
one unasked message per 10 minutes, folding what arrives meanwhile into
one message. [needs review: the 10-minute fold and the quiet-hours
exception] A proactive message raises a notification titled "Znga" only
when the app is not focused on the Znga chat and notifications are on.

implementations:
  - src/znga/thread.rs:Thread
  - src/znga/chat.rs:unasked

verifies:
  - src/znga/thread.rs:unasked_messages_fold_within_ten_minutes
  - src/znga/thread.rs:quiet_hours_hold_everything_but_needs_you
  - src/znga/thread.rs:a_digest_runs_once_a_day_at_the_first_chance
  - src/znga/thread.rs:a_kind_switched_off_while_held_is_dropped
  - src/znga/chat.rs:needs_you_from_a_hidden_chat_is_never_posted

## Acceptance
<!-- id: REQ-ZCHAT-005.A1 -->
- A message scoring as Needs you produces one Znga message within 5 seconds
  of arriving, with no AI call made to decide it.
<!-- id: REQ-ZCHAT-005.A2 -->
- Three qualifying events within 10 minutes produce one Znga message listing
  all three, not three messages.
<!-- id: REQ-ZCHAT-005.A3 -->
- With digests on at 08:00, one digest is posted at the first chance after
  08:00, and not again that day, across restarts.
<!-- id: REQ-ZCHAT-005.A4 -->
- During quiet hours only Needs you messages are posted; the rest wait for
  quiet hours to end.
<!-- id: REQ-ZCHAT-005.A5 -->
- Switching a kind off in Settings stops that kind at once; nothing of
  that kind is posted afterwards.
<!-- id: REQ-ZCHAT-005.A6 -->
- Nothing from a locked or excluded chat is ever named or quoted in a
  proactive message.

<!-- id: REQ-ZCHAT-006 -->
## Znga MUST act on what the person replies, and MUST wait for the click

The person can answer Znga in plain words, and Znga acts by preparing, not
sending: "tell Ada I'll call at 5" gives a draft to Ada; "remind me to pay
Sam on Friday" adds a promise; "summarise the school group" posts a
catch-up; "done" on a Needs you message marks it done. Anything that would
send a message to someone other than the person arrives as a draft with
Send, Edit in chat and Discard (REQ-ZCHAT-003). When Znga cannot tell who
or what the person means, it asks instead of guessing. Every action passes
the Sentinel and is recorded in its log.

implementations:
  - src/znga/chat.rs:send_message
  - src/znga/sentinel.rs:check
  - src/znga/prompt.rs:tell

verifies:
  - src/znga/chat.rs:tell_makes_a_draft_and_sends_only_on_the_click
  - src/znga/chat.rs:an_ambiguous_name_is_asked_about
  - src/znga/chat.rs:locked_and_excluded_chats_are_refused_and_logged
  - src/znga/chat.rs:a_hundred_instructions_send_nothing

## Acceptance
<!-- id: REQ-ZCHAT-006.A1 -->
- "Tell Ada I'll call at 5" produces a draft to Ada and sends nothing until
  Send is clicked.
<!-- id: REQ-ZCHAT-006.A2 -->
- "Tell Sam I'm late" with two contacts called Sam makes Znga ask which
  one, listing both, and prepare nothing until the person answers.
<!-- id: REQ-ZCHAT-006.A3 -->
- An instruction naming a locked or excluded chat is refused with a short
  reason, and the refusal appears in the Sentinel log.
<!-- id: REQ-ZCHAT-006.A4 -->
- No code path from a Znga reply reaches the WhatsApp send command without
  a click on Send; a test drives 100 instructions and counts zero sends.

<!-- id: REQ-ZCHAT-007 -->
## Settings MUST hold Znga's setup and memory

Settings gets a **Znga** section with: turning Znga on or off; the engine
and its key or address (REQ-ZNGA-002); which chats are Twin and which are
excluded; the proactive kinds, digest times and quiet hours
(REQ-ZCHAT-005); copying briefings to "Message yourself"; what Znga
remembers about people, with add and forget; open promises; and the
Sentinel log of the last 100 actions.

implementations:
  - web/src/features/settings/ZngaSettings.tsx:ZngaSettings
  - src-tauri/src/znga.rs:settings

## Acceptance
<!-- id: REQ-ZCHAT-007.A1 -->
- Every setting the old Setup tab had can be changed in Settings › Znga and
  takes effect without a restart.
<!-- id: REQ-ZCHAT-007.A2 -->
- Forgetting a fact removes it from Znga's memory and from later answers.
<!-- id: REQ-ZCHAT-007.A3 -->
- The Sentinel log lists each action with its time, kind and whether it was
  allowed, and never message text.
<!-- id: REQ-ZCHAT-007.A4 -->
- Settings search finds the Znga section by "Znga", "AI" and "engine".
