// The open chat (REQ-WEBUI-007), as WhatsApp Desktop for macOS draws it:
// a slim header with a button pill on the right, the wallpaper, and the
// composer floating over it.
import { createEffect, createSignal, For, on, Show } from "solid-js";
import { commands, state, type Open } from "../../lib/bridge";
import { dayKey, dayLabel } from "../../lib/time";
import Avatar from "../../components/Avatar";
import { CallsIcon, DotsIcon, VideoIcon } from "../../components/Icons";
import Composer from "../composer/Composer";
import Message from "./Message";
import { Menu } from "../../components/Menu";
import { chatMenu } from "../chats/chatMenu";
import { ask } from "../../lib/ui";

function HeaderButton(p: { label: string; onClick?: (e: MouseEvent) => void; children: import("solid-js").JSX.Element }) {
  return (
    <button type="button" aria-label={p.label} onClick={(e) => p.onClick?.(e)} class="flex h-full items-center justify-center" style={{ width: "33px", color: "var(--icon)" }}>
      {p.children}
    </button>
  );
}

export default function Conversation(p: { chat: Open }) {
  let list!: HTMLDivElement;
  const [menu, setMenu] = createSignal<{ x: number; y: number } | null>(null);
  const row = () => state.chats.find((c) => c.id === p.chat.id);
  // Settings › Appearance › Wallpaper colour, for the theme on screen.
  const wallpaper = () => {
    const dark =
      state.settings.theme === "dark" ||
      (state.settings.theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    const id = dark ? state.settings.wallpaperDark : state.settings.wallpaperLight;
    return (dark ? state.settings.swatchesDark : state.settings.swatchesLight).find((s) => s.id === id)?.color ?? null;
  };
  // Opening a chat or a new message keeps the newest in view.
  createEffect(
    on(
      () => [p.chat.id, p.chat.messages.length],
      () => queueMicrotask(() => (list.scrollTop = list.scrollHeight)),
    ),
  );
  const run = (a: Open["messages"][number] | undefined, b: Open["messages"][number] | undefined) =>
    !!a && !!b && dayKey(a.time) === dayKey(b.time) && a.fromMe === b.fromMe && a.senderId === b.senderId;
  return (
    <section class="flex h-full min-w-0 flex-1 flex-col" aria-label={p.chat.title}>
      <header
        data-tauri-drag-region
        class="flex shrink-0 items-center"
        style={{
          height: "var(--conv-header-h)",
          "padding-left": "var(--bubble-gutter)",
          "padding-right": "10px",
          background: "var(--conversation-header-bg)",
          "border-bottom": "var(--hairline) solid var(--conversation-header-border)",
        }}
      >
        {/* The picture and name open the info panel, as on WhatsApp; the
            rest of the bar moves the window. */}
        <button
          type="button"
          aria-label={p.chat.group ? "Group info" : "Contact info"}
          onClick={() => (state.info?.id === p.chat.id ? commands.closeInfo() : commands.openInfo(p.chat.id))}
          class="flex min-w-0 shrink items-center text-left"
        >
          <Avatar src={p.chat.avatar} size={30} group={p.chat.group} seed={p.chat.id} />
          <span
            class="truncate"
            style={{ "margin-left": "7.5px", "font-size": "var(--fs-header-title)", "font-weight": 600, "line-height": "16px", color: "var(--text-primary)" }}
          >
            {p.chat.title}
          </span>
        </button>
        <div data-tauri-drag-region class="h-full min-w-[12px] flex-1" />
        <div
          class="flex items-center"
          style={{ height: "var(--header-pill-h)", padding: "0 2px", "border-radius": "calc(var(--header-pill-h) / 2)", background: "var(--header-pill-bg)" }}
        >
          <HeaderButton label="Video call"><VideoIcon size={18} /></HeaderButton>
          <HeaderButton label="Voice call"><CallsIcon size={16} /></HeaderButton>
          <HeaderButton
            label="Menu"
            onClick={(e) => {
              const box = (e.currentTarget as HTMLElement).getBoundingClientRect();
              setMenu({ x: box.right - 210, y: box.bottom + 6 });
            }}
          >
            <DotsIcon size={17} />
          </HeaderButton>
        </div>
      </header>
      <Show when={menu() && row()}>
        <Menu x={menu()!.x} y={menu()!.y} entries={chatMenu(row()!, ask, state.section === "archived")} onClose={() => setMenu(null)} />
      </Show>
      <div class="relative flex min-h-0 flex-1 flex-col" style={{ background: wallpaper() ?? "var(--conversation-bg)" }}>
        <Show when={state.settings.wallpaperImage}>
          {(image) => (
            <div
              class="pointer-events-none absolute inset-0"
              style={{ "background-image": `url("${image()}")`, "background-size": "cover", "background-position": "center" }}
            />
          )}
        </Show>
        <Show when={state.settings.showWallpaper && !state.settings.wallpaperImage}>
          <div
            class="pointer-events-none absolute inset-0"
            style={{ "background-image": "url(/wallpaper.svg)", opacity: "var(--wallpaper-opacity)", filter: "var(--wallpaper-filter)" }}
          />
        </Show>
        <div ref={list} class="scroll relative min-h-0 flex-1" style={{ "padding-bottom": "8px" }} role="log" aria-label="Messages">
          <For each={p.chat.messages}>
            {(msg, i) => {
              const prev = () => p.chat.messages[i() - 1];
              const next = () => p.chat.messages[i() + 1];
              const newDay = () => !prev() || dayKey(prev()!.time) !== dayKey(msg.time);
              return (
                <>
                  <Show when={newDay()}>
                    <div class="flex justify-center" style={{ margin: "var(--date-chip-gap) 0 calc(var(--date-chip-gap) - var(--bubble-gap-run))" }}>
                      <span
                        class="flex items-center justify-center"
                        style={{
                          height: "var(--date-chip-h)",
                          "min-width": "var(--date-chip-w)",
                          padding: "0 12px",
                          "border-radius": "calc(var(--date-chip-h) / 2)",
                          background: "var(--date-chip-bg)",
                          "box-shadow": "var(--shadow-bubble)",
                          "font-size": "var(--fs-date-chip)",
                          "font-weight": 600,
                          color: "var(--text-primary)",
                        }}
                      >
                        {dayLabel(msg.time)}
                      </span>
                    </div>
                  </Show>
                  <Message msg={msg} first={!run(prev(), msg)} last={!run(msg, next())} group={p.chat.group} />
                </>
              );
            }}
          </For>
        </div>
        <Composer chat={p.chat.id} />
      </div>
    </section>
  );
}
