---
id: ZPROF-DOC
title: Znga profiles of contacts, groups and group members
owner: znga
priority: medium
---

<!-- id: ZPROF-DOC -->
# Znga profiles of contacts, groups and group members

Znga acts as the person's digital twin: it drafts replies, answers
questions about chats and speaks first when something needs them. Today
it knows each chat only as up to 20 loose facts (REQ-ZNGA-019). This
document gives Znga a **profile** of every contact, every group and every
member of a group, built from the chats it may see, so its drafts, answers
and questions fit who it is talking to and about.

A **contact profile** says who the contact is to the person and how the
person talks with them. A **group profile** says what the group is for,
who is active in it and how it feels, with a summary and insights. A
**member profile** describes one person as seen across the groups the
person shares with them, so someone met only in groups is known too.

House rules this keeps (specs/znga.md, specs/znga-chat.md, AGENTS.md):
Znga is opt-in; it never reads locked or excluded chats; it never sends
anything without the person's click; profiles stay on this computer in
the encrypted `znga.db`; message text and profile contents never reach
the log; every model call passes the Sentinel and is logged by kind only.

Non-goals: sharing or exporting profiles to anyone; showing a profile to
the contact it describes; profiles of people in locked or excluded chats;
Znga acting on a profile by sending messages on its own.

<!-- id: REQ-ZPROF-001 -->
## Znga MUST keep a profile of each contact, group and group member

Profiles are kept per account. Every field records where it came from:
**learned** (inferred from chats, with the date it was last seen), or
**confirmed** (written or approved by the person). Confirmed values are
never overwritten by learning.

A contact profile holds: who they are to the person (family, partner,
friend, colleague, client, service, other), how the person talks with
them (language, formality, usual length, emoji, typical reply time),
topics they share, preferences and dislikes, important dates, people
they mention, and free facts (today's REQ-ZNGA-019 facts, carried over).

A group profile holds: what the group is for (its theme), the person's
role in it, its norms (language, tone, when it is busy), its most active
members, the sentiment of the group and of each member, a summary of
recent activity and insights (open questions, decisions, plans, recurring
topics).

A member profile holds the contact-profile fields that can be learned
from group messages, keyed by the member's WhatsApp id, so it is the same
person in every shared group and joins their contact profile when the
person also chats with them directly.

implementations:
  - src/znga/profiles.rs:Profile
  - src/znga/profiles.rs:Profiles
  - src/znga/store.rs:Db.save_profiles
  - src/znga/learn.rs:save

verifies:
  - src/znga/profiles.rs:a_confirmed_value_is_never_overwritten_by_learning
  - src/znga/profiles.rs:the_same_id_on_two_accounts_is_two_profiles
  - src/znga/profiles.rs:a_member_seen_in_two_groups_is_one_profile_shared_with_their_chat
  - src/znga/store.rs:profiles_survive_a_reload_with_their_sources
  - src/znga/learn.rs:an_account_whose_memory_was_not_read_learns_nothing

## Acceptance
<!-- id: REQ-ZPROF-001.A1 -->
- Facts stored under REQ-ZNGA-019 before this change appear, unchanged,
  as free facts in the matching contact or group profile after it.
<!-- id: REQ-ZPROF-001.A2 -->
- A field the person edited keeps their value after new messages would
  have changed it, and is shown as confirmed.
<!-- id: REQ-ZPROF-001.A3 -->
- A member seen in two groups has one member profile, listed in both
  groups, and the same profile is shown on their contact once the person
  chats with them directly.
<!-- id: REQ-ZPROF-001.A4 -->
- The same WhatsApp id on two accounts has two separate profiles.
<!-- id: REQ-ZPROF-001.A5 -->
- If `znga.db` cannot be read, Znga keeps no profiles in memory for that
  account, learns nothing, and never overwrites the unreadable file.

<!-- id: REQ-ZPROF-002 -->
## Znga MUST learn profiles from new messages and once from history

As messages arrive, Znga updates profiles in the same background pass
that extracts facts and promises today, at most once per chat per pass
interval.

When profiles are first turned on, Znga reads recent history once in
every chat it may see, newest chats first (the last 90 days or 500
messages per chat, whichever is fewer), and shows its progress in
Settings › Znga › People with Pause and Cancel. The backfill runs only
while the engine may read messages (the "Let this engine read my
messages" switch) or the engine runs on this computer; otherwise it
waits and says why. It spends at most 200 model calls a day on profiles
(live updates included) and resumes the next day where it
stopped. A chat backfilled once is not backfilled again; "Learn from
history" on one profile runs it again for that chat.

implementations:
  - src/znga/learn.rs:observe
  - src/znga/learn.rs:history
  - src/znga/learn.rs:start_backfill
  - src/znga/profiles.rs:Backfill
  - src/znga/prompt.rs:profile

verifies:
  - src/znga/learn.rs:a_contact_is_learned_from_its_messages_and_used_in_drafts
  - src/znga/learn.rs:the_history_read_waits_for_message_reading_with_a_cloud_engine
  - src/znga/learn.rs:the_daily_limit_holds_the_rest_for_tomorrow

## Acceptance
<!-- id: REQ-ZPROF-002.A1 -->
- Turning profiles on with a cloud engine and message reading off starts
  no backfill and shows why; turning reading on starts it.
<!-- id: REQ-ZPROF-002.A2 -->
- The backfill stops at the daily call limit and continues the next day
  from the next unread chat, without repeating finished chats.
<!-- id: REQ-ZPROF-002.A3 -->
- Cancel stops the backfill and keeps what was learned so far.
<!-- id: REQ-ZPROF-002.A4 -->
- Locked and excluded chats are never read, by the backfill or by later
  passes.
<!-- id: REQ-ZPROF-002.A5 -->
- A model failure during learning leaves the profile as it was and the
  chat is tried again in a later pass.

<!-- id: REQ-ZPROF-003 -->
## Sensitive traits MUST be marked and held back until confirmed

Znga may infer health, religion, politics, sexuality and money troubles
from messages, but it marks each such value sensitive. A sensitive value
that is not confirmed is shown only in the profile, marked, with Confirm
and Delete; it is never used in a draft, an answer, a proactive message
or a question, and never sent to the engine in another chat's request.
A value the person writes in themselves is confirmed.

implementations:
  - src/znga/profiles.rs:looks_sensitive
  - src/znga/profiles.rs:Value.usable
  - src/znga/hooks.rs:twin_for

verifies:
  - src/znga/profiles.rs:sensitive_values_stay_out_of_prompts_until_confirmed
  - src/znga/profiles.rs:a_deleted_value_is_not_learned_again

## Acceptance
<!-- id: REQ-ZPROF-003.A1 -->
- An inferred sensitive value appears marked in the profile and is absent
  from the prompt of a draft to that contact.
<!-- id: REQ-ZPROF-003.A2 -->
- After Confirm, the same value is present in the draft prompt.
<!-- id: REQ-ZPROF-003.A3 -->
- A deleted sensitive value is not inferred again (as REQ-ZNGA-019's
  forgotten facts).

<!-- id: REQ-ZPROF-004 -->
## Group profiles MUST carry analytics, a summary and insights

For each group Znga may see, the group profile shows:

- **Theme**: what the group is about, in one line, and its recurring
  topics.
- **Active members**: the members ranked by messages over the last 30
  days, with their counts, counted on this computer without the engine.
- **Sentiment**: the group's overall mood and each active member's
  (positive, neutral, negative or mixed) with its trend against the
  previous period.
- **Summary and insights**: what happened recently, open questions,
  decisions, plans and anything waiting for the person.
- **Members**: each member's profile (REQ-ZPROF-001).

Counts update as messages arrive. Theme, sentiment, summary and insights
are refreshed when enough has happened since the last refresh (50 new
messages, or 7 days with at least one), and on demand with Refresh. A
contact's profile is relearned after 10 new messages.

implementations:
  - src/znga/profiles.rs:activity
  - src/znga/learn.rs:activity
  - src/znga/prompt.rs:profile

verifies:
  - src/znga/profiles.rs:active_members_are_counted_over_thirty_days_with_former_members_marked
  - src/znga/learn.rs:active_members_are_counted_without_the_engine
  - src/znga/learn.rs:a_group_is_learned_with_tagged_members_and_no_numbers
  - src/znga/profiles.rs:groups_refresh_only_after_enough_has_happened

## Acceptance
<!-- id: REQ-ZPROF-004.A1 -->
- Active member counts match the number of messages each member sent in
  the group in the last 30 days, with no engine configured.
<!-- id: REQ-ZPROF-004.A2 -->
- After a refresh the group shows a theme, an overall sentiment, a
  sentiment for each active member and a summary.
<!-- id: REQ-ZPROF-004.A3 -->
- A group with fewer new messages than the refresh threshold is not sent
  to the engine again until Refresh is pressed.
<!-- id: REQ-ZPROF-004.A4 -->
- Members who left the group stay in its member list marked as former
  members until their messages fall out of the 30-day window.

<!-- id: REQ-ZPROF-005 -->
## Znga MUST use profiles when it drafts, answers and speaks first

Drafts to a contact follow their profile's relationship, language,
formality and length; drafts in a group follow the group's norms and the
person's role in it. Answers in the Znga chat may use any profile the
question is about ("what is the school group about?", "how is Sam
doing?"). Needs you alerts may name why the sender matters ("your
manager"). Only confirmed sensitive values are ever used.

implementations:
  - src/znga/hooks.rs:twin_for
  - src/znga/prompt.rs:twin_system
  - src/znga/learn.rs:named_in
  - src/znga/chat.rs:needs_you

verifies:
  - src/znga/learn.rs:a_contact_is_learned_from_its_messages_and_used_in_drafts
  - src/znga/learn.rs:needs_you_names_why_the_sender_matters

## Acceptance
<!-- id: REQ-ZPROF-005.A1 -->
- A draft prompt for a contact includes their relationship and how the
  person talks with them; one for a group includes its norms and the
  person's role.
<!-- id: REQ-ZPROF-005.A2 -->
- Asking the Znga chat about a group answers from its profile, and
  asking about a locked or excluded chat is refused as today.

<!-- id: REQ-ZPROF-006 -->
## Znga SHOULD ask the person to fill gaps, at most once a day

When a profile lacks something that would change its drafts (who someone
is to the person, the language to use), Znga asks in its chat with
answer buttons ("Is Ada your sister, a friend or a colleague?") and a
Skip. It asks at most one question a day, never in quiet hours, never
about a sensitive trait, and never the same question again after Skip.
The answer is stored as confirmed. Asking can be switched off in
Settings › Znga.

implementations:
  - src/znga/chat.rs:profile_question
  - src/znga/profiles.rs:Profiles.next_question

verifies:
  - src/znga/learn.rs:one_question_a_day_and_the_answer_is_confirmed
  - src/znga/profiles.rs:a_skipped_question_is_not_asked_again

## Acceptance
<!-- id: REQ-ZPROF-006.A1 -->
- With two gaps open, one question is posted on a day and the second the
  next day.
<!-- id: REQ-ZPROF-006.A2 -->
- Answering stores the value as confirmed; Skip stores nothing and the
  question is not asked again.
<!-- id: REQ-ZPROF-006.A3 -->
- No question is posted during quiet hours or with asking switched off.

<!-- id: REQ-ZPROF-007 -->
## The person MUST be able to see, edit and forget every profile

A **Znga knows** section in the contact and group info panel shows the
profile (for a group, its analytics, summary and members) with edit,
delete and "Learn from history". Settings › Znga › People lists every
profile, searchable, with the backfill's progress and Forget all. In the
Znga chat, "what do you know about Ada?" shows her profile, and "forget
that Ada …" deletes that value. A deleted value is not learned again.
Turning profiles off keeps them until the person forgets them; excluding
or locking a chat deletes its profile and its members' values learned
only from it.

implementations:
  - src-tauri/src/znga.rs:profile_view
  - src-tauri/src/znga.rs:profile_op
  - web/src/features/info/ZngaKnows.tsx:ZngaKnows
  - web/src/features/settings/ZngaSettings.tsx:People
  - src/znga/chat.rs:show_profile
  - src/znga/chat.rs:forget_in

verifies:
  - src/znga/learn.rs:the_chat_shows_and_forgets_what_znga_knows
  - src/znga/learn.rs:excluding_a_chat_deletes_its_profile

## Acceptance
<!-- id: REQ-ZPROF-007.A1 -->
- An edit in the info panel shows in Settings › Znga › People and is
  used by the next draft.
<!-- id: REQ-ZPROF-007.A2 -->
- Forget all leaves no profile in `znga.db`, and the backfill does not
  restart until the person runs it again.
<!-- id: REQ-ZPROF-007.A3 -->
- Excluding a group deletes its group profile and the member values
  learned only from it, and keeps values learned in other chats.
<!-- id: REQ-ZPROF-007.A4 -->
- "Forget that Ada likes jazz" in the Znga chat removes that value and
  says so.
