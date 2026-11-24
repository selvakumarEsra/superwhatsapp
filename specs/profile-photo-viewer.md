---
id: PHOTO-DOC
title: Profile photo viewer
owner: fork
priority: medium
---

<!-- id: PHOTO-DOC -->
# Profile photo viewer

The contact / group info panel shows the chat's profile photo. Clicking it
opens the photo on its own in the full-window image viewer, where it can be
zoomed (upstream #640). A profile photo belongs to no message, so the viewer
holds it with an empty chat and message and browses no album.

Bug (2026-11-14): clicking the photo showed nothing. The viewer opened and was
closed again on the next frame by the rule "the viewer shows the open chat's
media, and leaves with it" (`App::tick_video`), because the photo's empty chat
is never the open chat.

<!-- id: REQ-PHOTO-001 -->
## Clicking the info panel's profile photo MUST open it in the viewer and keep it open

A viewer opened on a picture that belongs to no chat (empty chat id) stays
open until the person closes it. The rules that close the viewer when its chat
is left, locked or no longer listed apply only to a viewer that belongs to a
chat. The photo shown is the full-size profile picture when it has been
fetched, otherwise the preview picture already on screen.

implementations:
  - src/app.rs:App.tick_video
  - src/app.rs:App.open_viewer
  - src/ui/info.rs:view_photo

verifies:
  - src/app.rs:a_profile_picture_stays_open_in_the_viewer

## Acceptance
<!-- id: REQ-PHOTO-001.A1 -->
- With a chat open and its info panel showing a photo, clicking the photo
  opens the viewer, and the viewer is still open after further frames.
<!-- id: REQ-PHOTO-001.A2 -->
- A chat-list refresh (`Event::Chats`) while the photo is in the viewer does
  not close it.
<!-- id: REQ-PHOTO-001.A3 -->
- Close, Escape or a click on the empty field still closes the viewer.
<!-- id: REQ-PHOTO-001.A4 -->
- A viewer opened on a chat's media still closes when that chat is left
  (existing behaviour unchanged).
<!-- id: REQ-PHOTO-001.A5 -->
- Opening a profile photo asks the backend for no chat album (no
  `LoadChatMedia` for an empty chat id).
