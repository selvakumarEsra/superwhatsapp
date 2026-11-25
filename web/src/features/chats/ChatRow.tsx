// One chat in the list: an inset rounded row, as WhatsApp Desktop for
// macOS draws it. Under the Favourites heading a row is one line, the name.
// A right click opens the chat's menu (chatMenu.ts).
import { createSignal, Show } from "solid-js";
import { state, type ChatRow as Row } from "../../lib/bridge";
import { ask } from "../../lib/ui";
import { Menu } from "../../components/Menu";
import { chatMenu } from "./chatMenu";
import { listTime } from "../../lib/time";
import Avatar from "../../components/Avatar";
import { MuteIcon, PinIcon, Ticks } from "../../components/Icons";

export default function ChatRow(p: { row: Row; selected: boolean; compact?: boolean; onOpen: () => void }) {
  const unread = () => p.row.unread > 0 || p.row.markedUnread;
  const [menu, setMenu] = createSignal<{ x: number; y: number } | null>(null);
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
        setMenu({ x: e.clientX, y: e.clientY });
      }}
      class="chat-row flex cursor-default items-center"
      classList={{ selected: p.selected }}
      style={{
        height: "var(--row-h)",
        margin: "0 var(--row-inset)",
        "padding-left": "var(--row-pad-l)",
        "border-radius": "var(--row-radius)",
        background: p.selected ? "var(--row-selected)" : undefined,
      }}
    >
      <Avatar src={p.row.avatar} size={40} group={p.row.group} seed={p.row.id} />
      <div
        class="row-text flex h-full min-w-0 flex-1 flex-col justify-center"
        style={{
          "margin-left": "var(--row-gap)",
          "padding-right": "var(--row-pad-r)",
          "border-bottom": p.selected ? "var(--hairline) solid transparent" : "var(--hairline) solid var(--divider)",
        }}
      >
        <div class="flex items-baseline justify-between gap-[6px]">
          <span
            class="truncate"
            style={{ "font-size": "var(--fs-row-title)", "font-weight": "var(--fw-row-title)", "line-height": "15px", color: "var(--text-primary)" }}
          >
            {p.row.title}
          </span>
          <Show when={!p.compact}>
          <span
            class="shrink-0"
            style={{ "font-size": "var(--fs-meta)", "line-height": "14px", color: unread() ? "var(--unread-time)" : "var(--text-secondary)" }}
          >
            {listTime(p.row.time)}
          </span>
          </Show>
        </div>
        <Show when={!p.compact}>
        <div class="mt-[2px] flex items-center justify-between gap-[6px]">
          <span
            class="flex min-w-0 items-center gap-[3px]"
            style={{ "font-size": "var(--fs-row-text)", "line-height": "14px", color: "var(--text-secondary)" }}
          >
            <Show when={p.row.last?.fromMe}>
              <span class="shrink-0" style={{ color: "var(--tick)" }}><Ticks status={p.row.last!.status} size={15} /></span>
            </Show>
            <span class="truncate">
              <Show when={p.row.last?.sender}>{p.row.last!.sender}: </Show>
              {p.row.last?.text ?? ""}
            </span>
          </span>
          <span class="flex shrink-0 items-center gap-[5px]" style={{ color: "var(--text-secondary)" }}>
            <Show when={p.row.muted}><MuteIcon size={14} /></Show>
            <Show when={p.row.pinned}><PinIcon size={14} /></Show>
            <Show when={unread()}>
              <span
                class="flex items-center justify-center rounded-full"
                style={{
                  height: "var(--badge-h)",
                  "min-width": "var(--badge-h)",
                  padding: "0 4px",
                  "font-size": "var(--fs-badge)",
                  "font-weight": 600,
                  background: "var(--unread)",
                  color: "var(--unread-fg)",
                }}
              >
                {p.row.unread > 0 ? p.row.unread : ""}
              </span>
            </Show>
          </span>
        </div>
        </Show>
      </div>
    </div>
    </>
  );
}
