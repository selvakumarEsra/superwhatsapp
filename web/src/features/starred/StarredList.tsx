// The starred messages, newest first (REQ-WEBUI-006: Starred). Each row
// names its chat, shows the message, and opens it in its chat; a right
// click unstars it.
import { createSignal, For, Show } from "solid-js";
import { commands, state, type StarredRow } from "../../lib/bridge";
import { listTime } from "../../lib/time";
import { Menu } from "../../components/Menu";
import { StarIcon } from "../../components/Icons";

function Row(p: { row: StarredRow }) {
  const [menu, setMenu] = createSignal<{ x: number; y: number } | null>(null);
  const author = () => (p.row.msg.fromMe ? "You" : (p.row.msg.sender ?? p.row.chatTitle));
  return (
    <>
      <Show when={menu()}>
        {(at) => (
          <Menu
            x={at().x}
            y={at().y}
            entries={[
              { label: "Go to message", run: () => commands.goToMessage(p.row.chat, p.row.msg.id) },
              { label: "Unstar message", run: () => commands.setStar(p.row.chat, p.row.msg.id, false) },
            ]}
            onClose={() => setMenu(null)}
          />
        )}
      </Show>
      <div
        role="option"
        tabIndex={0}
        onClick={() => commands.goToMessage(p.row.chat, p.row.msg.id)}
        onKeyDown={(e) => e.key === "Enter" && commands.goToMessage(p.row.chat, p.row.msg.id)}
        onContextMenu={(e) => {
          e.preventDefault();
          setMenu({ x: e.clientX, y: e.clientY });
        }}
        class="chat-row cursor-default"
        style={{ margin: "0 var(--row-inset)", padding: "8px var(--row-pad-r) 9px var(--row-pad-l)", "border-radius": "var(--row-radius)" }}
      >
        <div class="flex items-baseline justify-between gap-[6px]" style={{ "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>
          <span class="truncate">
            {author()}
            <Show when={p.row.msg.fromMe || p.row.msg.sender}> ▸ {p.row.chatTitle}</Show>
          </span>
          <span class="shrink-0">{listTime(p.row.msg.time)}</span>
        </div>
        <div
          class="mt-[5px] inline-block max-w-full"
          style={{
            padding: "5px 9px",
            "border-radius": "10px",
            background: p.row.msg.fromMe ? "var(--bubble-out)" : "var(--bubble-in)",
            "font-size": "var(--fs-msg)",
            "line-height": "var(--lh-msg)",
            color: "var(--text-primary)",
          }}
        >
          <span class="line-clamp-4 whitespace-pre-wrap break-words">{p.row.msg.text}</span>
          <span class="ml-[6px] inline-flex items-center gap-[2px] align-bottom" style={{ "font-size": "var(--fs-msg-meta)", color: "var(--text-meta)" }}>
            <StarIcon size={10} />
          </span>
        </div>
      </div>
    </>
  );
}

export default function StarredList() {
  return (
    <Show
      when={state.starred.length > 0}
      fallback={
        <div class="flex flex-col items-center px-[32px] pt-[48px] text-center" style={{ color: "var(--text-secondary)" }}>
          <StarIcon size={40} />
          <p style={{ "font-size": "var(--fs-row-text)", "line-height": "16px", margin: "14px 0 0" }}>
            No starred messages. Right-click a message and choose Star to find it here.
          </p>
        </div>
      }
    >
      <div role="listbox" aria-label="Starred messages">
        <For each={state.starred}>{(row) => <Row row={row} />}</For>
      </div>
    </Show>
  );
}
