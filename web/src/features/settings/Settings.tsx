// Settings (REQ-WEBUI-012), laid out as the egui app lays them out
// (src/ui/settings.rs): one page beside the chat list, a 640-point column
// with the back arrow, the title and the search field, then each section
// as a bold heading over a rounded card. Every value is the core's stored
// one, so the egui build and this one share them.
import { createEffect, createSignal, For, type JSX, onMount, Show } from "solid-js";
import { commands, state, type LockForm, type SettingsView } from "../../lib/bridge";
import { ask } from "../../lib/ui";
import Avatar from "../../components/Avatar";
import { ArrowUpRightIcon, BackIcon, ChevronRightIcon, PlusIcon, SearchIcon } from "../../components/Icons";
import { t } from "../../lib/i18n";
import ZngaSettings from "./ZngaSettings";
import { Button, Caret, combo, Row, Switch } from "./parts";

type Key = keyof SettingsView;
type Page = "account" | "appearance" | "chats" | "notifications" | "privacy" | "znga" | "system" | "files" | "about";

/** egui's order (src/ui/settings.rs:sections). */
const PAGES: [Page, string][] = [
  ["appearance", "Appearance"],
  ["chats", "Chats"],
  ["notifications", "Notifications"],
  ["privacy", "Privacy"],
  ["system", "System"],
  ["account", "Account"],
  ["files", "Files"],
  ["about", "About"],
  ["znga", "Znga"],
];

const set = (key: Key, value: unknown) => void commands.setSetting(key, value);

/** Words that find a section in the settings search (REQ-ZCHAT-007.A4). */
const KEYWORDS: Record<Page, string> = {
  account: "account profile name about phone unlink",
  appearance: "appearance theme dark light zoom wallpaper language",
  chats: "chats enter download history archived media lock code",
  notifications: "notifications sound mention alert",
  privacy: "privacy read receipts typing app lock password blocked",
  znga: "znga ai assistant companion engine model ollama claude gemini openai key twin memory promises digest quiet sentinel",
  system: "system background login start proxy",
  files: "files archive downloads folder storage cache images videos stickers log",
  about: "about version",
};
const matches = (page: Page, label: string, query: string) =>
  query.split(/\s+/).every((word) => `${label} ${KEYWORDS[page]}`.toLowerCase().includes(word));

function Toggle(p: { title: string; description?: string; key: Key }) {
  const on = () => state.settings[p.key] === true;
  return (
    <Row title={p.title} description={p.description}>
      <Switch label={p.title} on={on()} onChange={(value) => set(p.key, value)} />
    </Row>
  );
}

/** Where About's links go, as egui's (Cargo.toml repository, AUTHOR_URL). */
const SOURCE_URL = "https://github.com/selvakumarEsra/superwhatsapp";
const AUTHOR_URL = "https://www.linkedin.com/in/selvae/";

/** The website's page on writing a theme (egui's THEMES_GUIDE). */
const THEMES_GUIDE = "https://selvakumaresra.github.io/superwhatsapp/themes/";

/** The Wallpaper row's button, as egui labels it. */
const wallpaperLabel = () => {
  if (state.settings.wallpaperImage) return "Image";
  const dark = darkNow();
  const id = dark ? state.settings.wallpaperDark : state.settings.wallpaperLight;
  const swatch = (dark ? state.settings.swatchesDark : state.settings.swatchesLight).find((s) => s.id === id);
  return swatch?.label ?? "Theme";
};

function Choice(p: { title: string; description?: string; key: Key; options: [string, string][] }) {
  return (
    <Row title={p.title} description={p.description}>
      <span class="relative inline-flex items-center" style={{ width: "200px" }}>
        <select aria-label={p.title} value={String(state.settings[p.key])} onChange={(e) => set(p.key, e.currentTarget.value)} class="w-full" style={combo}>
          <For each={p.options}>{([value, label]) => <option value={value}>{t(label)}</option>}</For>
        </select>
        <Caret />
      </span>
    </Row>
  );
}

/** A text setting saved on Enter or when the field loses focus. */
function Text(p: { title: string; description?: string; key: Key; placeholder?: string }) {
  const save = (value: string) => {
    if (value !== (state.settings[p.key] ?? "")) set(p.key, value);
  };
  return (
    <Row title={p.title} description={p.description}>
      <input
        aria-label={p.title}
        value={String(state.settings[p.key] ?? "")}
        placeholder={p.placeholder}
        onBlur={(e) => save(e.currentTarget.value.trim())}
        onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
        class="selectable outline-none"
        style={{ width: "240px", height: "28px", padding: "0 8px", "border-radius": "6px", background: "var(--input-bg)", border: "1px solid var(--panel-border)", color: "var(--text-primary)", "font-size": "14px" }}
      />
    </Row>
  );
}

/** Whether the page shows the dark theme now. */
const darkNow = () =>
  state.settings.theme === "dark" ||
  (state.settings.theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);

/** The wallpaper colours for the theme on screen, as WhatsApp offers them. */
function Swatches() {
  const dark = () => darkNow();
  const list = () => (dark() ? state.settings.swatchesDark : state.settings.swatchesLight);
  const chosen = () => (dark() ? state.settings.wallpaperDark : state.settings.wallpaperLight);
  return (
    <div style={{ padding: "10px 0 14px", "border-bottom": "var(--hairline) solid var(--divider)" }}>
      <div style={{ "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>Wallpaper colour</div>
      <div style={{ "margin-top": "2px", "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>
        For the {dark() ? "dark" : "light"} theme; the other theme keeps its own.
      </div>
      <div class="flex flex-wrap" style={{ gap: "8px", "margin-top": "10px" }}>
        <For each={list()}>
          {(swatch) => (
            <button
              type="button"
              title={swatch.label}
              aria-label={swatch.label}
              aria-pressed={chosen() === swatch.id}
              onClick={() => commands.setWallpaperColor(swatch.id, dark())}
              class="rounded-[8px]"
              style={{
                width: "34px",
                height: "34px",
                background: swatch.color ?? "var(--conversation-bg)",
                border: chosen() === swatch.id ? "2px solid var(--accent)" : "var(--hairline) solid var(--divider)",
                "box-shadow": chosen() === swatch.id ? "inset 0 0 0 2px var(--panel-bg)" : undefined,
              }}
            />
          )}
        </For>
      </div>
    </div>
  );
}

const field = {
  width: "220px",
  height: "26px",
  padding: "0 8px",
  "border-radius": "7px",
  background: "var(--search-bg)",
  color: "var(--text-primary)",
  "font-size": "var(--fs-row-text)",
} as const;

/** The secret code that opens the Locked chats on this computer. */
function ChatLockCode() {
  const [code, setCode] = createSignal("");
  return (
    <Row title="Locked chats code" description="Opens the Locked tab on this computer. It hides chats, it does not encrypt them.">
      <div class="flex items-center" style={{ gap: "8px" }}>
        <input
          type="password"
          aria-label="Secret code"
          placeholder={state.settings.chatLockCode ? "New secret code" : "Secret code"}
          value={code()}
          onInput={(e) => setCode(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && code().trim()) {
              void commands.setChatLockCode(code());
              setCode("");
            }
          }}
          class="selectable outline-none"
          style={{ ...field, width: "160px" }}
        />
        <Show when={state.settings.chatLockCode}>
          <Button label="Clear" onClick={() => commands.setChatLockCode(null)} />
        </Show>
      </div>
    </Row>
  );
}

const LOCK_ERRORS: Record<NonNullable<LockForm["error"]>, string> = {
  tooShort: "The password needs at least 6 characters.",
  mismatch: "The passwords do not match.",
  wrongCurrent: "The current password is wrong.",
};

/** Sets, changes or turns off the app lock's password. */
function AppLockForm(p: { mode: LockForm["mode"]; onDone: () => void }) {
  const [current, setCurrent] = createSignal("");
  const [next, setNext] = createSignal("");
  const [confirm, setConfirm] = createSignal("");
  const [seen, setSeen] = createSignal(false);
  // The core holds the form while it checks; it drops it once done.
  createEffect(() => {
    const form = state.settings.lockForm;
    if (form) setSeen(true);
    else if (seen()) p.onDone();
  });
  const input = (label: string, value: () => string, set: (v: string) => void) => (
    <input
      type="password"
      aria-label={t(label)}
      placeholder={t(label)}
      value={value()}
      onInput={(e) => set(e.currentTarget.value)}
      class="selectable outline-none"
      style={field}
    />
  );
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void commands.appLockForm(p.mode, current(), next(), confirm());
      }}
      class="flex flex-col items-end"
      style={{ gap: "6px", padding: "10px 0", "border-bottom": "var(--hairline) solid var(--divider)" }}
    >
      <Show when={p.mode !== "set"}>{input("Current password", current, setCurrent)}</Show>
      <Show when={p.mode !== "turnOff"}>
        {input("New password", next, setNext)}
        {input("Confirm password", confirm, setConfirm)}
      </Show>
      <Show when={state.settings.lockForm?.error}>
        {(error) => <span style={{ "font-size": "var(--fs-meta)", color: "var(--danger)" }}>{LOCK_ERRORS[error()]}</span>}
      </Show>
      <div class="flex" style={{ gap: "8px" }}>
        <Button
          label="Cancel"
          onClick={() => {
            void commands.appLockForm(null);
            p.onDone();
          }}
        />
        <button
          type="submit"
          disabled={state.settings.lockForm?.busy}
          class="rounded-full"
          style={{ height: "26px", padding: "0 12px", "font-size": "var(--fs-row-text)", "font-weight": 600, background: "var(--accent)", color: "var(--accent-fg)" }}
        >
          {p.mode === "turnOff" ? "Turn off" : "Save"}
        </button>
      </div>
    </form>
  );
}

function AppLock() {
  const [mode, setMode] = createSignal<LockForm["mode"] | null>(null);
  return (
    <>
      <Row
        title="App lock"
        description="Asks for a password at start and after a while unused. It keeps people using this computer out of your chats and encrypts nothing more. A forgotten password means unlinking."
      >
        <Show when={state.settings.appLock} fallback={<Button label="Set password…" onClick={() => setMode("set")} />}>
          <div class="flex" style={{ gap: "6px" }}>
            <Button label="Change password…" onClick={() => setMode("change")} />
            <Button label="Turn off…" danger onClick={() => setMode("turnOff")} />
          </div>
        </Show>
      </Row>
      <Show when={mode()}>{(m) => <AppLockForm mode={m()} onDone={() => setMode(null)} />}</Show>
      <Show when={state.settings.appLock}>
        <Choice
          title="Lock after"
          description="Time without using SuperWhatsapp, also counted while it is in the tray."
          key="autoLock"
          options={[["oneMinute", "1 minute"], ["fifteenMinutes", "15 minutes"], ["oneHour", "1 hour"]]}
        />
        <Row title="Lock now">
          <Button label="Lock" onClick={() => commands.lockNow()} />
        </Row>
      </Show>
    </>
  );
}

/** Blocked contacts, read from WhatsApp when the page opens. */
function BlockedContacts() {
  onMount(() => void commands.fetchBlocklist());
  return (
    <div style={{ padding: "10px 0", "border-bottom": "var(--hairline) solid var(--divider)" }}>
      <div style={{ "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>Blocked contacts</div>
      <Show
        when={state.settings.blocked}
        fallback={
          <div style={{ "margin-top": "4px", "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>
            {state.settings.blockedFailed ? "Could not load the list. It loads when connected." : "Loading…"}
          </div>
        }
      >
        {(blocked) => (
          <Show
            when={blocked().length > 0}
            fallback={<div style={{ "margin-top": "4px", "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>No blocked contacts.</div>}
          >
            <For each={blocked()}>
              {(row) => (
                <div class="flex items-center" style={{ height: "44px", gap: "10px" }}>
                  <Avatar src={row.avatar} size={30} seed={row.id} />
                  <span class="flex-1 truncate" style={{ "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>{row.title}</span>
                  <Button
                    label="Unblock"
                    onClick={() =>
                      ask({
                        title: `Unblock ${row.title}?`,
                        body: "They will be able to call you and send you messages.",
                        action: "Unblock",
                        run: () => commands.setBlocked(row.id, false),
                      })
                    }
                  />
                </div>
              )}
            </For>
          </Show>
        )}
      </Show>
    </div>
  );
}

const SOUNDS: [string, string][] = [
  ["receive", "Message"],
  ["alert", "Alert"],
  ["system", "System sound"],
  ["none", "None"],
];

/** Settings › Files, as egui lays it out (src/ui/settings.rs): the
 *  archive, downloads and log with their folders, and the media cache's
 *  size by kind with Clear…, measured again each time Settings opens. */
function Files() {
  const f = () => state.settings.files;
  onMount(() => void commands.measureStorage());
  return (
    <>
      <Row title="Message archive" description={f().archive} top>
        <Button label="Open folder" icon={<ArrowUpRightIcon size={15} />} onClick={() => commands.filesAction("archive")} />
      </Row>
      <Row title="Downloads" description={f().downloads} top>
        <div class="flex flex-wrap justify-end" style={{ gap: "6px" }}>
          <Button label="Open folder" icon={<ArrowUpRightIcon size={15} />} onClick={() => commands.filesAction("downloads")} />
          <Button label="Change…" onClick={() => commands.filesAction("pick")} />
          <Show when={f().customDownloads}>
            <Button label="Use default" onClick={() => commands.filesAction("default")} />
          </Show>
        </div>
      </Row>
      {/* storage_usage: the total and Clear…, the hint, then by kind. */}
      <div style={{ "padding-bottom": "18px" }}>
        <div class="flex items-center justify-between">
          <span style={{ "font-size": "14px", "line-height": "18px", "font-weight": 500, color: "var(--text-primary)" }}>{t("Storage used")}</span>
          <span class="flex items-center" style={{ gap: "10px" }}>
            <Show when={f().storageTotal}>
              <span style={{ "font-size": "14px", "font-weight": 600, color: "var(--text-primary)" }}>{f().storageTotal}</span>
            </Show>
            <Button
              label="Clear…"
              disabled={!f().canClear}
              onClick={() =>
                ask({
                  title: t("Clear the media cache?"),
                  body: f().storageHint,
                  action: t("Clear"),
                  run: () => commands.filesAction("clear"),
                })
              }
            />
          </span>
        </div>
        <p style={{ margin: "2px 0 0", "font-size": "12.5px", "line-height": "15px", color: "var(--text-secondary)" }}>{f().storageHint}</p>
        <Show when={f().storageRows.length > 0}>
          <div style={{ "margin-top": "10px" }}>
            <For each={f().storageRows}>
              {([label, detail, share]) => (
                <div style={{ "margin-bottom": "8px" }}>
                  <div class="flex items-center justify-between" style={{ height: "20px" }}>
                    <span style={{ "font-size": "14px", "font-weight": 500, color: "var(--text-primary)" }}>{label}</span>
                    <span style={{ "font-size": "12.5px", color: "var(--text-secondary)" }}>{detail}</span>
                  </div>
                  <div class="overflow-hidden rounded-full" style={{ height: "6px", background: "var(--panel-border)" }}>
                    <div class="h-full rounded-full" style={{ width: `${Math.round(share * 1000) / 10}%`, background: "var(--accent)" }} />
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
      <Row title="Log" description={f().log} top>
        <Button label="Open" onClick={() => commands.filesAction("log")} />
      </Row>
    </>
  );
}

function Body(p: { page: Page | "wallpaper"; onWallpaper?: () => void }) {
  return (
    <>
      <Show when={p.page === "account"}>
        <Text title="Your name" key="name" />
        <Text title="About" key="about" />
        <Row title="Phone number">
          <span class="selectable" style={{ "font-size": "var(--fs-row-text)", color: "var(--text-secondary)" }}>{state.settings.phone ?? "—"}</span>
        </Row>
        <Row title="Unlink this computer" description="SuperWhatsapp signs out of WhatsApp. Your archive stays on this computer.">
          <Button
            label="Unlink…"
            danger
            onClick={() =>
              ask({
                title: "Unlink this computer?",
                body: "SuperWhatsapp will sign out of WhatsApp on this computer. You can link it again with your phone.",
                action: "Unlink",
                run: () => commands.unlink(),
              })
            }
          />
        </Row>
      </Show>
      <Show when={p.page === "appearance"}>
        {/* egui's order: Theme, Font, Wallpaper, Zoom, Language. */}
        <Row title="Theme" top>
          <span class="relative inline-flex items-center" style={{ width: "200px" }}>
            <select
              aria-label="Theme"
              value={state.settings.customTheme ? `file:${state.settings.customTheme}` : state.settings.theme}
              onChange={(e) => {
                const value = e.currentTarget.value;
                if (value.startsWith("file:")) void commands.setCustomTheme(value.slice(5));
                else set("theme", value);
              }}
              class="w-full"
              style={combo}
            >
              <option value="system">{t("Follow system")}</option>
              <option value="light">{t("Light")}</option>
              <option value="dark">{t("Dark")}</option>
              <For each={state.settings.themes}>{([file, name]) => <option value={`file:${file}`}>{name}</option>}</For>
            </select>
            <Caret />
          </span>
          <Button label="Open themes folder" icon={<ArrowUpRightIcon size={15} />} onClick={() => commands.themesFolder(false)} />
          <Button label="How to make a theme" icon={<ArrowUpRightIcon size={15} />} onClick={() => commands.openUrl(THEMES_GUIDE)} />
        </Row>
        <Choice
          title="Font"
          description="System is your desktop's interface font. Inter looks the same on every computer."
          key="font"
          options={[["system", "System"], ["inter", "Inter"]]}
        />
        <Row title="Wallpaper">
          <Button label={wallpaperLabel()} icon={<ChevronRightIcon size={15} />} onClick={() => p.onWallpaper?.()} />
        </Row>
        <Row title="Zoom" description="⌘+Plus and ⌘+Minus work anywhere, and ⌘+0 resets it.">
          <div class="flex items-center" style={{ gap: "10px", color: "var(--text-secondary)" }}>
            <button type="button" title={t("Smaller")} aria-label={t("Smaller")} onClick={() => set("zoom", -0.1)} class="icon-button rounded-full" style={{ padding: "3px" }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M5 12h14" /></svg>
            </button>
            <button type="button" title={t("Reset")} onClick={() => set("zoom", 0)} style={{ "min-width": "40px", "font-size": "13.5px", "font-weight": 500, color: "var(--text-primary)" }}>
              {Math.round(state.settings.zoom * 100)}%
            </button>
            <button type="button" title={t("Larger")} aria-label={t("Larger")} onClick={() => set("zoom", 0.1)} class="icon-button rounded-full" style={{ padding: "3px" }}>
              <PlusIcon size={16} />
            </button>
          </div>
        </Row>
        <Row title="Language">
          <span class="relative inline-flex items-center" style={{ width: "200px" }}>
            <select
              aria-label={t("Language")}
              value={state.settings.language ?? ""}
              onChange={(e) => commands.setLanguage(e.currentTarget.value || null)}
              class="w-full"
              style={combo}
            >
              <option value="">{t("Auto")}</option>
              <For each={state.settings.languages}>{([code, name]) => <option value={code}>{name}</option>}</For>
            </select>
            <Caret />
          </span>
        </Row>
      </Show>
      <Show when={p.page === "wallpaper"}>
        <Swatches />
        <Row title="Wallpaper image" description="Shown behind messages instead of the colour and doodles.">
          <div class="flex" style={{ gap: "6px" }}>
            <Button label={state.settings.wallpaperImage ? "Change…" : "Choose image…"} onClick={() => commands.wallpaperImage(true)} />
            <Show when={state.settings.wallpaperImage}>
              <Button label="Remove image" danger onClick={() => commands.wallpaperImage(false)} />
            </Show>
          </div>
        </Row>
        <Toggle title="Chat wallpaper doodles" description="The pattern behind messages." key="showWallpaper" />
      </Show>
      <Show when={p.page === "chats"}>
        <Toggle title="Enter sends" description="When off, ⌘+Enter sends." key="enterSends" />
        <Toggle title="Download files automatically" description="Files up to 64 MiB download as they come into view." key="autoDownload" />
        <Toggle title="Fetch older history" description="Asks your phone in the background for older messages in the open, pinned and ten most recent chats." key="historyPrefetch" />
        <Toggle title="Keep chats archived" description="When off, a new message brings an archived chat back to the list." key="keepChatsArchived" />
        <Toggle title="Pause other media while recording or playing" description="Music and videos in other apps resume afterwards." key="pauseOtherMedia" />
        <ChatLockCode />
      </Show>
      <Show when={p.page === "notifications"}>
        <Toggle title="Desktop notifications" key="notifications" />
        <Show when={state.settings.notifications}>
          <Toggle title="Show message content in notifications" description="When off, they say only “New message”." key="notificationContent" />
          <Choice title="Message sound" key="messageSound" options={SOUNDS} />
          <Choice title="Mention sound" description="When a group mentions you or replies to you." key="mentionSound" options={SOUNDS} />
          <Toggle title="Play sounds for group messages" key="groupSounds" />
        </Show>
      </Show>
      <Show when={p.page === "privacy"}>
        <Toggle title="Send read receipts" description="Let people see when you read their messages." key="sendReadReceipts" />
        <Toggle title="Show when you are typing" key="sendTyping" />
        <AppLock />
        <BlockedContacts />
      </Show>
      <Show when={p.page === "system"}>
        <Toggle title="Keep running when the window closes" description="Closing the window leaves SuperWhatsapp in the menu bar or tray; quit from there or with ⌘Q." key="keepRunning" />
        <Show when={state.settings.startWithSystem !== null}>
          <Toggle title="Start at login" description="Starts in the tray, without a window." key="startWithSystem" />
        </Show>
        {/* Updates arrive with the Tauri updater (REQ-WEBUI-012.A3); until
            then the egui build keeps those switches. */}
        <Text title="Proxy" description="For WhatsApp, media, and updates. Empty uses ALL_PROXY or HTTPS_PROXY." key="proxy" placeholder="None" />
      </Show>
      <Show when={p.page === "znga"}>
        <ZngaSettings />
      </Show>
      <Show when={p.page === "files"}>
        <Files />
      </Show>
      <Show when={p.page === "about"}>
        {/* src/ui/settings.rs:about: the mark, name and version, what it is
            built with, the links as soft pills, and the credit. */}
        <div style={{ "padding-bottom": "16px" }}>
          <div class="flex items-center" style={{ gap: "14px" }}>
            <img src="/mark.png" alt="" width="44" height="44" style={{ width: "44px", height: "44px" }} />
            <div>
              <div class="selectable" style={{ "font-size": "16px", "font-weight": 600, color: "var(--text-primary)" }}>
                SuperWhatsapp {state.settings.version}
              </div>
              <div style={{ "font-size": "13px", "line-height": "17px", color: "var(--text-secondary)" }}>
                {t("A native WhatsApp client built with Rust, Tauri, Solid.js, and whatsapp-rust.")}
              </div>
            </div>
          </div>
          <div class="flex flex-wrap" style={{ gap: "8px", "margin-top": "12px" }}>
            <Button label="Source code" icon={<ArrowUpRightIcon size={15} />} onClick={() => commands.openUrl(SOURCE_URL)} />
          </div>
          <div style={{ "margin-top": "14px", "font-size": "13px", color: "var(--text-secondary)" }}>
            <span style={{ color: "var(--danger)" }}>{"\u2665"}&nbsp;&nbsp;</span>
            {t("Built with love by")}{" "}
            <a
              href={AUTHOR_URL}
              title={AUTHOR_URL}
              onClick={(e) => { e.preventDefault(); void commands.openUrl(AUTHOR_URL); }}
              style={{ "font-weight": 500, color: "var(--link)", "text-decoration": "none" }}
            >
              Selvakumar Esra
            </a>
          </div>
        </div>
      </Show>
    </>
  );
}

export default function Settings(p: { onClose: () => void }) {
  const [query, setQuery] = createSignal("");
  // egui's Wallpaper page, reached from Appearance › Wallpaper.
  const [wallpaper, setWallpaper] = createSignal(location.hash === "#settings/wallpaper");
  const shown = () => {
    const q = query().trim().toLowerCase();
    return PAGES.filter(([id, label]) => !q || matches(id, label, q));
  };
  return (
    <section
      class="scroll h-full min-w-0 flex-1"
      aria-label="Settings"
      style={{ background: "var(--panel-bg)" }}
      onKeyDown={(e) => e.key === "Escape" && p.onClose()}
    >
      <div style={{ width: "min(640px, calc(100% - 64px))", margin: "0 auto", padding: "24px 0" }}>
        <div data-tauri-drag-region class="flex items-center" style={{ gap: "8px", height: "34px" }}>
          <button
            type="button"
            title={t("Back (Esc)")}
            aria-label={t("Back (Esc)")}
            onClick={() => (wallpaper() ? setWallpaper(false) : p.onClose())}
            class="icon-button flex items-center justify-center rounded-full"
            style={{ width: "30px", height: "30px", "margin-left": "-6px", color: "var(--text-secondary)" }}
          >
            <BackIcon size={20} />
          </button>
          <h1 style={{ margin: 0, "font-size": "24px", "font-weight": 700, color: "var(--text-primary)" }}>{t(wallpaper() ? "Wallpaper" : "Settings")}</h1>
        </div>
        <Show when={wallpaper()}>
          <div style={{ "margin-top": "20px", background: "var(--card-bg)", border: "1px solid var(--panel-border)", "border-radius": "10px", padding: "16px 20px 6px" }}>
            <Body page="wallpaper" />
          </div>
        </Show>
        <Show when={!wallpaper()}>
        <label
          class="flex items-center"
          style={{ height: "var(--search-h)", "margin-top": "18px", "border-radius": "calc(var(--search-h) / 2)", background: "var(--search-bg)", "padding-left": "11px" }}
        >
          <span style={{ color: "var(--icon)" }}><SearchIcon size={17} /></span>
          <input
            type="search"
            aria-label="Search settings"
            placeholder={t("Search settings")}
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
            class="selectable h-full min-w-0 flex-1 bg-transparent outline-none"
            style={{ "padding-left": "9px", "padding-right": "12px", "font-size": "var(--fs-search)", color: "var(--text-primary)" }}
          />
        </label>
        <For each={shown()} fallback={
          <p style={{ margin: "24px 0 0", "font-size": "14px", color: "var(--text-secondary)" }}>
            {t("No settings match “{query}”.").replace("{query}", query().trim())}
          </p>
        }>
          {([id, label], i) => (
            // Measured against egui: a heading after a card sits 2 lower
            // and 2 nearer its own card than the first one does.
            <section aria-label={t(label)} style={{ margin: i() === 0 ? "24px 0 0" : "26px 0 0" }}>
              <h2 style={{ margin: i() === 0 ? "0 0 14px" : "0 0 12px", "font-size": "18px", "line-height": "22px", "font-weight": 700, color: "var(--text-primary)" }}>{t(label)}</h2>
              <div style={{ background: "var(--card-bg)", border: "1px solid var(--panel-border)", "border-radius": "10px", padding: "16px 20px 6px" }}>
                <Body page={id} onWallpaper={() => setWallpaper(true)} />
              </div>
            </section>
          )}
        </For>
        </Show>
      </div>
    </section>
  );
}
