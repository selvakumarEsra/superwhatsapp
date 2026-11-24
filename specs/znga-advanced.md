---
id: ZNGA-ADV-DOC
title: Znga — advanced companion features
owner: selvakumarEsra
priority: high
version: 1
---

<!-- id: ZNGA-ADV-DOC -->
# Znga — advanced companion features

Builds on `specs/znga.md`. Positioning against cloud agents such as Meta's
Muse: Znga is the agent that knows the person's relationships, keeps what
it learns on this computer, and never speaks for the person without a
click. Every feature here keeps the product stance of ZNGA-DOC: drafts
only toward other people, local-first, opt-in, never locked or excluded
chats, and everything under `src/znga/` behind the `znga` feature.

Znga's own memory lives in `znga.db`, an encrypted SQLite file next to
each account's archive, unlocked with the same OS-keyring key as the
archive. Nothing Znga learns is written in plain text.

Non-goals: Znga does not send messages to other people by itself, does
not schedule messages, does not upload memory or the archive anywhere,
and does not add events to a calendar without the person's click.

<!-- id: REQ-ZNGA-015 -->
## Znga's answers MUST stream, run on separate queues, and be stoppable

Answers appear as they are written instead of after the whole answer
arrives. Interactive requests (Ask, the person's own chat, rewrites,
explanations, catch-ups, calendar extraction, voice notes) run on their
own queue, so a background briefing or extraction never makes them wait.
A running interactive answer can be stopped.

implementations:
  - src/znga/runner.rs:Runner
  - src/znga/provider.rs:complete_streaming
  - src/znga/provider.rs:parse_stream_line

verifies:
  - src/znga/runner.rs:partial_text_accumulates_and_a_stop_keeps_it
  - src/znga/runner.rs:lanes_are_counted_apart
  - src/znga/provider.rs:streaming_requests_ask_each_engine_to_stream
  - src/znga/provider.rs:stream_lines_add_up_to_the_answer
  - src/znga/provider.rs:stream_errors_and_refusals_are_reported

## Acceptance
<!-- id: REQ-ZNGA-015.A1 -->
- For the local, OpenAI-compatible, Claude and Gemini engines, partial
  text is reported while the answer is written, and the final text equals
  the concatenation of the parts.
<!-- id: REQ-ZNGA-015.A2 -->
- An interactive request submitted while a background request is running
  starts without waiting for it.
<!-- id: REQ-ZNGA-015.A3 -->
- Stopping a request ends it with "Stopped." and keeps the text written so
  far in the Ask thread.
<!-- id: REQ-ZNGA-015.A4 -->
- The Claude Code engine still answers (whole answer at once).

<!-- id: REQ-ZNGA-016 -->
## Znga's memory MUST survive restarts, encrypted, and start from the archive

On start, Znga's activity is seeded from the archive's recent messages in
visible chats instead of starting empty. The Ask thread, people facts,
promises and the audit log are kept in `znga.db` (encrypted).

implementations:
  - src/znga/store.rs:Store
  - src/znga/store.rs:Db
  - src/znga/hooks.rs:tick
  - src/znga/store.rs:Db.load

verifies:
  - src/znga/store.rs:the_ask_thread_survives_a_reload
  - src/znga/store.rs:facts_promises_and_audit_round_trip
  - src/znga/store.rs:seeding_reads_only_visible_chats
  - src/znga/proactive.rs:seeded_history_comes_before_live_messages_and_is_not_briefed

## Acceptance
<!-- id: REQ-ZNGA-016.A1 -->
- After a restart, the Ask thread shows the earlier turns.
<!-- id: REQ-ZNGA-016.A2 -->
- After a restart, a briefing request can use messages that arrived before
  Znga started, from chats Znga may see.
<!-- id: REQ-ZNGA-016.A3 -->
- `znga.db` is unreadable without the archive key (it is not plain
  SQLite).
<!-- id: REQ-ZNGA-016.A4 -->
- Locked and excluded chats are never read into Znga's memory.

<!-- id: REQ-ZNGA-017 -->
## Every Znga action MUST pass the Sentinel, which logs it

The Sentinel is one code-level gate that every Znga action passes
through: notifications, notes to the person's own chat, files it writes,
archive searches and drafts. Text written by other people can inform an
answer but can never cause an action. Sending is allowed only to the
person's own chat. Every decision is logged and the log is readable in
Setup. Other people's text is fenced so it cannot close or fake the
`<messages>` boundary.

implementations:
  - src/znga/sentinel.rs:check
  - src/znga/sentinel.rs:fence
  - src/znga/sentinel.rs:Audit
  - src/znga/hooks.rs:apply

verifies:
  - src/znga/sentinel.rs:znga_never_sends_to_another_chat
  - src/znga/sentinel.rs:others_text_cannot_use_tools
  - src/znga/sentinel.rs:every_decision_is_logged_and_the_log_is_capped
  - src/znga/sentinel.rs:fenced_text_cannot_close_the_messages_block

## Acceptance
<!-- id: REQ-ZNGA-017.A1 -->
- A send to any chat other than the person's own chat is refused and
  logged as refused.
<!-- id: REQ-ZNGA-017.A2 -->
- A tool action (archive search, writing a file) whose origin is other
  people's text is refused.
<!-- id: REQ-ZNGA-017.A3 -->
- A message containing `</messages>` or `<messages>` is fenced so the
  prompt still has exactly one opening and one closing tag around the
  data.
<!-- id: REQ-ZNGA-017.A4 -->
- Every allowed and refused action appears in the audit log with its time,
  kind, origin and verdict; the log keeps the last 500 entries.

<!-- id: REQ-ZNGA-018 -->
## Znga SHOULD track promises made in chats and remind the person

Znga picks out commitments the person made ("I'll send the photos
Friday") and requests made to them, from twin chats and chats the person
asked about. Each promise has who owes it, what, the chat, and a due time
when one was said. Before a promise is due, Znga notifies the person and
offers a draft. Promises can be marked done or dismissed.

implementations:
  - src/znga/promises.rs:parse
  - src/znga/promises.rs:Promises
  - src/znga/promises.rs:Promises.due
  - src/znga/prompt.rs:promises
  - src/znga/hooks.rs:check_promises

verifies:
  - src/znga/promises.rs:promises_are_parsed_with_who_and_when
  - src/znga/promises.rs:the_same_promise_is_kept_once
  - src/znga/promises.rs:a_due_promise_reminds_once_and_never_after_done

## Acceptance
<!-- id: REQ-ZNGA-018.A1 -->
- An extraction answer in the agreed JSON shape yields promises with who,
  text, and a due time when one was given.
<!-- id: REQ-ZNGA-018.A2 -->
- The same promise extracted twice is kept once.
<!-- id: REQ-ZNGA-018.A3 -->
- A promise due within the next 3 hours notifies once, never twice.
<!-- id: REQ-ZNGA-018.A4 -->
- A promise marked done or dismissed never notifies again.

<!-- id: REQ-ZNGA-019 -->
## Znga SHOULD remember facts about people, on this computer only

Per contact, Znga keeps short facts (birthdays, family names, allergies,
what they are going through) extracted from chats Znga may see. The
person can see, add and delete them. Facts feed the twin's drafts,
check-ins and briefings.

implementations:
  - src/znga/memory.rs:parse_facts
  - src/znga/memory.rs:People
  - src/znga/prompt.rs:facts
  - src/znga/prompt.rs:twin_reply
  - src/znga/hooks.rs:twin_for

verifies:
  - src/znga/memory.rs:facts_are_parsed_from_the_answer
  - src/znga/memory.rs:a_fact_is_stored_once_and_a_forgotten_one_stays_forgotten
  - src/znga/memory.rs:each_contact_keeps_its_newest_facts
  - src/znga/prompt.rs:the_twin_writes_as_the_person_with_their_style

## Acceptance
<!-- id: REQ-ZNGA-019.A1 -->
- Extracted facts are stored per account and contact, and the same fact
  is not stored twice.
<!-- id: REQ-ZNGA-019.A2 -->
- A fact the person deleted is not added back by a later extraction.
<!-- id: REQ-ZNGA-019.A3 -->
- A twin draft for a contact includes that contact's facts in its prompt.
<!-- id: REQ-ZNGA-019.A4 -->
- Facts are kept only in `znga.db`; none are kept for locked or excluded
  chats.

<!-- id: REQ-ZNGA-020 -->
## Ask and the person's own chat MUST be able to search the whole archive

Ask Znga (and the self-chat of REQ-ZNGA-012..014, built here) answers from
the whole local archive, not only recent activity: Znga first decides what
to search for, the archive is searched (visible chats only), and the
answer cites what it found. The search is a read-only tool that passes the
Sentinel with the person as its origin.

implementations:
  - src/znga/archive_ask.rs:parse_plan
  - src/znga/archive_ask.rs:search
  - src/znga/prompt.rs:plan_search
  - src/znga/prompt.rs:answer_with_hits
  - src/znga/selfchat.rs:SelfChat
  - src/znga/hooks.rs:observe_messages
  - src/znga/hooks.rs:search_for

verifies:
  - src/znga/archive_ask.rs:plans_are_parsed_and_tidied
  - src/znga/archive_ask.rs:hits_come_only_from_visible_chats
  - src/znga/prompt.rs:answers_with_hits_cite_the_archive

## Acceptance
<!-- id: REQ-ZNGA-020.A1 -->
- A question about something older than Znga's recent activity is
  answered from archive hits, each cited with chat and date.
<!-- id: REQ-ZNGA-020.A2 -->
- Hits from locked or excluded chats are never included.
<!-- id: REQ-ZNGA-020.A3 -->
- With the self-chat switch on, a text the person sends to their own chat
  is answered there with the archive search available (REQ-ZNGA-012 to
  014 apply).

<!-- id: REQ-ZNGA-021 -->
## Znga SHOULD catch the person up on one chat

For the open chat: what changed since the person last read it, what
involves them (mentions, questions to them), and a suggested reply as a
draft.

implementations:
  - src/znga/prompt.rs:catch_up
  - src/znga/hooks.rs:request_catch_up

verifies:
  - src/znga/prompt.rs:a_catch_up_splits_into_summary_and_reply

## Acceptance
<!-- id: REQ-ZNGA-021.A1 -->
- The catch-up covers the unread messages of the open chat, and at least
  the last 20 messages when fewer are unread.
<!-- id: REQ-ZNGA-021.A2 -->
- The suggested reply arrives as a draft, never sent.
<!-- id: REQ-ZNGA-021.A3 -->
- With no open chat, the action is disabled and says why.

<!-- id: REQ-ZNGA-022 -->
## Znga SHOULD turn plans in a chat into calendar files

From the open chat, Znga finds events (what, when, where) and offers an
"Add to calendar" button per event that writes a standard `.ics` file and
opens it with the system calendar. No calendar account is used.

implementations:
  - src/znga/calendar.rs:parse_events
  - src/znga/calendar.rs:ics
  - src/znga/prompt.rs:events
  - src/znga/hooks.rs:add_to_calendar

verifies:
  - src/znga/calendar.rs:an_event_becomes_a_valid_calendar_file
  - src/znga/calendar.rs:special_characters_are_escaped_and_long_lines_folded
  - src/znga/calendar.rs:events_without_a_title_or_time_are_skipped

## Acceptance
<!-- id: REQ-ZNGA-022.A1 -->
- An event with a title, start and place produces a valid iCalendar file
  with `BEGIN:VCALENDAR`, one `VEVENT`, `DTSTART`, `SUMMARY` and
  `LOCATION`.
<!-- id: REQ-ZNGA-022.A2 -->
- Commas, semicolons and line breaks in event text are escaped.
<!-- id: REQ-ZNGA-022.A3 -->
- An event without an end lasts one hour.
<!-- id: REQ-ZNGA-022.A4 -->
- Writing the file passes the Sentinel with the person as origin.

<!-- id: REQ-ZNGA-023 -->
## Znga SHOULD summarise long voice notes

A voice note that arrived in a chat Znga may see can be transcribed and
summarised with a suggested reply. Transcription uses a configurable
OpenAI-compatible transcription endpoint; the default is a local server
on this computer, so nothing leaves it unless the person points it at a
cloud service.

implementations:
  - src/znga/voice.rs:multipart
  - src/znga/voice.rs:transcribe
  - src/znga/prompt.rs:voice_summary
  - src/znga/hooks.rs:request_voice

verifies:
  - src/znga/voice.rs:the_upload_is_multipart_with_the_model_and_the_file
  - src/znga/voice.rs:transcripts_and_errors_are_read

## Acceptance
<!-- id: REQ-ZNGA-023.A1 -->
- The transcription request is a multipart upload with the audio file and
  the model name.
<!-- id: REQ-ZNGA-023.A2 -->
- A voice note that is not downloaded yet says so instead of failing
  silently.
<!-- id: REQ-ZNGA-023.A3 -->
- The summary shows the transcript, a short summary and a suggested reply
  draft.
<!-- id: REQ-ZNGA-023.A4 -->
- A cloud transcription endpoint requires the same cloud consent as a
  cloud engine.

<!-- id: REQ-ZNGA-024 -->
## Znga SHOULD coach the person on relationships that are drifting

For twin chats, Znga shows how the relationship's rhythm changed (who
wrote last, usual gap versus the current one, who usually starts) and
offers an opener in the person's voice.

implementations:
  - src/znga/coach.rs:assess
  - src/znga/coach.rs:Rhythm
  - src/znga/hooks.rs:request_opener

verifies:
  - src/znga/coach.rs:a_long_silence_against_a_weekly_rhythm_is_drifting
  - src/znga/coach.rs:a_short_or_normal_silence_is_not_drifting

## Acceptance
<!-- id: REQ-ZNGA-024.A1 -->
- A chat whose current silence is more than twice its usual gap, and at
  least 7 days, is reported as drifting.
<!-- id: REQ-ZNGA-024.A2 -->
- The reason names who wrote last and the usual versus current gap.
<!-- id: REQ-ZNGA-024.A3 -->
- The opener arrives as a draft, never sent.

<!-- id: REQ-ZNGA-025 -->
## Znga MAY use a separate persona per linked account

Each linked account can have its own name, voice notes, tone, and whether
proactive briefings run, so a work account and a personal one sound and
behave differently. Memory is already kept per account.

implementations:
  - src/znga/config.rs:Persona
  - src/znga/config.rs:Config.persona

verifies:
  - src/znga/config.rs:a_persona_overrides_the_global_voice_per_account

## Acceptance
<!-- id: REQ-ZNGA-025.A1 -->
- A draft for an account with a persona uses that persona's name, voice
  notes and tone.
<!-- id: REQ-ZNGA-025.A2 -->
- An account without a persona uses the global settings.
<!-- id: REQ-ZNGA-025.A3 -->
- Briefings switched off in an account's persona never run for that
  account.

<!-- id: REQ-ZNGA-026 -->
## Znga MAY explain what an edit changed

In Insights → Deleted & edited, an edited entry has an "Explain" button
(when Znga is on) that says in one or two sentences what the edit changed
in meaning ("the meeting moved from 6 to 7").

implementations:
  - src/znga/prompt.rs:explain_edit
  - src/znga/hooks.rs:request_explain_edit

verifies:
  - src/znga/prompt.rs:new_prompts_fence_data_and_ask_for_their_shapes

## Acceptance
<!-- id: REQ-ZNGA-026.A1 -->
- The explanation prompt contains both versions and asks for at most two
  sentences.
<!-- id: REQ-ZNGA-026.A2 -->
- With Znga off, the Explain button is not shown.
<!-- id: REQ-ZNGA-026.A3 -->
- The explanation shows under its entry and is not saved.
