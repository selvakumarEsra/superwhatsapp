// One chat in the list, as the egui app draws it (src/ui/chats.rs:row): a
// 68-point row, the 48-point picture, the name and time on the first line,
// ticks and the last message (or typing, or a draft) on the second, with
// the pin, mute, mention and unread marks at its end. The open chat is a
// raised card. A right click opens the chat's menu (chatMenu.ts).
import { createSignal, Show } from "solid-js";
import { state, ZNGA_CHAT, type ChatRow as Row } from "../../lib/bridge";
import ZngaAvatar from "../../components/ZngaAvatar";
import { ask } from "../../lib/ui";
import { Menu } from "../../components/Menu";
import { chatMenu } from "./chatMenu";
import Avatar from "../../components/Avatar";
import { MuteIcon, PinIcon, Ticks } from "../../components/Icons";
import { t } from "../../lib/i18n";

export default function ChatRow(p: { row: Row; selected: boolean; onOpen: () => void }) {
  const unread = () => p.row.unread > 0 || p.row.markedUnread;
  const [menu, setMenu] = createSignal<{ x: number; y: number } | null>(null);
  const badge = (fill: string, text: string) => (
    <span
      class="flex items-center justify-center rounded-full"
      style={{
        height: "var(--badge-h)",
        "min-width": "var(--badge-h)",
        padding: "0 6px",
        "font-size": "var(--fs-badge)",
        "font-weight": 600,
        background: fill,
        color: "var(--unread-fg)",
      }}
    >
      {text}
    </span>
  );
  return (
    <>
      <Show when={menu()}>
        {(at) => <Menu x={at().x} y={at().y} entries={chatMenu(p.row, ask, state.section === "archived")} onClose={() => setMenu(null)} />}
      </Show>
      <div
        role="option"
        aria-selected={p.selected}
        tabIndex={0}
        onClick={p.onOpen}
        onKeyDown={(e) => e.key === "Enter" && p.onOpen()}
        onContextMenu={(e) => {
          e.preventDefault();
          // The Znga chat is local: no WhatsApp chat menu.
          if (p.row.id !== ZNGA_CHAT) setMenu({ x: e.clientX, y: e.clientY });
        }}
        class="chat-row relative flex cursor-default items-center"
        classList={{ selected: p.selected }}
        style={{ height: "var(--row-h)" }}
      >
        <div class="row-card absolute" style={{ inset: "2px var(--row-inset)", "border-radius": "var(--row-radius)" }} />
        <div class="relative flex shrink-0 items-center justify-center" style={{ width: "76px" }}>
          <Show when={p.row.id === ZNGA_CHAT} fallback={<Avatar src={p.row.avatar} size={48} group={p.row.group} seed={p.row.id} />}>
            <ZngaAvatar size={48} />
          </Show>
        </div>
        <div class="relative flex h-full min-w-0 flex-1 flex-col" style={{ "padding-right": "var(--row-pad-r)", "padding-top": "13px" }}>
          <div class="flex items-center justify-between" style={{ gap: "8px", height: "20px" }}>
            <span
              class="truncate"
              style={{
                "font-size": "var(--fs-row-title)",
                "font-weight": unread() ? 600 : "var(--fw-row-title)",
                color: "var(--text-primary)",
              }}
            >
              {p.row.title}
            </span>
            <span
              class="shrink-0"
              style={{ "font-size": "var(--fs-meta)", color: unread() && !p.row.muted ? "var(--unread-time)" : "var(--text-meta)" }}
            >
              {p.row.stamp}
            </span>
          </div>
          <div class="flex items-center justify-between" style={{ gap: "6px", height: "18px", "margin-top": "4px" }}>
            <span
              class="flex min-w-0 items-center"
              style={{ gap: "3px", "font-size": "var(--fs-row-text)", color: unread() && !p.row.muted ? "var(--text-secondary)" : "var(--text-meta)" }}
            >
              <Show
                when={p.row.typing}
                fallback={
                  <Show
                    when={p.row.draft}
                    fallback={
                      <>
                        <Show when={p.row.last?.fromMe && p.row.last.status !== "none"}>
                          <span class="shrink-0" style={{ color: "var(--tick)" }}><Ticks status={p.row.last!.status} size={16} /></span>
                        </Show>
                        <span class="truncate">
                          <Show when={p.row.last?.sender}>{p.row.last!.sender}: </Show>
                          {p.row.last?.text ?? ""}
                        </span>
                      </>
                    }
                  >
                    <span class="truncate">
                      <span style={{ color: "var(--accent-text)", "font-weight": 500 }}>{t("Draft:")} </span>
                      {p.row.draft}
                    </span>
                  </Show>
                }
              >
                <span class="truncate" style={{ color: "var(--typing)", "font-weight": 500 }}>{p.row.typing}</span>
              </Show>
            </span>
            <span class="flex shrink-0 items-center" style={{ gap: "6px", color: "var(--text-meta)" }}>
              <Show when={p.row.pinned}><PinIcon size={14} /></Show>
              <Show when={p.row.muted}><MuteIcon size={15} /></Show>
              <Show when={p.row.mention}>{badge("var(--unread)", "@")}</Show>
              <Show when={unread()}>
                <Show
                  when={p.row.unread > 0}
                  fallback={<span class="rounded-full" style={{ width: "10px", height: "10px", background: p.row.muted ? "var(--muted-badge)" : "var(--unread)" }} />}
                >
                  {badge(p.row.muted ? "var(--muted-badge)" : "var(--unread)", p.row.unread > 99 ? "99+" : String(p.row.unread))}
                </Show>
              </Show>
            </span>
          </div>
        </div>
      </div>
    </>
  );
}
