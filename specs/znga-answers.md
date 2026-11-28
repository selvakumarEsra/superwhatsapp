---
id: ZANS-DOC
title: Znga's answers you can read, follow and act on
owner: selvakumarEsra
priority: high
---

<!-- id: ZANS-DOC -->
# Znga's answers you can read, follow and act on

This document covers three things in the Znga chat of the egui app (specs/znga-chat.md):

- **Links.** An answer that names messages, chats or groups links to them. Asking "show me the messages where people gave an email address" returns a list where each item opens that exact message.
- **Layout and actions.** Answers have readable formatting: headings, spacing, lists and emphasis. Hovering an answer shows small Copy, Reply and Forward buttons.
- **Translation.** In any chat Znga may read, a message written in another language has a "Translate to English" button on hover. It shows Znga's translation in the bubble, then turns into "Show original" to switch back.

The house rules of specs/znga.md and specs/znga-chat.md still apply:
- Znga is opt-in.
- Znga never reads locked or excluded chats.
- Nothing is sent to anyone without the person's click.
- Every Znga request passes the Sentinel and is logged without message text.
- Fork code stays in `src/znga/`, and upstream files get only one-line `// znga hook` lines (AGENTS.md). The new hooks are the message menu's "Translate with Znga" item, the translate button beside a bubble's hover buttons, and the bubble text showing a translation.

This is built in the egui app, the one SuperWhatsapp ships. The Tauri interface (specs/tauri-ui.md) is not covered yet.

<!-- id: REQ-ZANS-001 -->
## Znga's answers MUST link to the messages and chats they refer to

Every message Znga is given to answer a question has a short reference number. This covers recent messages and archive hits. Znga cites a message by its number, and the Znga chat shows each citation as a link naming the chat and the date. Clicking a link opens that chat at that message and highlights it, the same way clicking a quoted reply does. Older history loads when the message is not loaded yet.

Below an answer that cites messages, a Sources list shows each cited message once, in the order first cited. Each source shows the chat name, the sender, the date and the first line of the message. Each source is a link to its message, and its chat name opens the chat.

The links are saved with the answer, so they still work after a restart. A link to a chat that is now locked or excluded follows the app lock and Znga's exclusions: it opens nothing, and Znga never shows that message's text again.

implementations:
  - src/znga/archive_ask.rs:Hit
  - src/znga/archive_ask.rs:search
  - src/znga/proactive.rs:Seen
  - src/znga/prompt.rs:answer_with_hits
  - src/znga/answer.rs:cited
  - src/znga/thread.rs:Entry
  - src/znga/store.rs:save_thread
  - src/znga/chat.rs:answer_cited
  - src/znga/chat.rs:open_message
  - src/znga/chat_ui.rs:sources

verifies:
  - src/znga/prompt.rs:answers_with_hits_cite_the_archive
  - src/znga/answer.rs:citations_list_given_messages_once_in_order
  - src/znga/store.rs:an_answers_links_survive_a_reload
  - src/znga/store.rs:a_znga_db_from_before_links_gains_them
  - src/znga/chat.rs:an_answer_keeps_links_only_to_chats_znga_may_read
  - src/znga/chat.rs:a_cited_message_opens_at_it_and_hidden_chats_open_nothing

## Acceptance
<!-- id: REQ-ZANS-001.A1 -->
- Ask "show my messages that have an email address". Every message the answer lists is a link. Clicking one opens its chat with that message in view and highlighted, even when it is older than the loaded history.
<!-- id: REQ-ZANS-001.A2 -->
- The prompt gives each message a unique number in that question, and tells Znga to cite messages by those numbers. A citation of a number that was not given shows as plain text, never as a link.
<!-- id: REQ-ZANS-001.A3 -->
- An answer citing three messages from two chats shows a Sources list of three items, each naming its chat, sender and date.
<!-- id: REQ-ZANS-001.A4 -->
- After a restart, the saved answer's links still open the same messages.
<!-- id: REQ-ZANS-001.A5 -->
- If the chat a source belongs to becomes locked or excluded, clicking the source does not open it, and the source's message text is no longer shown.

<!-- id: REQ-ZANS-002 -->
## Znga's answers MUST be laid out for reading

Znga writes answers with light formatting:
- headings
- bullet and numbered lists
- bold and italic text
- inline code
- blank lines between paragraphs

The Znga chat draws the formatting, never the raw markers. Headings are larger and set off with space above them. List items are indented and keep their numbers or bullets. Paragraphs have space between them. An answer still being written is drawn the same way as it streams. Text from people, such as the person's own questions and quoted messages, is shown as written, without this formatting.

implementations:
  - src/znga/answer.rs:blocks
  - src/znga/answer.rs:spans
  - src/znga/prompt.rs:answer_with_hits
  - src/znga/chat_ui.rs:formatted
  - src/znga/chat_ui.rs:writing_bubble

verifies:
  - src/znga/answer.rs:formatting_is_recognised_and_markers_are_not_kept
  - src/znga/answer.rs:an_unclosed_marker_is_shown_as_written
  - src/znga/prompt.rs:answers_with_hits_cite_the_archive

## Acceptance
<!-- id: REQ-ZANS-002.A1 -->
- Given an answer containing `## Emails`, a three-item `-` list, a `1.` list and `**bold**`, the bubble shows a heading, two lists and bold text. It shows no `#`, `-` used as a bullet marker, or `**`.
<!-- id: REQ-ZANS-002.A2 -->
- The answer prompts ask for this formatting instead of "plain text".
<!-- id: REQ-ZANS-002.A3 -->
- An unclosed marker such as a lone `**` shows as written and never hides the rest of the answer.

<!-- id: REQ-ZANS-003 -->
## Znga's answers MUST offer Copy, Reply and Forward on hover

Hovering one of Znga's messages in the Znga chat shows a small row of buttons at its top corner: Copy, Reply and Forward. The row hides when the pointer leaves, and is reachable from the keyboard.

- **Copy** puts the answer on the clipboard as readable text: no formatting markers, and each citation written as "(chat, date)".
- **Reply** quotes the answer above the Znga composer. The next question is sent with that answer as its context, and the quote can be dismissed.
- **Forward** lets the person pick a chat, then opens that chat with the answer's text offered above its composer, as Edit in chat does: Use puts it in the composer. Nothing is sent until the person sends it.

implementations:
  - src/znga/chat_ui.rs:hover_actions
  - src/znga/chat_ui.rs:reply_quote
  - src/znga/chat_ui.rs:forward_picker
  - src/znga/answer.rs:plain
  - src/znga/chat.rs:send_message
  - src/znga/chat.rs:forward

verifies:
  - src/znga/answer.rs:plain_text_writes_citations_as_chat_and_date
  - src/znga/chat.rs:a_reply_asks_about_the_quoted_answer
  - src/znga/chat.rs:forwarding_offers_the_text_in_the_chat_and_sends_nothing

## Acceptance
<!-- id: REQ-ZANS-003.A1 -->
- Hovering a Znga answer shows Copy, Reply and Forward; moving away hides them; the person's own messages show Copy only.
<!-- id: REQ-ZANS-003.A2 -->
- Copy of an answer with a heading and a citation gives plain text with the heading's words and "(chat, date)" in place of the citation.
<!-- id: REQ-ZANS-003.A3 -->
- Reply shows the quoted answer above the composer. The prompt for the next question includes the quoted text. Dismissing the quote removes it.
<!-- id: REQ-ZANS-003.A4 -->
- Forward, then picking a chat, opens that chat with the text offered above its composer and sends nothing.

<!-- id: REQ-ZANS-004 -->
## Znga SHOULD translate messages written in another language, in place

In a chat Znga may read, one-to-one or group, Znga notices incoming text messages and captions that are not in the person's language. The person's language is Znga's "Translate to" setting, or English when it is unset. The check runs on this computer and sends nothing anywhere.

Hovering such a message shows a "Translate to English" button beside its React, Reply and Forward buttons. The button names the person's language. Clicking it sends only that message to Znga's engine, and the bubble's text is replaced by the translation, with a line saying it was translated and from which language. The button then reads "Show original", and clicking it brings the original text back. The two can be switched as often as the person likes: the translation is kept for the session and is not asked for again.

The same toggle is in each such message's menu ("Translate with Znga", then "Show original"). When the open chat has such messages, Znga's bar above the composer offers to translate the recent ones in place at once. Nothing is ever sent to anyone: the translation is only shown on this computer.

implementations:
  - src/znga/translate.rs:foreign
  - src/znga/translate.rs:inline
  - src/znga/hooks.rs:sync_inline
  - src/znga/hooks.rs:request_translate
  - src/znga/prompt.rs:translate_incoming
  - src/znga/sentinel.rs:check
  - src/znga/chat_ui.rs:translate_control
  - src/znga/chat_ui.rs:translate_bar
  - src/znga/chat_ui.rs:message_menu
  - src/ui/conversation.rs:rich_body

verifies:
  - src/znga/translate.rs:other_languages_are_found_and_english_is_not
  - src/znga/translate.rs:the_persons_own_language_is_not_offered
  - src/znga/translate.rs:numbered_translations_are_read_back
  - src/znga/translate.rs:a_toggle_shows_the_translation_and_then_the_original
  - src/znga/chat.rs:translating_sends_only_the_chosen_message_and_toggles_in_place
  - src/znga/chat.rs:locked_and_excluded_chats_never_translate
  - src/znga/sentinel.rs:translations_run_only_when_asked_and_never_in_hidden_chats

## Acceptance
<!-- id: REQ-ZANS-004.A1 -->
- Hovering a Hindi, Tamil or Spanish message in a one-to-one or group chat shows "Translate to English" beside React, Reply and Forward. An English message shows no such button, and neither does a message of only emoji, links or numbers.
<!-- id: REQ-ZANS-004.A2 -->
- No message text reaches an engine until the person clicks, and only the clicked message is sent.
<!-- id: REQ-ZANS-004.A3 -->
- After the click, the bubble shows the translation and a "translated from" line, and the button reads "Show original". Clicking it shows the original again, and the next "Translate to English" shows the kept translation without asking the engine again.
<!-- id: REQ-ZANS-004.A4 -->
- Locked or excluded chats never show the button. Each translation is logged by the Sentinel as a translate action, without message text.
