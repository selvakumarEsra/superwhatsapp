// Settings (REQ-WEBUI-012), laid out as WhatsApp Desktop for macOS lays
// them out: the profile and the sections in the list column, the chosen
// section's rows on the right. Every value is the core's stored one, so
// the egui build and this one share them.
import { createEffect, createSignal, For, type JSX, onMount, Show } from "solid-js";
import { commands, state, type LockForm, type SettingsView } from "../../lib/bridge";
import { ask } from "../../lib/ui";
import Avatar from "../../components/Avatar";
import { ChevronRightIcon } from "../../components/Icons";
import { t } from "../../lib/i18n";

type Key = keyof SettingsView;
type Page = "account" | "appearance" | "chats" | "notifications" | "privacy" | "system" | "about";

const PAGES: [Page, string][] = [
  ["account", "Account"],
  ["appearance", "Appearance"],
  ["chats", "Chats"],
  ["notifications", "Notifications"],
  ["privacy", "Privacy"],
  ["system", "System"],
  ["about", "About"],
];

const set = (key: Key, value: unknown) => void commands.setSetting(key, value);

function Row(p: { title: string; description?: string; children?: JSX.Element }) {
  return (
    <div class="flex items-center justify-between" style={{ "min-height": "46px", padding: "8px 0", gap: "16px", "border-bottom": "var(--hairline) solid var(--divider)" }}>
      <div class="min-w-0">
        <div style={{ "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>{t(p.title)}</div>
        <Show when={p.description}>
          <div style={{ "margin-top": "2px", "font-size": "var(--fs-meta)", "line-height": "14px", color: "var(--text-secondary)" }}>{t(p.description!)}</div>
        </Show>
      </div>
      <div class="shrink-0">{p.children}</div>
    </div>
  );
}

function Toggle(p: { title: string; description?: string; key: Key }) {
  const on = () => state.settings[p.key] === true;
  return (
    <Row title={p.title} description={p.description}>
      <button
        type="button"
        role="switch"
        aria-checked={on()}
        aria-label={p.title}
        onClick={() => set(p.key, !on())}
        class="relative inline-block rounded-full"
        style={{ width: "30px", height: "18px", background: on() ? "var(--accent)" : "var(--switch-off)", transition: "background .15s" }}
      >
        <span class="absolute rounded-full" style={{ top: "2px", left: on() ? "14px" : "2px", width: "14px", height: "14px", background: "#fff", transition: "left .15s" }} />
      </button>
    </Row>
  );
}

function Choice(p: { title: string; description?: string; key: Key; options: [string, string][] }) {
  return (
    <Row title={p.title} description={p.description}>
      <select
        aria-label={p.title}
        value={String(state.settings[p.key])}
        onChange={(e) => set(p.key, e.currentTarget.value)}
        style={{
          height: "26px",
          padding: "0 8px",
          "border-radius": "7px",
          background: "var(--search-bg)",
          color: "var(--text-primary)",
          "font-size": "var(--fs-row-text)",
          border: "none",
        }}
      >
        <For each={p.options}>{([value, label]) => <option value={value}>{t(label)}</option>}</For>
      </select>
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
        style={{ width: "220px", height: "26px", padding: "0 8px", "border-radius": "7px", background: "var(--search-bg)", color: "var(--text-primary)", "font-size": "var(--fs-row-text)" }}
      />
    </Row>
  );
}

function Button(p: { label: string; danger?: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={p.onClick}
      class="rounded-full"
      style={{
        height: "26px",
        padding: "0 12px",
        "font-size": "var(--fs-row-text)",
        background: "var(--round-button-bg)",
        color: p.danger ? "var(--danger)" : "var(--text-primary)",
      }}
    >
      {t(p.label)}
    </button>
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
          description="Time without using ZapFast, also counted while it is in the tray."
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

function Body(p: { page: Page }) {
  return (
    <>
      <Show when={p.page === "account"}>
        <Text title="Your name" key="name" />
        <Text title="About" key="about" />
        <Row title="Phone number">
          <span class="selectable" style={{ "font-size": "var(--fs-row-text)", color: "var(--text-secondary)" }}>{state.settings.phone ?? "—"}</span>
        </Row>
        <Row title="Unlink this computer" description="ZapFast signs out of WhatsApp. Your archive stays on this computer.">
          <Button
            label="Unlink…"
            danger
            onClick={() =>
              ask({
                title: "Unlink this computer?",
                body: "ZapFast will sign out of WhatsApp on this computer. You can link it again with your phone.",
                action: "Unlink",
                run: () => commands.unlink(),
              })
            }
          />
        </Row>
      </Show>
      <Show when={p.page === "appearance"}>
        <Row title="Language">
          <select
            aria-label={t("Language")}
            value={state.settings.language ?? ""}
            onChange={(e) => commands.setLanguage(e.currentTarget.value || null)}
            style={{ height: "26px", padding: "0 8px", "border-radius": "7px", background: "var(--search-bg)", color: "var(--text-primary)", "font-size": "var(--fs-row-text)", border: "none" }}
          >
            <option value="">{t("Follow system")}</option>
            <For each={state.settings.languages}>{([code, name]) => <option value={code}>{name}</option>}</For>
          </select>
        </Row>
        <Row title="Theme" description="Theme files in the themes folder appear here too.">
          <div class="flex items-center" style={{ gap: "6px" }}>
            <select
              aria-label="Theme"
              value={state.settings.customTheme ? `file:${state.settings.customTheme}` : state.settings.theme}
              onChange={(e) => {
                const value = e.currentTarget.value;
                if (value.startsWith("file:")) void commands.setCustomTheme(value.slice(5));
                else set("theme", value);
              }}
              style={{ height: "26px", padding: "0 8px", "border-radius": "7px", background: "var(--search-bg)", color: "var(--text-primary)", "font-size": "var(--fs-row-text)", border: "none" }}
            >
              <option value="system">Follow system</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <For each={state.settings.themes}>{([file, name]) => <option value={`file:${file}`}>{name}</option>}</For>
            </select>
            <Button label="Open themes folder" onClick={() => commands.themesFolder(false)} />
            <Button label="Reload" onClick={() => commands.themesFolder(true)} />
          </div>
        </Row>
        <Row title="Zoom">
          <div class="flex items-center" style={{ gap: "8px" }}>
            <Button label="−" onClick={() => set("zoom", -0.1)} />
            <button type="button" onClick={() => set("zoom", 0)} style={{ width: "44px", "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>
              {Math.round(state.settings.zoom * 100)}%
            </button>
            <Button label="+" onClick={() => set("zoom", 0.1)} />
          </div>
        </Row>
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
        <Toggle title="Enter sends" description="When off, Enter starts a new line and ⌘+Enter sends." key="enterSends" />
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
        <Toggle title="Keep running when the window closes" description="Closing the window leaves ZapFast in the menu bar or tray; quit from there or with ⌘Q." key="keepRunning" />
        <Show when={state.settings.startWithSystem !== null}>
          <Toggle title="Start at login" description="Starts in the tray, without a window." key="startWithSystem" />
        </Show>
        {/* Updates arrive with the Tauri updater (REQ-WEBUI-012.A3); until
            then the egui build keeps those switches. */}
        <Text title="Proxy" description="For WhatsApp, media, and updates. Empty uses ALL_PROXY or HTTPS_PROXY." key="proxy" placeholder="None" />
      </Show>
      <Show when={p.page === "about"}>
        <Row title="ZapFast" description="A fast WhatsApp client, with Znga.">
          <span class="selectable" style={{ "font-size": "var(--fs-row-text)", color: "var(--text-secondary)" }}>{state.settings.version}</span>
        </Row>
      </Show>
    </>
  );
}

export default function Settings() {
  // Outside Tauri, `#settings/<page>` opens that page, for screenshots.
  const linked = location.hash.split("/")[1] as Page | undefined;
  const [page, setPage] = createSignal<Page>(PAGES.some(([id]) => id === linked) ? linked! : "account");
  const title = () => PAGES.find(([id]) => id === page())![1];
  return (
    <>
      <section
        class="flex h-full shrink-0 flex-col"
        aria-label="Settings"
        style={{ width: "var(--list-w)", background: "var(--panel-bg)", "border-right": "var(--hairline) solid var(--panel-border)" }}
      >
        <header data-tauri-drag-region class="flex shrink-0 items-center justify-center" style={{ height: "var(--list-header-h)" }}>
          <h1 data-tauri-drag-region style={{ margin: 0, "font-size": "var(--fs-title)", "font-weight": "var(--fw-title)", color: "var(--text-primary)" }}>
            {t("Settings")}
          </h1>
        </header>
        <div class="scroll min-h-0 flex-1" style={{ padding: "4px var(--row-inset)" }}>
          <button
            type="button"
            onClick={() => setPage("account")}
            class="chat-row flex w-full items-center text-left"
            style={{ height: "64px", padding: "0 10px", gap: "12px", "border-radius": "var(--row-radius)" }}
          >
            <Avatar src={null} size={44} seed={state.me ?? "me"} />
            <div class="min-w-0">
              <div class="truncate" style={{ "font-size": "var(--fs-row-title)", "font-weight": 600, color: "var(--text-primary)" }}>
                {state.settings.name ?? "You"}
              </div>
              <div class="truncate" style={{ "font-size": "var(--fs-row-text)", color: "var(--text-secondary)" }}>{state.settings.about ?? state.settings.phone ?? ""}</div>
            </div>
          </button>
          <div style={{ height: "8px" }} />
          <For each={PAGES.slice(1)}>
            {([id, label]) => (
              <button
                type="button"
                aria-current={page() === id ? "page" : undefined}
                onClick={() => setPage(id)}
                class="chat-row flex w-full items-center justify-between text-left"
                classList={{ selected: page() === id }}
                style={{
                  height: "36px",
                  padding: "0 10px",
                  "border-radius": "var(--row-radius)",
                  background: page() === id ? "var(--row-selected)" : undefined,
                  "font-size": "var(--fs-row-text)",
                  color: "var(--text-primary)",
                }}
              >
                {t(label)}
                <span style={{ color: "var(--text-secondary)" }}><ChevronRightIcon size={14} /></span>
              </button>
            )}
          </For>
        </div>
      </section>
      <section class="flex h-full min-w-0 flex-1 flex-col" style={{ background: "var(--conversation-header-bg)" }} aria-label={title()}>
        <header
          data-tauri-drag-region
          class="flex shrink-0 items-center"
          style={{ height: "var(--conv-header-h)", padding: "0 24px", "border-bottom": "var(--hairline) solid var(--conversation-header-border)" }}
        >
          <h2 data-tauri-drag-region style={{ margin: 0, "font-size": "var(--fs-header-title)", "font-weight": 600, color: "var(--text-primary)" }}>{t(title())}</h2>
        </header>
        <div class="scroll min-h-0 flex-1" style={{ padding: "8px 24px 24px" }}>
          <div style={{ "max-width": "640px" }}>
            <Body page={page()} />
          </div>
        </div>
      </section>
    </>
  );
}
