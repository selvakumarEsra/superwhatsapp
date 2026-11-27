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
  /** The time as egui writes it in the list. */
  stamp: string; typing: string | null; draft: string | null; mention: boolean;
}
export interface FilterChip { id: string; label: string; count: number }
export interface MediaView {
  url: string | null; thumb: string | null; width: number | null; height: number | null; size: number;
  mime: string; name: string | null; pages: number | null; seconds: number | null; waveform: number[];
  voice: boolean; gif: boolean; round: boolean;
}
export interface PreviewView { url: string; title: string | null; description: string | null; thumb: string | null }
export interface CallView {
  chat: string; name: string; avatar: string | null; video: boolean;
  phase: "incoming" | "live" | "over"; status: string; seconds: number | null;
  failed: boolean; muted: boolean; cameraOn: boolean; camera: boolean; connected: boolean; hidden: boolean;
  microphones: [string, string][]; speakers: [string, string][]; cameras: [string, string][];
  microphone: string | null; speaker: string | null; cameraDevice: string | null;
}
export interface Quote { author: string; text: string; hue: number; mine: boolean }
export interface Msg {
  id: string; fromMe: boolean; sender: string | null; senderId: string; senderHue: number; time: number;
  kind: string; text: string; status: Status; edited: boolean; forwarded: boolean;
  quoted: Quote | null; reactions: [string, number][]; reacted: boolean;
  /** The time as egui writes it in a bubble. */
  clock: string; starred: boolean; media: MediaView | null; preview: PreviewView | null;
}
export interface Open {
  id: string; title: string; avatar: string | null; group: boolean; complete: boolean; messages: Msg[];
  /** The composer offers Znga's tools here (REQ-ZCHAT-004). */
  zngaTools: boolean;
  subtitle: string; subtitleAccent: boolean;
}
/** The Znga chat's id; it never names a WhatsApp chat (REQ-ZCHAT-001). */
export const ZNGA_CHAT = "znga@superwhatsapp.local";
export interface ZAction { id: string; label: string }
export interface ZCard {
  kind: "needsYou" | "briefing" | "draft" | "catchUp" | "plan" | "voice" | "promise" | "drift" | "link";
  title: string; body: string; meta: string; actions: ZAction[]; outcome: string | null;
}
export interface ZEntry { id: number; at: number; fromMe: boolean; text: string; unasked: boolean; cards: ZCard[] }
export interface ZngaChat { entries: ZEntry[]; writing: string | null; asking: boolean }
export interface OfferView { id: number; chat: string; text: string; rewrite: boolean }
export interface ZFact { chat: string; chatTitle: string; text: string }
export interface ZPromise { chat: string; chatTitle: string; text: string; due: number | null; owesMe: boolean }
export interface ZAudit { at: number; action: string; origin: string; allowed: boolean; reason: string }
export interface ZngaSettings {
  enabled: boolean; engine: string; engines: [string, string][]; model: string; endpoint: string;
  defaultModel: string; defaultEndpoint: string; needsKey: boolean; keySource: string | null;
  cloud: boolean; cloudConsent: boolean; alerts: boolean; briefings: boolean;
  briefingMinMessages: number; briefingIntervalMinutes: number; digests: boolean;
  digestMorning: string; digestEvening: string; reminders: boolean; quietHours: boolean;
  quietFrom: string; quietTo: string; selfNotes: boolean; selfChat: boolean; includeGroups: boolean;
  promises: boolean; memory: boolean; checkinDays: number; yourName: string; voiceNotes: string;
  tone: string; voiceEndpoint: string; voiceModel: string; twin: [string, string][];
  excluded: [string, string][]; facts: ZFact[]; openPromises: ZPromise[]; audit: ZAudit[];
  status: [string, boolean] | null;
}
export type Section = "chats" | "favourites" | "archived" | "starred" | "calls" | "locked";
export interface StarredRow { chat: string; chatTitle: string; msg: Msg; moment: string }
export interface CallRow {
  id: string; chat: string; title: string; summary: string; stamp: string; bad: boolean; avatar: string | null; group: boolean; time: number;
  outgoing: boolean; missed: boolean; video: boolean; duration: number;
}
export interface Member { id: string; name: string; avatar: string | null; admin: boolean }
export interface MediaCell { id: string; src: string | null; video: boolean; seconds: number | null }
export interface Info {
  id: string; title: string; avatar: string | null; group: boolean; channel: boolean;
  phone: string | null; description: string | null; memberCount: number; members: Member[];
  commonGroups: [string, string][]; muted: boolean; pinned: boolean; archived: boolean;
  favorite: boolean; canLeave: boolean; left: boolean; timer: number;
  phoneDisplay: string | null; status: string | null; blockable: boolean; blocked: boolean; blockEditable: boolean;
  contactEditable: boolean; known: boolean; editorNames: [string, string];
  mediaCount: string | null; media: MediaCell[];
}
export interface Swatch { id: string; label: string; color: string | null }
export interface Blocked { id: string; title: string; avatar: string | null }
export interface LockForm { mode: "set" | "change" | "turnOff"; error: "tooShort" | "mismatch" | "wrongCurrent" | null; busy: boolean }
export interface LockView { wrong: boolean; wait: number | null; checking: boolean; forgetting: "no" | "confirming" | "unlinking" }
export interface FilesView {
  archive: string; downloads: string; customDownloads: boolean; log: string; storageHint: string;
  storageTotal: string | null; storageRows: [string, string, number][]; clearing: boolean; canClear: boolean;
}
export interface SettingsView {
  theme: "dark" | "light" | "system"; zoom: number; showWallpaper: boolean; enterSends: boolean; showShortcutHints: boolean; font: "system" | "inter";
  autoDownload: boolean; historyPrefetch: boolean; keepChatsArchived: boolean; pauseOtherMedia: boolean;
  notifications: boolean; notificationContent: boolean; groupSounds: boolean;
  messageSound: string; mentionSound: string; sendReadReceipts: boolean; sendTyping: boolean;
  keepRunning: boolean; startWithSystem: boolean | null; checkForUpdates: boolean; downloadUpdates: boolean;
  proxy: string; name: string | null; about: string | null; phone: string | null; version: string; files: FilesView;
  wallpaperLight: string; wallpaperDark: string; swatchesLight: Swatch[]; swatchesDark: Swatch[];
  chatLockCode: boolean; appLock: boolean; autoLock: "oneMinute" | "fifteenMinutes" | "oneHour";
  lockForm: LockForm | null; blocked: Blocked[] | null; blockedLoading: boolean; blockedFailed: boolean;
  wallpaperImage: string | null; customTheme: string | null; themes: [string, string][];
  palette: [string, string][] | null;
  locale: string; language: string | null; languages: [string, string][];
  znga: ZngaSettings;
}
export interface State {
  link: Link; call: CallView | null; me: string | null; meAvatar: string | null; filters: FilterChip[]; filter: string; search: string; chats: ChatRow[]; unreadChats: number; open: Open | null;
  zngaRow: ChatRow | null; znga: ZngaChat | null; offer: OfferView | null;
  section: Section; starred: StarredRow[]; calls: CallRow[]; callFilter: "all" | "missed"; callsSearch: string; info: Info | null; settings: SettingsView;
  lock: LockView | null;
  /** The locked-chats folder behind the rail's Lock plate. */
  locked: { offered: boolean; open: boolean; prompt: { setup: boolean; error: boolean } | null };
}

const empty: State = {
  link: { state: "starting" }, call: null, me: null, meAvatar: null, filters: [], filter: "all", search: "", chats: [], unreadChats: 0, open: null,
  zngaRow: null, znga: null, offer: null,
  section: "chats", starred: [], calls: [], callFilter: "all", callsSearch: "", info: null,
  settings: {
    theme: "system", zoom: 1, showWallpaper: true, enterSends: true, showShortcutHints: true, font: "system", autoDownload: true, historyPrefetch: true,
    keepChatsArchived: true, pauseOtherMedia: true, notifications: true, notificationContent: true, groupSounds: true,
    messageSound: "receive", mentionSound: "alert", sendReadReceipts: true, sendTyping: true, keepRunning: false,
    startWithSystem: null, checkForUpdates: true, downloadUpdates: false, proxy: "", name: null, about: null,
    phone: null, version: "", files: { archive: "", downloads: "", customDownloads: false, log: "", storageHint: "", storageTotal: null, storageRows: [], clearing: false, canClear: false }, wallpaperLight: "theme", wallpaperDark: "theme", swatchesLight: [], swatchesDark: [],
    chatLockCode: false, appLock: false, autoLock: "fifteenMinutes", lockForm: null, blocked: null,
    blockedLoading: false, blockedFailed: false, wallpaperImage: null, customTheme: null, themes: [], palette: null,
    locale: "en", language: null, languages: [],
    znga: {
      enabled: false, engine: "local", engines: [], model: "", endpoint: "", defaultModel: "", defaultEndpoint: "",
      needsKey: false, keySource: null, cloud: false, cloudConsent: false, alerts: true, briefings: true,
      briefingMinMessages: 5, briefingIntervalMinutes: 30, digests: false, digestMorning: "08:00",
      digestEvening: "20:00", reminders: true, quietHours: false, quietFrom: "22:00", quietTo: "07:00",
      selfNotes: false, selfChat: false, includeGroups: true, promises: true, memory: true, checkinDays: 7,
      yourName: "", voiceNotes: "", tone: "", voiceEndpoint: "", voiceModel: "", twin: [], excluded: [],
      facts: [], openPromises: [], audit: [], status: null,
    },
  },
  lock: null,
  locked: { offered: false, open: false, prompt: null },
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
  openUrl: (url: string) => invoke("open_url", { url }),
  startCall: (chat: string, video: boolean) => invoke("start_call", { chat, video }),
  callControl: (control: "answer" | "decline" | "hangup" | "mute" | "camera" | "hide" | "return", on?: boolean) =>
    invoke("call_control", { control, on: on ?? null }),
  callDevice: (kind: "microphone" | "speaker" | "camera", id: string | null) => invoke("call_device", { kind, id }),
  callFullscreen: () => invoke<boolean>("call_fullscreen"),
  saveContact: (id: string, first: string, last: string) => invoke("save_contact", { id, first, last }),
  measureStorage: () => invoke("measure_storage"),
  filesAction: (action: "archive" | "downloads" | "log" | "pick" | "default" | "clear") => invoke("files_action", { action }),
  setCallFilter: (filter: "all" | "missed") => invoke("set_call_filter", { filter }),
  searchCalls: (text: string) => invoke("search_calls", { text }),
  unlockLocked: (code: string) => invoke("unlock_locked", { code }),
  closeDialog: () => invoke("close_dialog"),
  fetchBlocklist: () => invoke("fetch_blocklist"),
  setBlocked: (chat: string, blocked: boolean) => invoke("set_blocked", { chat, blocked }),
  setWallpaperColor: (color: string, dark: boolean) => invoke("set_wallpaper_color", { color, dark }),
  setCustomTheme: (file: string) => invoke("set_custom_theme", { file }),
  themesFolder: (reload: boolean) => invoke("themes_folder", { reload }),
  wallpaperImage: (pick: boolean) => invoke("wallpaper_image", { pick }),
  setLanguage: (code: string | null) => invoke("set_language", { code }),
  // Znga (specs/znga-chat.md).
  zngaSend: (text: string) => invoke("znga_send", { text }),
  zngaStop: () => invoke("znga_stop"),
  zngaCard: (entry: number, index: number, action: string) => invoke("znga_card", { entry, index, action }),
  zngaTool: (chat: string, tool: string, text = "", language = "") => invoke("znga_tool", { chat, tool, text, language }),
  zngaOfferDone: (id: number) => invoke("znga_offer_done", { id }),
  zngaKey: (key: string | null) => invoke("znga_key", { key }),
  zngaTest: () => invoke("znga_test"),
  zngaChatList: (list: "twin" | "excluded", chat: string, on: boolean) => invoke("znga_chat_list", { list, chat, on }),
  zngaFact: (chat: string, text: string, forget: boolean) => invoke("znga_fact", { chat, text, forget }),
  zngaPromise: (chat: string, text: string, done: boolean) => invoke("znga_promise", { chat, text, done }),
  zngaSet: (key: string, value: unknown) => invoke("znga_set", { key, value }),
};
