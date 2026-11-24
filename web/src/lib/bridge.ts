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
export interface State {
  link: Link; me: string | null; filter: string; search: string; chats: ChatRow[]; open: Open | null;
}

const empty: State = { link: { state: "starting" }, me: null, filter: "all", search: "", chats: [], open: null };
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
};
