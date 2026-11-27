// The starred messages, newest first (REQ-WEBUI-006: Starred). Each row
// names its chat, shows the message, and opens it in its chat; a right
// click unstars it.
import { createSignal, For, Show } from "solid-js";
import { commands, state, type StarredRow } from "../../lib/bridge";
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
      {/* chats.rs:starred_row: the chat's name and the moment on one
          line, then the message as its bubble, on its own side, with the
          clock in its corner. */}
      <div
        role="option"
        tabIndex={0}
        onClick={() => commands.goToMessage(p.row.chat, p.row.msg.id)}
        onKeyDown={(e) => e.key === "Enter" && commands.goToMessage(p.row.chat, p.row.msg.id)}
        onContextMenu={(e) => {
          e.preventDefault();
          setMenu({ x: e.clientX, y: e.clientY });
        }}
        class="starred-row relative flex cursor-default flex-col"
        style={{ padding: "0 16px 10px", "border-radius": "6px" }}
        aria-label={`${p.row.chatTitle}, ${p.row.msg.text}`}
      >
        <div class="flex items-center justify-between" style={{ height: "28px", gap: "8px" }}>
          <span class="truncate" style={{ "font-size": "14px", "font-weight": 500, color: "var(--text-primary)" }}>{p.row.chatTitle}</span>
          <span class="shrink-0" style={{ "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>{p.row.moment}</span>
        </div>
        <div
          class="flex items-end"
          style={{
            "align-self": p.row.msg.fromMe ? "flex-end" : "flex-start",
            gap: "6px",
            padding: "6px 10px",
            "border-radius": "10px",
            background: p.row.msg.fromMe ? "var(--bubble-out)" : "var(--bubble-in)",
          }}
        >
          {/* The words wrap beside the clock: the row less the margins,
              the clock and its gap (chats.rs:starred_row). */}
          <span
            class="line-clamp-[12] whitespace-pre-wrap break-words"
            style={{ "max-width": "178px", "font-size": "13px", "line-height": "16px", color: "var(--text-primary)" }}
          >
            {p.row.msg.text}
          </span>
          <span class="shrink-0" style={{ "font-size": "11px", "line-height": "13px", color: "var(--text-secondary)" }}>{p.row.msg.clock}</span>
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
