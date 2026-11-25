// Every entry of a chat's menu (REQ-WEBUI-006: pin, mute, archive, mark
// read or unread, favourite, info, clear, delete). Deleting and clearing
// ask first, through `confirm`.
import { commands, type ChatRow } from "../../lib/bridge";
import type { Entry } from "../../components/Menu";

export interface Ask {
  title: string;
  body: string;
  action: string;
  run: () => void;
}

const HOUR = 3600;

export function chatMenu(row: ChatRow, confirm: (ask: Ask) => void, archived: boolean): (Entry | "-")[] {
  const now = Math.floor(Date.now() / 1000);
  const unread = row.unread > 0 || row.markedUnread;
  const mute: (Entry | "-")[] = row.muted
    ? [{ label: "Unmute notifications", run: () => commands.setMuted(row.id, null) }]
    : [
        { label: "Mute for 8 hours", run: () => commands.setMuted(row.id, now + 8 * HOUR) },
        { label: "Mute for 1 week", run: () => commands.setMuted(row.id, now + 7 * 24 * HOUR) },
        { label: "Mute always", run: () => commands.setMuted(row.id, 0) },
      ];
  return [
    { label: row.group ? "Group info" : "Contact info", run: () => commands.openInfo(row.id) },
    "-",
    { label: archived ? "Unarchive chat" : "Archive chat", run: () => commands.setArchived(row.id, !archived) },
    ...mute,
    { label: row.pinned ? "Unpin chat" : "Pin chat", run: () => commands.setPinned(row.id, !row.pinned) },
    { label: unread ? "Mark as read" : "Mark as unread", run: () => commands.setRead(row.id, unread) },
    {
      label: row.favorite ? "Remove from Favourites" : "Add to Favourites",
      run: () => commands.setFavorite(row.id, !row.favorite),
    },
    "-",
    {
      label: "Clear chat",
      danger: true,
      run: () =>
        confirm({
          title: `Clear this chat?`,
          body: "Messages will be removed from this computer and your phone. The chat stays in the list.",
          action: "Clear chat",
          run: () => commands.clearChat(row.id),
        }),
    },
    row.group
      ? {
          label: "Exit group",
          danger: true,
          run: () =>
            confirm({
              title: `Exit "${row.title}"?`,
              body: "You will stop getting messages from this group. Only the group's admins are told.",
              action: "Exit group",
              run: () => commands.leaveGroup(row.id),
            }),
        }
      : {
          label: "Delete chat",
          danger: true,
          run: () =>
            confirm({
              title: `Delete chat with ${row.title}?`,
              body: "The chat and its messages will be deleted from this computer and your phone.",
              action: "Delete chat",
              run: () => commands.deleteChat(row.id),
            }),
        },
  ];
}
