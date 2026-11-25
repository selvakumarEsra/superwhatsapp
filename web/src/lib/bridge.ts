// The only way the page reaches the core (REQ-WEBUI-002): typed commands
// in, state events out. Types mirror src-tauri/src/view.rs.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createStore, reconcile } from "solid-js/store";

export type Status = "none" | "pending" | "sent" | "delivered" | "read" | "failed";

export type Link =
  | { state: "starting" }
  | { state: "unlinked"; qr: string | null; code: string | null }
  | { state: "connecting" }
  | { state: "connected" }
  | { state: "disconnected"; reason: string }
  | { state: "loggedOut" }
  | { state: "failed"; reason: string };

export interface Last { fromMe: boolean; sender: string | null; text: string; status: Status }
export interface ChatRow {
  id: string; title: string; avatar: string | null; group: boolean; time: number;
  unread: number; markedUnread: boolean; muted: boolean; pinned: boolean;
  favorite: boolean; favoritePosition: number; last: Last | null;
}
export interface Quote { author: string; text: string }
export interface Msg {
  id: string; fromMe: boolean; sender: string | null; senderId: string; time: number;
  kind: string; text: string; status: Status; edited: boolean; forwarded: boolean;
  quoted: Quote | null; reactions: [string, number][];
}
export interface Open { id: string; title: string; avatar: string | null; group: boolean; complete: boolean; messages: Msg[] }
export type Section = "chats" | "favourites" | "archived" | "starred" | "calls";
export interface StarredRow { chat: string; chatTitle: string; msg: Msg }
export interface CallRow {
  id: string; chat: string; title: string; avatar: string | null; group: boolean; time: number;
  outgoing: boolean; missed: boolean; video: boolean; duration: number;
}
export interface Member { id: string; name: string; avatar: string | null; admin: boolean }
export interface Info {
  id: string; title: string; avatar: string | null; group: boolean; channel: boolean;
  phone: string | null; description: string | null; memberCount: number; members: Member[];
  commonGroups: [string, string][]; muted: boolean; pinned: boolean; archived: boolean;
  favorite: boolean; canLeave: boolean; left: boolean; timer: number;
}
export interface State {
  link: Link; me: string | null; filter: string; search: string; chats: ChatRow[]; open: Open | null;
  section: Section; starred: StarredRow[]; calls: CallRow[]; info: Info | null;
}

const empty: State = {
  link: { state: "starting" }, me: null, filter: "all", search: "", chats: [], open: null,
  section: "chats", starred: [], calls: [], info: null,
};
const [state, setState] = createStore<State>(empty);
export { state };

export async function connect() {
  // Outside Tauri (a browser, for screenshots and tests), show a state
  // dumped by `zapfast-desktop --dump-state`.
  if (!("__TAURI_INTERNALS__" in window)) {
    const response = await fetch("/demo-state.json");
    if (response.ok) setState(reconcile((await response.json()) as State, { key: "id" }));
    return;
  }
  await listen<State>("zapfast://state", (event) => setState(reconcile(event.payload, { key: "id" })));
  await invoke("refresh");
}

export const commands = {
  openChat: (id: string) => invoke("open_chat", { id }),
  closeChat: () => invoke("close_chat"),
  sendText: (chat: string, text: string) => invoke("send_text", { chat, text }),
  setFilter: (filter: string) => invoke("set_filter", { filter }),
  search: (text: string) => invoke("search", { text }),
  setPinned: (chat: string, pinned: boolean) => invoke("set_pinned", { chat, pinned }),
  setFavorite: (chat: string, favorite: boolean) => invoke("set_favorite", { chat, favorite }),
  showSection: (section: Section) => invoke("show_section", { section }),
  setMuted: (chat: string, until: number | null) => invoke("set_muted", { chat, until }),
  setArchived: (chat: string, archived: boolean) => invoke("set_archived", { chat, archived }),
  setRead: (chat: string, read: boolean) => invoke("set_read", { chat, read }),
  deleteChat: (chat: string) => invoke("delete_chat", { chat }),
  clearChat: (chat: string) => invoke("clear_chat", { chat }),
  openInfo: (chat: string) => invoke("open_info", { chat }),
  closeInfo: () => invoke("close_info"),
  leaveGroup: (chat: string) => invoke("leave_group", { chat }),
  setStar: (chat: string, message: string, starred: boolean) => invoke("set_star", { chat, message, starred }),
  goToMessage: (chat: string, message: string) => invoke("go_to_message", { chat, message }),
};
