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
export interface Swatch { id: string; label: string; color: string | null }
export interface Blocked { id: string; title: string; avatar: string | null }
export interface LockForm { mode: "set" | "change" | "turnOff"; error: "tooShort" | "mismatch" | "wrongCurrent" | null; busy: boolean }
export interface LockView { wrong: boolean; wait: number | null; checking: boolean; forgetting: "no" | "confirming" | "unlinking" }
export interface SettingsView {
  theme: "dark" | "light" | "system"; zoom: number; showWallpaper: boolean; enterSends: boolean;
  autoDownload: boolean; historyPrefetch: boolean; keepChatsArchived: boolean; pauseOtherMedia: boolean;
  notifications: boolean; notificationContent: boolean; groupSounds: boolean;
  messageSound: string; mentionSound: string; sendReadReceipts: boolean; sendTyping: boolean;
  keepRunning: boolean; startWithSystem: boolean | null; checkForUpdates: boolean; downloadUpdates: boolean;
  proxy: string; name: string | null; about: string | null; phone: string | null; version: string;
  wallpaperLight: string; wallpaperDark: string; swatchesLight: Swatch[]; swatchesDark: Swatch[];
  chatLockCode: boolean; appLock: boolean; autoLock: "oneMinute" | "fifteenMinutes" | "oneHour";
  lockForm: LockForm | null; blocked: Blocked[] | null; blockedLoading: boolean; blockedFailed: boolean;
  wallpaperImage: string | null; customTheme: string | null; themes: [string, string][];
  palette: [string, string][] | null;
  locale: string; language: string | null; languages: [string, string][];
}
export interface State {
  link: Link; me: string | null; filter: string; search: string; chats: ChatRow[]; unreadChats: number; open: Open | null;
  section: Section; starred: StarredRow[]; calls: CallRow[]; info: Info | null; settings: SettingsView;
  lock: LockView | null;
}

const empty: State = {
  link: { state: "starting" }, me: null, filter: "all", search: "", chats: [], unreadChats: 0, open: null,
  section: "chats", starred: [], calls: [], info: null,
  settings: {
    theme: "system", zoom: 1, showWallpaper: true, enterSends: true, autoDownload: true, historyPrefetch: true,
    keepChatsArchived: true, pauseOtherMedia: true, notifications: true, notificationContent: true, groupSounds: true,
    messageSound: "receive", mentionSound: "alert", sendReadReceipts: true, sendTyping: true, keepRunning: false,
    startWithSystem: null, checkForUpdates: true, downloadUpdates: false, proxy: "", name: null, about: null,
    phone: null, version: "", wallpaperLight: "theme", wallpaperDark: "theme", swatchesLight: [], swatchesDark: [],
    chatLockCode: false, appLock: false, autoLock: "fifteenMinutes", lockForm: null, blocked: null,
    blockedLoading: false, blockedFailed: false, wallpaperImage: null, customTheme: null, themes: [], palette: null,
    locale: "en", language: null, languages: [],
  },
  lock: null,
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
  setSetting: (key: string, value: unknown) => invoke("set_setting", { key, value }),
  unlink: () => invoke("unlink"),
  activity: () => invoke("activity"),
  unlock: (password: string) => invoke("unlock", { password }),
  lockNow: () => invoke("lock_now"),
  forgotPassword: (open: boolean, unlink = false) => invoke("forgot_password", { open, unlink }),
  appLockForm: (mode: LockForm["mode"] | null, current = "", next = "", confirm = "") =>
    invoke("app_lock_form", { mode, current, new: next, confirm }),
  setChatLockCode: (code: string | null) => invoke("set_chat_lock_code", { code }),
  fetchBlocklist: () => invoke("fetch_blocklist"),
  setBlocked: (chat: string, blocked: boolean) => invoke("set_blocked", { chat, blocked }),
  setWallpaperColor: (color: string, dark: boolean) => invoke("set_wallpaper_color", { color, dark }),
  setCustomTheme: (file: string) => invoke("set_custom_theme", { file }),
  themesFolder: (reload: boolean) => invoke("themes_folder", { reload }),
  wallpaperImage: (pick: boolean) => invoke("wallpaper_image", { pick }),
  setLanguage: (code: string | null) => invoke("set_language", { code }),
};
