---
id: ZANS-DOC
title: Znga answers you can act on
owner: selvakumarEsra
priority: high
---

<!-- id: ZANS-DOC -->
# Znga answers you can act on

Znga's answers in the Znga chat (specs/znga-chat.md) point at the messages,
chats and groups they talk about, so one click opens the exact message. They
read like a short document, with headings, lists and spacing, and each answer
offers Copy, Reply and Forward when the pointer is over it. In any chat, a
message in another language can be translated by Znga into a popup beside
it, without anything being sent.

Non-goals: sending anything on its own (Forward and translations only ever
fill the composer or show a popup; the person sends); translating whole
chats, or translating automatically without a click; the egui interface,
which keeps its plain-text answers.

<!-- id: REQ-ZANS-001 -->
## Answers MUST link to the messages and chats they cite

When Znga answers from archive messages it found, every message it uses is
cited by its number, and the answer shows each cited message as a link: the
sender, the chat, the date and the start of the text. Clicking one opens
that chat scrolled to that message, highlighted, as the search results do.
A chat or group the answer names by its exact title links to that chat.
"Show my messages that gave an email address" therefore answers with one
link per such message. A number that matches no message found is shown as
plain text, never as a link. A cited message whose chat has been removed
does not open anything; Znga says the chat is no longer here. A cited
message that was deleted from a chat that still exists opens the chat, and
the app's usual jump to a message says it cannot be found. Links are
kept with the answer, so they still work after the app restarts.

implementations:
  - src/znga/archive_ask.rs:Hit
  - src/znga/prompt.rs:answer_with_hits
  - src/znga/thread.rs:cited
  - src/znga/chat.rs:card_action
  - src-tauri/src/znga.rs:chat_view
  - web/src/features/znga/ZngaAnswer.tsx:ZngaAnswer

verifies:
  - src/znga/thread.rs:citations_become_links_and_unknown_numbers_stay_text
  - src/znga/prompt.rs:answers_with_hits_number_each_message
  - src/znga/chat.rs:a_cited_message_opens_where_it_is
  - web/src/features/znga/format.test.ts:citationsAndChatsBecomeLinks

## Acceptance
<!-- id: REQ-ZANS-001.A1 -->
- Asking "show all my messages that gave an email id" with three such
  messages in the archive gives an answer with three message links; clicking
  the second opens its chat with that message highlighted.
<!-- id: REQ-ZANS-001.A2 -->
- An answer that names a chat by its exact title shows the title as a link
  that opens the chat.
<!-- id: REQ-ZANS-001.A3 -->
- A citation number with no matching message renders as text, not a link.
<!-- id: REQ-ZANS-001.A4 -->
- After a restart the links in earlier answers still open their messages.
<!-- id: REQ-ZANS-001.A5 -->
- A link to a message whose chat has since been removed opens nothing and
  Znga says the chat is no longer here.

<!-- id: REQ-ZANS-002 -->
## Answers MUST be formatted for reading

Znga writes answers with light formatting: short headings, bulleted or
numbered lists, bold for the key fact, and paragraphs separated by space.
The Znga chat shows that formatting, and nothing else: no raw markup
characters are left showing, no HTML, scripts or images from the answer are
ever run or loaded, and web addresses open only through the app's browser
hand-off. An answer without formatting reads as plain paragraphs, unchanged.

implementations:
  - src/znga/prompt.rs:answer_with_hits
  - web/src/features/znga/format.ts:blocks
  - web/src/features/znga/ZngaAnswer.tsx:ZngaAnswer

verifies:
  - web/src/features/znga/format.test.ts:formatsHeadingsListsAndBold
  - web/src/features/znga/format.test.ts:markupStaysText
  - web/src/features/znga/format.test.ts:plainAnswersAreUnchanged

## Acceptance
<!-- id: REQ-ZANS-002.A1 -->
- An answer with `## Today`, a three-item list and `**9 am**` shows a
  heading, three list items and bold text, with no `#` or `*` visible.
<!-- id: REQ-ZANS-002.A2 -->
- An answer containing `<img src=x onerror=…>` or `<script>` shows it as
  text; nothing is loaded or run.
<!-- id: REQ-ZANS-002.A3 -->
- A plain answer with no markup shows exactly its text and line breaks.

<!-- id: REQ-ZANS-003 -->
## Each answer MUST offer Copy, Reply and Forward on hover

When the pointer is over one of Znga's answers (or it has keyboard focus),
small buttons appear on it: Copy puts the answer's text, without markup, on
the clipboard; Reply starts a follow-up in the Znga composer that quotes
the answer; Forward lets the person pick a chat and puts the answer's text
in that chat's composer as a draft, then opens the chat. Nothing is sent
until the person sends it. Answers still being written, and the person's
own lines, show no Forward. Each button names its action for screen readers.

implementations:
  - web/src/features/znga/ZngaAnswer.tsx:ZngaAnswer
  - web/src/features/znga/ForwardPicker.tsx:ForwardPicker
  - src-tauri/src/znga.rs:znga_forward
  - src/znga/chat.rs:forward

verifies:
  - src/znga/chat.rs:forwarding_an_answer_drafts_it_in_the_chosen_chat
  - src/znga/chat.rs:a_reply_quotes_the_answer_it_follows
  - src/znga/thread.rs:plain_text_drops_marks_and_citations
  - web/src/features/znga/format.test.ts:copyDropsMarks

## Acceptance
<!-- id: REQ-ZANS-003.A1 -->
- Copy on an answer with a heading and a list copies its words without `#`
  or `*`.
<!-- id: REQ-ZANS-003.A2 -->
- Forward to Ada opens Ada's chat with the answer in the composer, and no
  message is sent.
<!-- id: REQ-ZANS-003.A3 -->
- Reply puts a quote of the answer above the Znga composer; sending the
  follow-up gives Znga the quoted answer as context.
<!-- id: REQ-ZANS-003.A4 -->
- Forward to a chat Znga may not see (excluded or locked) is not offered.

<!-- id: REQ-ZANS-004 -->
## A message in another language MUST be translatable in place

In a chat, a text message that is not in the person's translation language
(Znga's "Translate to" setting, English by default) offers Translate when
the pointer is over it. Clicking it asks Znga for a translation of that
message only and shows it in a popup beside the message, with the
language it came from, Copy and Close. The message, the chat and WhatsApp
are not changed and nothing is sent. The popup says when Znga is off, the
chat is one Znga may not see, or the engine fails, and Translate is not
offered for messages in the translation language, media without a caption,
or chats Znga may not see. A translation is reused for the same message
while the app runs, without asking Znga again.

implementations:
  - src/znga/translate.rs:foreign
  - src/znga/translate.rs:request
  - src/znga/prompt.rs:translate_message
  - src-tauri/src/znga.rs:znga_translate
  - web/src/features/conversation/Translate.tsx:Translate

verifies:
  - src/znga/translate.rs:english_is_not_offered_and_other_languages_are
  - src/znga/translate.rs:a_translation_is_shown_once_and_reused
  - src/znga/translate.rs:a_failed_translation_can_be_tried_again

## Acceptance
<!-- id: REQ-ZANS-004.A1 -->
- Hovering a message "¿Nos vemos mañana a las 5?" shows Translate; clicking
  it shows "Shall we meet tomorrow at 5?" (or similar) in a popup marked as
  from Spanish, and the chat is unchanged.
<!-- id: REQ-ZANS-004.A2 -->
- An English message shows no Translate when the translation language is
  English.
<!-- id: REQ-ZANS-004.A3 -->
- Clicking Translate again on the same message shows the saved translation
  without a second request to the engine.
<!-- id: REQ-ZANS-004.A4 -->
- In a chat excluded from Znga, or with Znga off, Translate is not offered.
<!-- id: REQ-ZANS-004.A5 -->
- When the engine fails, the popup says so and offers Try again.
