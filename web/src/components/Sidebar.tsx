// The navigation rail, as the egui app draws it (src/ui/rail.rs,
// REQ-WEBUI-004): a narrow column of icons, the lit one on a raised plate.
// Chats, Calls, Znga, Favourites and Insights at the top; Starred,
// Archived, Locked (while chats are locked) and Settings at the bottom over your own picture. On macOS the
// window's traffic lights sit above the first icon.
import { For, type JSX } from "solid-js";
import { ArchiveIcon, CallsIcon, ChartIcon, ChatsIcon, HeartIcon, LockIcon, SettingsIcon, SparkleIcon, StarIcon } from "./Icons";
import Avatar from "./Avatar";
import { t } from "../lib/i18n";

/** The core's destinations plus the panels drawn here (Znga, Insights). */
export type Section = "chats" | "calls" | "znga" | "favourites" | "insights" | "archived" | "starred" | "locked";

function Plate(p: { label: string; icon: JSX.Element; active: boolean; badge?: number; cell?: string; onClick: () => void }) {
  return (
    <div class="flex shrink-0 items-center justify-center" style={{ width: "var(--rail-w)", height: p.cell ?? "var(--rail-cell)" }}>
      <button
        type="button"
        title={t(p.label)}
        aria-label={t(p.label)}
        aria-current={p.active ? "page" : undefined}
        onClick={p.onClick}
        class="rail-plate relative flex items-center justify-center"
        classList={{ lit: p.active }}
        style={{
          width: "var(--rail-plate)",
          height: "var(--rail-plate)",
          "border-radius": "var(--sidebar-radius)",
          color: p.active ? "var(--icon-active)" : "var(--icon)",
        }}
      >
        {p.icon}
        {p.badge ? (
          <span
            class="absolute flex items-center justify-center rounded-full"
            style={{
              top: "0px",
              right: "-2px",
              height: "16px",
              "min-width": "16px",
              padding: "0 4px",
              "font-size": "10px",
              "font-weight": 600,
              background: "var(--unread)",
              color: "var(--unread-fg)",
            }}
          >
            {p.badge > 99 ? "99+" : p.badge}
          </span>
        ) : null}
      </button>
    </div>
  );
}

/** The lower group's cells: a plate and a gap, as rail.rs stacks them. */
const BOTTOM_CELL = "calc(var(--rail-plate) + var(--rail-gap))";

export default function Sidebar(p: {
  section: Section | "settings";
  onSection: (s: Section | "settings") => void;
  unread: number;
  avatar?: string | null;
  /** egui offers Locked only while chats are locked. */
  locked?: boolean;
}) {
  const icon = (i: (q: { size?: number }) => JSX.Element) => i({ size: 20 });
  const top: [Section, string, JSX.Element][] = [
    ["chats", "Chats", icon(ChatsIcon)],
    ["calls", "Calls", icon(CallsIcon)],
    ["znga", "Znga", icon(SparkleIcon)],
    ["favourites", "Favourites", icon(HeartIcon)],
    ["insights", "Insights", icon(ChartIcon)],
  ];
  const bottom = (): [Section, string, JSX.Element][] => [
    ["starred", "Starred", icon(StarIcon)],
    ["archived", "Archived", icon(ArchiveIcon)],
    ...(p.locked ? ([["locked", "Locked chats", icon(LockIcon)]] as [Section, string, JSX.Element][]) : []),
  ];
  return (
    <nav
      class="flex h-full shrink-0 flex-col"
      style={{ width: "var(--rail-w)", background: "var(--sidebar-bg)" }}
      aria-label="Navigation"
    >
      {/* The traffic lights' strip: it moves the window. */}
      <div data-tauri-drag-region class="shrink-0" style={{ height: "42px" }} />
      <For each={top}>
        {([id, label, i]) => (
          <Plate label={label} icon={i} active={p.section === id} onClick={() => p.onSection(id)} />
        )}
      </For>
      <div data-tauri-drag-region class="min-h-0 flex-1" />
      <For each={bottom()}>
        {([id, label, i]) => (
          <Plate label={label} icon={i} active={p.section === id} cell={BOTTOM_CELL} onClick={() => p.onSection(id)} />
        )}
      </For>
      <Plate label="Settings" icon={icon(SettingsIcon)} active={p.section === "settings"} cell={BOTTOM_CELL} onClick={() => p.onSection("settings")} />
      <div class="flex shrink-0 items-center justify-center" style={{ height: "49px", "margin-top": "-4.5px" }}>
        <button type="button" title={t("Profile")} aria-label={t("Profile")} onClick={() => p.onSection("settings")} class="rounded-full">
          <Avatar src={p.avatar ?? null} size={32} />
        </button>
      </div>
    </nav>
  );
}
