// The open chat (REQ-WEBUI-007), as the egui app draws it
// (src/ui/conversation.rs): the picture, name and status line, the search,
// call and menu buttons, the doodle wallpaper in egui's ink, and the
// composer floating over it.
import { createEffect, createSignal, For, on, onCleanup, Show } from "solid-js";
import { commands, state, ZNGA_CHAT, type Open } from "../../lib/bridge";
import { dayKey, dayLabel } from "../../lib/time";
import Avatar from "../../components/Avatar";
import { CallsIcon, DotsIcon, SearchIcon, VideoIcon } from "../../components/Icons";
import Composer from "../composer/Composer";
import Message from "./Message";
import { Menu } from "../../components/Menu";
import { chatMenu } from "../chats/chatMenu";
import { ask } from "../../lib/ui";

function HeaderButton(p: { label: string; onClick?: (e: MouseEvent) => void; children: import("solid-js").JSX.Element }) {
  return (
    <button
      type="button"
      title={p.label}
      aria-label={p.label}
      onClick={(e) => p.onClick?.(e)}
      class="icon-button flex items-center justify-center rounded-full"
      style={{ width: "34px", height: "34px", "margin-left": "4px", color: "var(--icon)" }}
    >
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
  // Opening a chat or a new message keeps the newest in view, unless a
  // jump to an older message is under way.
  createEffect(
    on(
      () => [p.chat.id, p.chat.messages.length],
      () => queueMicrotask(() => {
        if (p.chat.jump) return;
        list.scrollTop = list.scrollHeight;
        pinned = true;
      }),
    ),
  );
  // At the newest message the view stays there while bubbles settle their
  // size and the composer grows, as egui's stick-to-bottom does; scrolled
  // up, it stays where the reader left it.
  let content!: HTMLDivElement;
  let pinned = true;
  const settle = new ResizeObserver(() => {
    if (pinned && !p.chat.jump) list.scrollTop = list.scrollHeight;
  });
  queueMicrotask(() => {
    settle.observe(content);
    settle.observe(list);
  });
  onCleanup(() => settle.disconnect());
  // A jump (a search result, a starred message, a media cell) scrolls its
  // message to the middle and flashes it, as egui's JumpHighlight does. An
  // older message the core is still loading is reached when it arrives.
  const [flash, setFlash] = createSignal<string | null>(null);
  createEffect(
    on(
      () => [p.chat.jump, p.chat.messages.length] as const,
      ([jump]) => {
        if (!jump) return;
        queueMicrotask(() => {
          const target = list.querySelector<HTMLElement>(`[data-msg="${CSS.escape(jump)}"]`);
          if (!target) return;
          target.scrollIntoView({ block: "center" });
          setFlash(null);
          requestAnimationFrame(() => setFlash(jump));
          setTimeout(() => setFlash((now) => (now === jump ? null : now)), 2000);
          void commands.jumped();
        });
      },
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
          height: "var(--list-header-h)",
          "padding-left": "13px",
          "padding-right": "10px",
          background: "var(--conversation-header-bg)",
          "border-bottom": "var(--hairline) solid var(--conversation-header-border)",
        }}
      >
        {/* The picture and name open the info panel, as in egui; the rest
            of the bar moves the window. */}
        <button
          type="button"
          aria-label={p.chat.group ? "Group info" : "Contact info"}
          onClick={() => (state.info?.id === p.chat.id ? commands.closeInfo() : commands.openInfo(p.chat.id))}
          class="flex min-w-0 shrink items-center text-left"
        >
          {/* Measured against egui: the picture 1 and the name 2 higher than
              centring puts them; the status line where it was. */}
          <span class="flex" style={{ "margin-bottom": "2px" }}>
            <Avatar src={p.chat.avatar} size={40} group={p.chat.group} seed={p.chat.id} />
          </span>
          <span class="flex min-w-0 flex-col" style={{ "margin-left": "12px" }}>
            <span class="truncate" style={{ "font-size": "var(--fs-header-title)", "font-weight": 600, "line-height": "16px", color: "var(--text-primary)" }}>
              {p.chat.title}
            </span>
            <Show when={p.chat.subtitle}>
              <span
                class="truncate"
                style={{ "font-size": "var(--fs-header-sub)", "line-height": "17px", "margin-top": "7px", color: p.chat.subtitleAccent ? "var(--accent-text)" : "var(--text-secondary)" }}
              >
                {p.chat.subtitle}
              </span>
            </Show>
          </span>
        </button>
        <div data-tauri-drag-region class="h-full min-w-[12px] flex-1" />
        <HeaderButton label="Search"><SearchIcon size={20} /></HeaderButton>
        {/* The Znga chat is local: nobody to call. */}
        <Show when={p.chat.id !== ZNGA_CHAT}>
          <HeaderButton label="Video call" onClick={() => commands.startCall(p.chat.id, true)}><VideoIcon size={20} /></HeaderButton>
          <HeaderButton label="Voice call" onClick={() => commands.startCall(p.chat.id, false)}><CallsIcon size={20} /></HeaderButton>
        </Show>
        <HeaderButton
          label="Menu"
          onClick={(e) => {
            const box = (e.currentTarget as HTMLElement).getBoundingClientRect();
            setMenu({ x: box.right - 210, y: box.bottom + 6 });
          }}
        >
          <DotsIcon size={20} />
        </HeaderButton>
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
            style={{
              background: "var(--wallpaper-ink)",
              "-webkit-mask-image": "url(/wallpaper.svg)",
              "mask-image": "url(/wallpaper.svg)",
              "-webkit-mask-repeat": "repeat",
              "mask-repeat": "repeat",
            }}
          />
        </Show>
        <div
          ref={list}
          class="scroll relative min-h-0 flex-1"
          style={{ "padding-bottom": "12px" }}
          role="log"
          aria-label="Messages"
          onScroll={() => (pinned = list.scrollHeight - list.scrollTop - list.clientHeight < 4)}
        >
          <div ref={content}>
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
                  <div data-msg={msg.id} classList={{ "jump-flash": flash() === msg.id }}>
                    <Message msg={msg} first={!run(prev(), msg)} last={!run(msg, next())} group={p.chat.group} />
                  </div>
                </>
              );
            }}
          </For>
          </div>
        </div>
        <Composer chat={p.chat.id} znga={p.chat.zngaTools} />
      </div>
    </section>
  );
}
