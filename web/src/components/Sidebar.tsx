// The labelled sidebar on the left, as WhatsApp Desktop for macOS draws
// it (REQ-WEBUI-004). The window's traffic lights sit above the first item.
import { createSignal, For, type JSX, Show } from "solid-js";
import { ArchiveIcon, CallsIcon, ChartIcon, ChatsIcon, ChevronDownIcon, ChevronRightIcon, DotsIcon, HeartIcon, SparkleIcon, StarIcon } from "./Icons";
import Avatar from "./Avatar";

export type Section = "chats" | "calls" | "znga" | "favourites" | "insights" | "archived" | "starred";

function Item(p: { label: string; icon: JSX.Element; count?: number; active: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      aria-current={p.active ? "page" : undefined}
      onClick={p.onClick}
      class="sidebar-item flex w-full items-center text-left"
      style={{
        height: "var(--sidebar-item-h)",
        "margin-bottom": "var(--sidebar-item-gap)",
        "padding-left": "9px",
        "padding-right": "10px",
        gap: "8px",
        "border-radius": "var(--sidebar-radius)",
        background: p.active ? "var(--sidebar-selected)" : undefined,
        color: "var(--text-primary)",
      }}
    >
      <span class="flex shrink-0 items-center justify-center" style={{ width: "var(--sidebar-icon)" }}>
        {p.icon}
      </span>
      <span class="flex-1 truncate" style={{ "font-size": "var(--fs-sidebar)" }}>{p.label}</span>
      <Show when={p.count}>
        <span style={{ "font-size": "var(--fs-sidebar-count)", color: "var(--text-secondary)" }}>{p.count}</span>
      </Show>
    </button>
  );
}

function Heading(p: { label: string; open: boolean; onToggle: () => void }) {
  return (
    <button
      type="button"
      aria-expanded={p.open}
      onClick={p.onToggle}
      class="flex w-full items-center justify-between"
      style={{
        height: "22px",
        "margin-top": "14px",
        "padding-left": "9px",
        "padding-right": "10px",
        "font-size": "var(--fs-section)",
        "font-weight": 600,
        color: "var(--text-section)",
      }}
    >
      {p.label}
      <Show when={p.open} fallback={<ChevronRightIcon size={13} />}>
        <ChevronDownIcon size={13} />
      </Show>
    </button>
  );
}

export default function Sidebar(p: { section: Section; onSection: (s: Section) => void; unread: number }) {
  const [moreOpen, setMoreOpen] = createSignal(true);
  const icon = (i: (q: { size?: number }) => JSX.Element) => i({ size: 18 });
  const main: [Section, string, JSX.Element][] = [
    ["chats", "Chats", icon(ChatsIcon)],
    ["calls", "Calls", icon(CallsIcon)],
    ["znga", "Znga", icon(SparkleIcon)],
    ["favourites", "Favourites", icon(HeartIcon)],
    ["insights", "Insights", icon(ChartIcon)],
  ];
  const more: [Section, string, JSX.Element][] = [
    ["archived", "Archived", icon(ArchiveIcon)],
    ["starred", "Starred", icon(StarIcon)],
  ];
  return (
    <nav
      class="flex h-full shrink-0 flex-col"
      style={{ width: "var(--sidebar-w)", background: "var(--sidebar-bg)" }}
      aria-label="Navigation"
    >
      <div data-tauri-drag-region class="shrink-0" style={{ height: "var(--titlebar-h)" }} />
      <div
        class="scroll min-h-0 flex-1"
        style={{ "padding-left": "var(--sidebar-inset-l)", "padding-right": "var(--sidebar-inset-r)" }}
      >
        <For each={main}>
          {([id, label, i]) => (
            <Item label={label} icon={i} count={id === "chats" ? p.unread : undefined} active={p.section === id} onClick={() => p.onSection(id)} />
          )}
        </For>
        <Heading label="More" open={moreOpen()} onToggle={() => setMoreOpen(!moreOpen())} />
        <Show when={moreOpen()}>
          <For each={more}>{([id, label, i]) => <Item label={label} icon={i} active={p.section === id} onClick={() => p.onSection(id)} />}</For>
        </Show>
      </div>
      <div
        class="flex shrink-0 items-center"
        style={{ height: "46px", "padding-left": "16px", "padding-right": "18px", gap: "10px", color: "var(--text-primary)" }}
      >
        <Avatar src={null} size={24} />
        <span class="flex-1 truncate" style={{ "font-size": "var(--fs-sidebar)", "font-weight": 600 }}>You</span>
        <button type="button" aria-label="Settings" class="flex items-center" style={{ color: "var(--icon)" }}>
          <DotsIcon size={18} />
        </button>
      </div>
    </nav>
  );
}
