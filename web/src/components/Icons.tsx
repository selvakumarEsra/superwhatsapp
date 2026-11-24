// ZapFast's own icons, drawn to WhatsApp Desktop's sizes and positions
// (REQ-WEBUI-004.A6): no WhatsApp icon files are used.
import type { JSX } from "solid-js";

type P = { size?: number; class?: string; style?: JSX.CSSProperties };
// `path` is a function: a Solid element is one DOM node, so each icon
// draws its own.
const svg = (path: () => JSX.Element, viewBox = "0 0 24 24") => (p: P) => (
  <svg
    width={p.size ?? 24}
    height={p.size ?? 24}
    viewBox={viewBox}
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    class={p.class}
    style={p.style}
    aria-hidden="true"
  >
    {path()}
  </svg>
);

export const ChatsIcon = svg(() => <path d="M4 18.5V6.5A2.5 2.5 0 0 1 6.5 4h11A2.5 2.5 0 0 1 20 6.5v8a2.5 2.5 0 0 1-2.5 2.5H8l-4 3.5z" />);
export const CallsIcon = svg(() => <path d="M6.6 3.5 9 6.9a1.2 1.2 0 0 1-.1 1.5l-1.4 1.5a12 12 0 0 0 6.6 6.6l1.5-1.4a1.2 1.2 0 0 1 1.5-.1l3.4 2.4a1.2 1.2 0 0 1 .3 1.6l-1 1.6a2.5 2.5 0 0 1-2.6 1.1A17 17 0 0 1 2.3 6.9 2.5 2.5 0 0 1 3.4 4.3l1.6-1a1.2 1.2 0 0 1 1.6.2z" />);
export const SparkleIcon = svg(() => <><path d="M10 3.5l1.6 4.4 4.4 1.6-4.4 1.6L10 15.5l-1.6-4.4L4 9.5l4.4-1.6z" /><path d="M17.5 14l.8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2-2.2-.8 2.2-.8z" /></>);
export const HeartIcon = svg(() => <path d="M12 20s-7.5-4.4-7.5-10A4.3 4.3 0 0 1 12 7.4 4.3 4.3 0 0 1 19.5 10c0 5.6-7.5 10-7.5 10z" />);
export const ChartIcon = svg(() => <><path d="M5 20V12" /><path d="M10 20V5" /><path d="M15 20v-9" /><path d="M20 20v-5" /></>);
export const StarIcon = svg(() => <path d="m12 3.8 2.5 5.1 5.6.8-4 3.9.9 5.6-5-2.6-5 2.6.9-5.6-4-3.9 5.6-.8z" />);
export const ArchiveIcon = svg(() => <><rect x="3.5" y="4.5" width="17" height="4" rx="1" /><path d="M5 8.5v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9" /><path d="M10 12.5h4" /></>);
export const LockIcon = svg(() => <><rect x="5" y="10.5" width="14" height="10" rx="2" /><path d="M8 10.5V8a4 4 0 0 1 8 0v2.5" /></>);
export const SettingsIcon = svg(() => <><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-1.8-.3 1.6 1.6 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.6 1.6 0 0 0-1-1.5 1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.6 1.6 0 0 0 .3-1.8 1.6 1.6 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.6 1.6 0 0 0 1.5-1 1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.6 1.6 0 0 0 1.8.3H9a1.6 1.6 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 1 1.5 1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8V9a1.6 1.6 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1z" /></>);
export const NewChatIcon = svg(() => <><path d="M12 4.5H6.5a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2h11a2 2 0 0 0 2-2V12" /><path d="M17.8 3.7a1.8 1.8 0 0 1 2.5 2.5L12.5 14 9.5 14.5l.5-3z" /></>);
export const MenuIcon = svg(() => <><circle cx="12" cy="5.5" r="1.2" fill="currentColor" /><circle cx="12" cy="12" r="1.2" fill="currentColor" /><circle cx="12" cy="18.5" r="1.2" fill="currentColor" /></>);
export const SearchIcon = svg(() => <><circle cx="11" cy="11" r="6.5" /><path d="m20 20-4.4-4.4" /></>);
export const EmojiIcon = svg(() => <><circle cx="12" cy="12" r="8.5" /><path d="M8.5 14a4 4 0 0 0 7 0" /><circle cx="9.3" cy="10" r=".9" fill="currentColor" /><circle cx="14.7" cy="10" r=".9" fill="currentColor" /></>);
export const PlusIcon = svg(() => <><path d="M12 5v14" /><path d="M5 12h14" /></>);
export const MicIcon = svg(() => <><rect x="9" y="3.5" width="6" height="11" rx="3" /><path d="M5.5 11.5a6.5 6.5 0 0 0 13 0" /><path d="M12 18v2.5" /></>);
export const SendIcon = svg(() => <path d="M4 4.5 21 12 4 19.5l2.5-7.5zM6.5 12H13" />);
export const DotsIcon = svg(() => <><circle cx="5.5" cy="12" r="1.3" fill="currentColor" stroke="none" /><circle cx="12" cy="12" r="1.3" fill="currentColor" stroke="none" /><circle cx="18.5" cy="12" r="1.3" fill="currentColor" stroke="none" /></>);
export const VideoIcon = svg(() => <><rect x="3" y="6.5" width="12.5" height="11" rx="2.5" /><path d="m15.5 10.5 5-3v9l-5-3" /></>);
export const ChevronRightIcon = svg(() => <path d="m9 6 6 6-6 6" />);
export const ArrowUpRightIcon = svg(() => <><path d="M7 17 17 7" /><path d="M9 7h8v8" /></>);
export const BackIcon = svg(() => <><path d="M19 12H5" /><path d="m11 6-6 6 6 6" /></>);
export const ChevronDownIcon = svg(() => <path d="m6 9 6 6 6-6" />);
export const MuteIcon = svg(() => <><path d="M11 5 6.5 9H3.5v6h3L11 19z" /><path d="m16 9.5 5 5" /><path d="m21 9.5-5 5" /></>);
export const PinIcon = svg(() => <><path d="M9 4h6l-1 5 3 3v1.5H7V12l3-3z" /><path d="M12 13.5V20" /></>);

/** Message ticks: one (sent), two (delivered), two blue (read), a clock (pending). */
export function Ticks(p: { status: string; size?: number }) {
  const w = p.size ?? 16;
  if (p.status === "pending")
    return (
      <svg width={w * 0.75} height={w * 0.75} viewBox="0 0 12 12" aria-label="Pending">
        <circle cx="6" cy="6" r="4.6" fill="none" stroke="currentColor" stroke-width="1.1" />
        <path d="M6 3.4V6l1.7 1.1" fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
      </svg>
    );
  if (p.status === "sent")
    return (
      <svg width={w} height={w * 0.69} viewBox="0 0 16 11" aria-label="Sent">
        <path d="M1.5 6.2 4.4 9l6.1-7.4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    );
  if (p.status === "delivered" || p.status === "read")
    return (
      <svg
        width={w}
        height={w * 0.69}
        viewBox="0 0 16 11"
        aria-label={p.status === "read" ? "Read" : "Delivered"}
        style={{ color: p.status === "read" ? "var(--tick-read)" : undefined }}
      >
        <path d="M1 6.2 3.9 9 10 1.6" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M6.8 8.4 7.4 9l6.1-7.4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    );
  return null;
}

/** The silhouette shown without a picture, tinted per chat as WhatsApp
 *  Desktop does. */
export function DefaultAvatar(p: { size: number; group?: boolean; seed?: string }) {
  const tint = () => {
    let h = 0;
    for (const c of p.seed ?? "") h = (h * 31 + c.charCodeAt(0)) >>> 0;
    return p.seed ? h % 5 : 5;
  };
  return (
    <svg width={p.size} height={p.size} viewBox="0 0 212 212" aria-hidden="true">
      <circle cx="106" cy="106" r="106" style={{ fill: `var(--avatar-${tint()}-bg)` }} />
      {p.group ? (
        <g style={{ fill: `var(--avatar-${tint()}-fg)` }}>
          <circle cx="80" cy="88" r="20" />
          <circle cx="134" cy="88" r="20" />
          <path d="M42 146c3-20 18-32 38-32s35 12 38 32z" />
          <path d="M100 146c3-20 18-32 34-32 20 0 35 12 38 32z" />
        </g>
      ) : (
        <g style={{ fill: `var(--avatar-${tint()}-fg)` }}>
          <circle cx="106" cy="84" r="34" />
          <path d="M44 172c6-34 32-54 62-54s56 20 62 54a106 106 0 0 1-124 0z" />
        </g>
      )}
    </svg>
  );
}
