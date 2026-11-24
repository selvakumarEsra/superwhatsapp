---
id: MINIAPP-DOC
title: Mini-apps — apps that live in a chat
owner: selvakumarEsra
priority: high
version: 1
---

<!-- id: MINIAPP-DOC -->
# Mini-apps — apps that live in a chat

WhatsApp has no mini-app surface: the other people in a chat use stock
WhatsApp, so an app can only reach them through what WhatsApp already has
(messages, replies, polls, links). A ZapFast mini-app therefore has two
halves: what it says in the chat it is installed in, and a panel of cards
that only the ZapFast user sees.

This document defines the app host: how an app describes itself, how it is
installed in one chat at a time, which events it receives, how every action
it takes passes the Znga Sentinel (REQ-ZNGA-017), how it draws cards, and
where it keeps its data. Apps are built into ZapFast in this version; an API
on this computer for outside apps such as bookmygame is a later document on
the same model. The first app is **Goal of the day** (REQ-GOAL-001..005),
which uses Znga to propose a daily goal and track it in the person's own
"Message yourself" chat.

Everything lives under `src/miniapps/` behind a `miniapps` Cargo feature,
on by default in the fork, joined to upstream code only by one-line hooks
marked `// miniapps hook`. No widget is added inside an upstream panel: the
host has its own window and rail button, like Znga.

Non-goals: apps from outside ZapFast or loaded at run time (later
document); apps reading every message of a chat; apps sending to a chat they
are not installed in; any app message to another person that the person
did not either click or set up by installing the app there.

<!-- id: REQ-MINIAPP-001 -->
## Every mini-app MUST describe itself with a manifest

The manifest names the app and states, before it is installed, everything
it may do. The host refuses anything an app's manifest does not declare.

The manifest holds: an id, a name, an icon, a one-line description, the
command word it answers to in chats (e.g. `goal` for `/goal`), the kinds of
chat it can be installed in (own chat, one-to-one, group), and its
permissions, chosen from:

- **send**: send messages to the chat it is installed in;
- **answer members**: act on commands and replies from other people in
  that chat (needed only in one-to-one and group chats);
- **use Znga**: ask the configured Znga engine for completions;
- **read activity**: read Znga's recent activity, promises and found plans
  from visible chats (REQ-ZNGA-016, 018, 022).

implementations:
  - src/miniapps/mod.rs:Manifest
  - src/miniapps/mod.rs:Permission
  - src/miniapps/mod.rs:MiniApp

verifies:
  - src/miniapps/mod.rs:every_built_in_app_has_a_unique_manifest
  - src/miniapps/host.rs:an_undeclared_permission_is_refused
  - src/miniapps/host.rs:installs_respect_chat_kinds_and_locked_chats

## Acceptance
<!-- id: REQ-MINIAPP-001.A1 -->
- Every built-in app has a manifest with an id, name, icon, description,
  command word, chat kinds and permissions, and no two apps share an id or
  a command word.
<!-- id: REQ-MINIAPP-001.A2 -->
- An action needing a permission the manifest does not declare is refused
  and logged, even when the app is installed.
<!-- id: REQ-MINIAPP-001.A3 -->
- An app cannot be installed in a kind of chat its manifest does not list.

<!-- id: REQ-MINIAPP-002 -->
## A mini-app MUST be installed in one chat at a time, by the person, with the permissions shown

Apps are off everywhere until the person installs them. Installing happens
from the Mini-apps window for the open chat (or the own chat), after a
screen that lists the manifest's permissions in plain words. Each install
is one app in one chat on one account.

implementations:
  - src/miniapps/host.rs:Host.install
  - src/miniapps/host.rs:Host.uninstall
  - src/miniapps/ui.rs:install_sheet

verifies:
  - src/miniapps/host.rs:nothing_is_installed_and_uninstalled_apps_hear_nothing
  - src/miniapps/host.rs:locked_chats_stop_their_installs
  - src/miniapps/host.rs:uninstalling_stops_the_app_and_drops_its_data
  - src/miniapps/host.rs:the_master_switch_stops_everything_and_keeps_installs

## Acceptance
<!-- id: REQ-MINIAPP-002.A1 -->
- A new ZapFast has no app installed anywhere, and an app that is not
  installed in a chat receives no events from it and can send nothing to it.
<!-- id: REQ-MINIAPP-002.A2 -->
- Installing shows every permission from the manifest before the install
  takes effect, and cancelling installs nothing.
<!-- id: REQ-MINIAPP-002.A3 -->
- Locked chats, chats excluded from Znga, and broadcast lists are never
  offered for an install, and an install stops working when its chat
  becomes locked or excluded.
<!-- id: REQ-MINIAPP-002.A4 -->
- Uninstalling stops the app in that chat at once and deletes its data for
  that chat.
<!-- id: REQ-MINIAPP-002.A5 -->
- One switch turns all mini-apps off; while off, no app receives events
  or acts, and installs are kept for when it is turned back on.

<!-- id: REQ-MINIAPP-003 -->
## A mini-app MUST receive only the events addressed to it

An app is told about: commands with its command word (`/goal …`) in a chat
where it is installed; quote-replies to one of its own messages there;
clicks on its cards; and the times it asked to be woken at. It never sees
other messages in the chat. A command or reply from someone other than the
person reaches the app only with the "answer members" permission.

implementations:
  - src/miniapps/host.rs:Host.observe
  - src/miniapps/host.rs:Host.tick
  - src/miniapps/mod.rs:Event

verifies:
  - src/miniapps/mod.rs:commands_are_found_only_with_the_command_word
  - src/miniapps/host.rs:commands_reach_the_app_once_and_other_text_does_not
  - src/miniapps/host.rs:quote_replies_reach_the_app_only_for_its_own_messages
  - src/miniapps/host.rs:other_people_reach_only_apps_that_answer_members
  - src/miniapps/host.rs:old_replayed_and_history_messages_are_not_delivered
  - src/miniapps/host.rs:wake_ups_missed_while_closed_are_delivered_once_marked_late

## Acceptance
<!-- id: REQ-MINIAPP-003.A1 -->
- `/goal done` in a chat where Goal of the day is installed reaches that
  app as a command with the text `done`; the same text without the command
  word reaches no app.
<!-- id: REQ-MINIAPP-003.A2 -->
- A quote-reply to an app's message reaches that app; a quote-reply to any
  other message does not.
<!-- id: REQ-MINIAPP-003.A3 -->
- A command from another person is dropped unless the app has "answer
  members", and then it carries who sent it.
<!-- id: REQ-MINIAPP-003.A4 -->
- Messages from older history pages, replays, or more than 5 minutes old
  when they arrive are not delivered, and each message is delivered at most
  once.
<!-- id: REQ-MINIAPP-003.A5 -->
- A wake-up time that passed while ZapFast was closed is delivered once
  when ZapFast starts, marked late, so the app decides whether to act.

<!-- id: REQ-MINIAPP-004 -->
## Every mini-app action MUST pass the Sentinel and be capped

Apps never touch the account directly: they return actions (send a message,
update a card, ask Znga, wake me at, store data) and the host carries them
out. Each one is checked by the Sentinel with the app's name, the chat, and
the origin of the event that caused it (the person, a schedule, or another
person's text), and logged.

implementations:
  - src/miniapps/host.rs:Host.apply
  - src/miniapps/mod.rs:AppAction
  - src/znga/sentinel.rs:check

verifies:
  - src/znga/sentinel.rs:mini_apps_send_only_home_and_never_for_others_with_znga
  - src/miniapps/host.rs:sends_are_capped_per_day_and_every_ten_seconds
  - src/miniapps/host.rs:every_action_is_logged_with_the_app_name

## Acceptance
<!-- id: REQ-MINIAPP-004.A1 -->
- An app can send only to the chat the triggering event came from, and
  only when that install has "send"; a send to any other chat is refused.
<!-- id: REQ-MINIAPP-004.A2 -->
- An action caused by another person's text is refused unless the install
  has "answer members"; "use Znga" and "read activity" are refused for it
  in any case (REQ-ZNGA-017.A2).
<!-- id: REQ-MINIAPP-004.A3 -->
- Every message an app sends starts with its icon and name (e.g.
  `🎯 Goal of the day`), so it is never taken for the person's own words or
  for a question to Znga (REQ-ZNGA-014.A1).
<!-- id: REQ-MINIAPP-004.A4 -->
- Each install sends at most 20 messages a day and at most 1 every 10
  seconds; actions past either cap are refused and logged.
<!-- id: REQ-MINIAPP-004.A5 -->
- Every allowed and refused action appears in the Sentinel log with the
  app's name.

<!-- id: REQ-MINIAPP-005 -->
## A mini-app MUST draw its panel only as cards the host renders

Apps describe cards; the host draws them in the Mini-apps window. A card
has a title, lines of text, an optional progress bar, a list of steps that
can be ticked, text fields whose values come with every click, and up to
four buttons. A click becomes an event for the app
(REQ-MINIAPP-003). Apps cannot draw anything else.

implementations:
  - src/miniapps/mod.rs:Card
  - src/miniapps/ui.rs:card

verifies:
  - src/miniapps/mod.rs:cards_show_at_most_four_buttons
  - src/miniapps/host.rs:clicks_deliver_one_event_with_their_control

## Acceptance
<!-- id: REQ-MINIAPP-005.A1 -->
- The window lists the installs on the active account, each with its
  app's current cards.
<!-- id: REQ-MINIAPP-005.A2 -->
- Clicking a button or ticking a step delivers exactly one event to the
  app, with the card and control it came from.
<!-- id: REQ-MINIAPP-005.A3 -->
- A card with more than four buttons shows the first four; text is shown
  as plain text, never as markup.

<!-- id: REQ-MINIAPP-006 -->
## A mini-app's data MUST be kept encrypted on this computer, per install

Each install gets its own small store of named values. It lives in an
encrypted file beside the account's archive, unlocked with the archive's
key like `znga.db` (REQ-ZNGA-016.A3), and is read and written off the
interface thread.

implementations:
  - src/miniapps/store.rs:Store
  - src/miniapps/store.rs:Db
  - src/archive.rs:Archive.znga_connection

verifies:
  - src/miniapps/store.rs:values_are_kept_per_install
  - src/miniapps/store.rs:values_survive_a_restart_and_need_the_key
  - src/miniapps/host.rs:stores_are_capped_and_the_app_is_told

## Acceptance
<!-- id: REQ-MINIAPP-006.A1 -->
- Values an app stores are there after ZapFast restarts.
<!-- id: REQ-MINIAPP-006.A2 -->
- The file cannot be opened without the archive's key.
<!-- id: REQ-MINIAPP-006.A3 -->
- An app can read only its own install's values.
<!-- id: REQ-MINIAPP-006.A4 -->
- One install stores at most 1 MB; writes past that are refused and the
  app is told.

<!-- id: REQ-MINIAPP-007 -->
## A failing mini-app MUST NOT affect ZapFast or other apps

implementations:
  - src/miniapps/host.rs:Host.run

verifies:
  - src/miniapps/host.rs:a_failing_app_is_turned_off_alone
  - src/miniapps/host.rs:znga_errors_come_back_to_the_app

## Acceptance
<!-- id: REQ-MINIAPP-007.A1 -->
- An app that fails while handling an event (or keeps asking for more
  without stopping) is turned off in that install, shows the error on its
  card with "Turn back on", and every other app keeps working.
<!-- id: REQ-MINIAPP-007.A2 -->
- An app's Znga request that fails or times out comes back to the app as
  an error, never as silence.
<!-- id: REQ-MINIAPP-007.A3 -->
- Znga requests from apps run on Znga's background lane, so they never
  delay a question the person asked (REQ-ZNGA-015.A2).

<!-- id: REQ-MINIAPP-008 -->
## In the own chat, commands for an app MUST go to the app, not to Znga

With Znga's self-chat on (REQ-ZNGA-012), every text in the own chat is a
question for Znga. A text starting with an installed app's command word,
or a quote-reply to an app's message, is the app's instead.

implementations:
  - src/miniapps/host.rs:Host.claims
  - src/miniapps/hooks.rs:claims
  - src/znga/hooks.rs:observe_messages

verifies:
  - src/miniapps/host.rs:app_commands_and_messages_are_claimed_from_znga

## Acceptance
<!-- id: REQ-MINIAPP-008.A1 -->
- `/goal done` in the own chat, with Goal of the day installed there and
  Znga's self-chat on, is handled by the app and gets no Znga answer.
<!-- id: REQ-MINIAPP-008.A2 -->
- An app's own messages in the own chat are never taken as questions to
  Znga, and Znga's are never delivered to an app.
<!-- id: REQ-MINIAPP-008.A3 -->
- `/goal` text when Goal of the day is not installed in the own chat goes
  to Znga as before.

<!-- id: REQ-GOAL-001 -->
## Goal of the day MUST propose one goal each morning, from what is going on

Installed in the own chat (its only chat kind), with "send", "use Znga" and
"read activity". At the morning time (08:00 by default) it asks Znga for
one goal for today with up to three small steps, from: yesterday's goal if
it was not done, open promises due today or tomorrow (REQ-ZNGA-018), plans
found for today (REQ-ZNGA-022), and recent activity in visible chats. The
proposal is sent to the own chat and shown on the app's card.

implementations:
  - src/miniapps/goal.rs:Turn.wake
  - src/miniapps/goal.rs:parse_proposal
  - src/miniapps/goal.rs:proposal_prompt

verifies:
  - src/miniapps/goal.rs:the_proposal_comes_at_the_morning_time_and_not_before
  - src/miniapps/goal.rs:the_prompt_has_yesterday_promises_and_plans
  - src/miniapps/goal.rs:ok_takes_the_proposal_and_text_sets_your_own
  - src/miniapps/goal.rs:without_znga_the_morning_asks_for_a_goal
  - src/miniapps/goal.rs:a_missed_morning_is_sent_late_only_before_noon

## Acceptance
<!-- id: REQ-GOAL-001.A1 -->
- At the morning time, one proposal with a goal and at most three steps is
  sent to the own chat and shown on the card, and none is sent before it.
<!-- id: REQ-GOAL-001.A2 -->
- The prompt includes yesterday's unfinished goal, promises due today or
  tomorrow, and today's plans, and nothing from locked or excluded chats.
<!-- id: REQ-GOAL-001.A3 -->
- `/goal ok` or the card's Accept button makes the proposal today's goal;
  `/goal <text>` (while there is no goal today), `/goal set <text>`, or the
  card's field makes the person's own text today's goal instead. `/goal new`
  asks for a proposal at once; it is not a scheduled message.
<!-- id: REQ-GOAL-001.A4 -->
- When Znga is off or its engine fails, the app sends one message asking
  "What's your goal today?" instead of a proposal.
<!-- id: REQ-GOAL-001.A5 -->
- If ZapFast was closed at the morning time, the proposal is sent when it
  starts if that is before 12:00, and skipped otherwise.

<!-- id: REQ-GOAL-002 -->
## Goal of the day MUST track progress through the day

implementations:
  - src/miniapps/goal.rs:Turn.with_day
  - src/miniapps/goal.rs:parse_progress

verifies:
  - src/miniapps/goal.rs:progress_commands_and_steps_change_the_goal
  - src/miniapps/goal.rs:free_text_is_read_by_znga_into_progress
  - src/miniapps/goal.rs:the_check_in_is_skipped_when_done_or_without_a_goal

## Acceptance
<!-- id: REQ-GOAL-002.A1 -->
- At the check-in time (13:00 by default) one message asks how it is
  going, with the steps not yet done; no check-in is sent when the goal is
  already done or there is no goal today.
<!-- id: REQ-GOAL-002.A2 -->
- `/goal done` marks the goal done; `/goal 50%` sets progress; `/goal step
  2` ticks step 2; ticking steps on the card does the same.
<!-- id: REQ-GOAL-002.A3 -->
- Other `/goal` text of more than one word once today has a goal, or a
  quote-reply to a goal message, is read by Znga
  into progress (done, a percentage, or steps done); when Znga cannot tell,
  progress is unchanged and the app says what it understood.
<!-- id: REQ-GOAL-002.A4 -->
- Every change is confirmed with one short message and shown on the card
  at once.

<!-- id: REQ-GOAL-003 -->
## Goal of the day MUST close each day with a recap

implementations:
  - src/miniapps/goal.rs:recap

verifies:
  - src/miniapps/goal.rs:the_recap_gives_the_outcome_streak_and_carry

## Acceptance
<!-- id: REQ-GOAL-003.A1 -->
- At the evening time (20:00 by default) one message gives the outcome
  (done, partly done with the progress, or not done) and the current streak
  of days with a done goal.
<!-- id: REQ-GOAL-003.A2 -->
- A goal not done is offered for tomorrow; `/goal carry` or the card's
  button makes it the base of tomorrow's proposal.
<!-- id: REQ-GOAL-003.A3 -->
- Days are counted in the computer's time zone, and a day with no goal
  neither breaks nor extends the streak.

<!-- id: REQ-GOAL-004 -->
## Goal of the day MUST stay quiet when asked and never send more than three scheduled messages a day

implementations:
  - src/miniapps/goal.rs:Turn.scheduled
  - src/miniapps/goal.rs:Schedule

verifies:
  - src/miniapps/goal.rs:no_more_than_three_scheduled_messages_a_day
  - src/miniapps/goal.rs:pause_resume_and_skip_quiet_the_schedule
  - src/miniapps/goal.rs:help_and_unknown_words_list_the_commands
  - src/miniapps/goal.rs:times_and_days_are_changed_on_the_card

## Acceptance
<!-- id: REQ-GOAL-004.A1 -->
- On any day it sends at most three scheduled messages (proposal,
  check-in, recap); replies to the person's commands are not counted.
<!-- id: REQ-GOAL-004.A2 -->
- `/goal pause` stops scheduled messages until `/goal resume`; `/goal
  skip` skips the rest of today.
<!-- id: REQ-GOAL-004.A3 -->
- The three times and the days of the week it runs on can be changed on
  its card; by default it runs every day.
<!-- id: REQ-GOAL-004.A4 -->
- `/goal` alone, or `/goal help`, answers with the list of commands; an
  unknown `/goal` word changes nothing and answers with the same list.
<!-- id: REQ-GOAL-004.A5 -->
- While paused or skipped, no scheduled message is sent, including late
  ones delivered when ZapFast starts (REQ-MINIAPP-003.A5).

<!-- id: REQ-GOAL-005 -->
## Goal of the day MUST keep a short history the person can see and delete

implementations:
  - src/miniapps/goal.rs:History

verifies:
  - src/miniapps/goal.rs:history_keeps_ninety_days_shows_fourteen_and_can_be_deleted
  - src/miniapps/goal.rs:history_never_reaches_a_chat

## Acceptance
<!-- id: REQ-GOAL-005.A1 -->
- Each day's goal, steps, progress and outcome are kept for 90 days in the
  app's store (REQ-MINIAPP-006) and older days are dropped.
<!-- id: REQ-GOAL-005.A2 -->
- The card shows the last 14 days and the streak.
<!-- id: REQ-GOAL-005.A3 -->
- "Delete history" on the card removes every past day; uninstalling
  removes everything (REQ-MINIAPP-002.A4).
<!-- id: REQ-GOAL-005.A4 -->
- History is never sent to any chat, and the only part of it given to Znga
  is yesterday's goal, for the morning proposal (REQ-GOAL-001.A2).
