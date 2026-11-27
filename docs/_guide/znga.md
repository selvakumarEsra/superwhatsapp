---
title: Znga, the AI agent
description: Znga is SuperWhatsapp's proactive AI agent. It spots the messages that need you, briefs you on what you missed, drafts replies in your voice and keeps your promises. You approve every message.
nav_order: 1
---

Znga is the AI agent built into SuperWhatsapp. It is **proactive**: instead
of waiting for a question, it watches new messages as they arrive and brings
you what matters. It is also **careful**: it never sends a message to anyone
but you without your click.

Open it with the ✨ button on the left of the window. It has five tabs:
**Today**, **Drafts**, **Ask**, **Memory** and **Setup**.

<img class="VPImage" src="{{ '/screenshot.png' | relative_url }}" alt="Znga's Today tab: Ada Lovelace's message 'Can you call me before 6? It's about Saturday' under Needs you, marked as a question, time-sensitive and written repeatedly, with Open chat, Draft reply and Done; below it a briefing of what happened while you were away" width="1800" height="1157">

## Needs you

Every new message is scored on your computer, with no AI model and nothing
sent anywhere. Znga looks for the signals that someone is waiting on you:

- a direct question,
- urgency or a time ("before 6", "tonight", "by Friday"),
- plans being made,
- the same person writing again and again,
- your name mentioned in a group.

Messages that cross the line go to **Needs you** on the Today tab, each with
the reasons it was picked, and can raise a notification titled "Znga". From
there, **Open chat**, **Draft reply** or mark it **Done**.

## Briefings

Press **Brief me** for a short summary of every chat since your last
briefing: **what happened** and **what needs you**, as a numbered to-do list.
Znga also writes one on its own when you come back after being away.

If you want briefings on your phone too, Znga can copy them to your own
"Message yourself" chat. That option is off by default and capped at six a
day.

## Drafts

Znga learns how you write (how long your messages are, how you start and
punctuate them, how often you use emoji and how you laugh) and drafts in your
voice. The style profile it keeps holds those statistics and your favourite
emoji, never your messages.

<img class="VPImage" src="{{ '/assets/images/product/znga-drafts.png' | relative_url }}" alt="The Drafts tab: Suggest reply, Catch me up and Find plans for the open chat; tone buttons; and two 'running about 20 min late' updates, to Grace Hopper and to Rust Berlin, each with Send, Edit in chat and Discard" width="1800" height="1157">

- **Suggest reply** for the open chat, and **Catch me up** on a long one.
- **Friendlier, Shorter, More formal, Fix spelling** and **Translate** for
  the message you are writing.
- **Twin replies** for the chats you mark as your close ones.
- **Check-ins** with people you are close to who have gone quiet.
- **Update your people:** type "tell Mom and Alex I'm 20 minutes late" and
  get one personal draft for each of them.
- **Find plans** turns a plan in the chat into a calendar file.

Each draft waits with **Send**, **Edit in chat** and **Discard**. Znga does
not send on its own: WhatsApp bans accounts for automated messaging, and the
people you talk to should know a message from you came from you.

## Ask

Ask questions about your chats in plain words: "Do I owe anyone a reply?",
"When did Sam say the flight lands?", "What did the school group decide about
Friday?". Answers come from the messages on your computer.

<img class="VPImage" src="{{ '/assets/images/product/znga-ask.png' | relative_url }}" alt="The Ask tab answering a question about recent chats" width="1800" height="1157">

## Memory

Znga keeps an encrypted memory on your computer:

- **Promises.** "I'll send it tomorrow" becomes a promise with a due time,
  and Znga reminds you before it is due.
- **People.** Facts learned from your close ones' chats (a birthday, a new
  job, a child's name) are there when you write to them next.
- **Your Ask thread**, across restarts.

The Memory tab shows what Znga remembers about people. Add a fact yourself,
or forget any of them with one click.

## Also from Znga

- Summaries of long voice notes.
- An opener for a friendship that has drifted.
- A separate persona for each linked number, so work and personal accounts
  each sound like you in the right way.

## Set up Znga

Znga is off until you turn it on. Open ✨, go to **Setup**, choose an engine,
and decide which chats Znga may read.

| Engine | What it needs | Does your text leave your computer? |
|---|---|---|
| Claude Code on this computer | Your Claude Code login. SuperWhatsapp runs `claude -p` with tools and history off, and never reads its token. | Yes, to Anthropic |
| Local model (Ollama) | `ollama pull llama3.2` | No |
| Claude API | An Anthropic API key, in `ANTHROPIC_API_KEY` or the system keychain | Yes, to Anthropic |
| Gemini | A Google AI key, in `GEMINI_API_KEY` or the keychain | Yes, to Google |
| OpenAI-compatible | `OPENAI_API_KEY`, or the address of a local server such as LM Studio or llama.cpp | Only if the server is remote |

In Setup you can also:

- mark chats as **Twin** (your close ones, for voice drafts and check-ins),
- **exclude** chats Znga must never read,
- turn proactive alerts and automatic briefings on or off.

## Privacy and control

- **Opt-in.** Znga does nothing until you turn it on.
- **Consent first.** A cloud engine asks for your consent before it reads
  anything.
- **Never reads locked or excluded chats.**
- **Message text stays in memory.** Znga's memory file is encrypted, and its
  style profile holds statistics, never your messages.
- **Every action is logged.** Each thing Znga does passes a guard that checks
  it against your settings and records it. The Memory tab's Sentinel log
  lists the last 100: everything Znga did or was stopped from doing.
- **You send.** No message to anyone else leaves without your click.

More in [Privacy and security](/privacy/).
