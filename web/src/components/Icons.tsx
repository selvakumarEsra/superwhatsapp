// The icons the egui app draws (src/theme.rs:Icon): Lucide's shapes (ISC
// licence, assets/icons/LICENSE.txt) at stroke 2, plus Znga's sparkle and
// the Insights bars painted as egui paints them (REQ-WEBUI-004.A6). No
// WhatsApp icon files are used.
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
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class={p.class}
    style={p.style}
    aria-hidden="true"
  >
    {path()}
  </svg>
);

export const ChatsIcon = svg(() => <path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z" />);
export const CallsIcon = svg(() => <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" />);
export const SparkleIcon = svg(() => <path d="M10.08 5.28L12.67 11.33L18.72 13.92L12.67 16.51L10.08 22.56L7.49 16.51L1.44 13.92L7.49 11.33ZM19.2 1.44L20.21 3.79L22.56 4.8L20.21 5.81L19.2 8.16L18.19 5.81L15.84 4.8L18.19 3.79Z" fill="currentColor" stroke="none" />);
export const HeartIcon = svg(() => <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z" />);
export const ChartIcon = svg(() => <><rect x="2.9" y="12" width="3.2" height="7" rx="1.2" fill="currentColor" stroke="none" /><rect x="7.9" y="7" width="3.2" height="12" rx="1.2" fill="currentColor" stroke="none" /><rect x="12.9" y="10" width="3.2" height="9" rx="1.2" fill="currentColor" stroke="none" /><rect x="17.9" y="4" width="3.2" height="15" rx="1.2" fill="currentColor" stroke="none" /></>);
export const StarIcon = svg(() => <path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" />);
export const ArchiveIcon = svg(() => <><rect width="20" height="5" x="2" y="3" rx="1" /><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8" /><path d="M10 12h4" /></>);
export const LockIcon = svg(() => <><rect width="18" height="11" x="3" y="11" rx="2" ry="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" /></>);
export const SettingsIcon = svg(() => <><path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915" /><circle cx="12" cy="12" r="3" /></>);
export const NewChatIcon = svg(() => <><path d="M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" /><path d="M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z" /></>);
export const MenuIcon = svg(() => <><circle cx="12" cy="5.5" r="1.2" fill="currentColor" /><circle cx="12" cy="12" r="1.2" fill="currentColor" /><circle cx="12" cy="18.5" r="1.2" fill="currentColor" /></>);
export const SearchIcon = svg(() => <><path d="m21 21-4.34-4.34" /><circle cx="11" cy="11" r="8" /></>);
export const PhoneIncomingIcon = svg(() => <><path d="M16 2v6h6" /><path d="m22 2-6 6" /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" /></>);
export const PhoneOutgoingIcon = svg(() => <><path d="M22 8V2h-6" /><path d="m16 8 6-6" /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" /></>);
export const PhoneMissedIcon = svg(() => <><path d="m16 2 6 6" /><path d="m22 2-6 6" /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" /></>);
export const BellIcon = svg(() => <><path d="M10.268 21a2 2 0 0 0 3.464 0" /><path d="M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" /></>);
export const BellOffIcon = svg(() => <><path d="M8.7 3A6 6 0 0 1 18 8a21.3 21.3 0 0 0 .6 5" /><path d="M17 17H3s3-2 3-9a4.67 4.67 0 0 1 .3-1.7" /><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" /><path d="m2 2 20 20" /></>);
export const BanIcon = svg(() => <><circle cx="12" cy="12" r="10" /><path d="m4.9 4.9 14.2 14.2" /></>);
export const CopyIcon = svg(() => <><rect width="14" height="14" x="8" y="8" rx="2" ry="2" /><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" /></>);
export const TimerIcon = svg(() => <><line x1="10" x2="14" y1="2" y2="2" /><line x1="12" x2="15" y1="14" y2="11" /><circle cx="12" cy="14" r="8" /></>);
export const PinOffIcon = svg(() => <><path d="M12 17v5" /><path d="M15 9.34V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H7.89" /><path d="m2 2 20 20" /><path d="M9 9v1.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h11" /></>);
export const CloseIcon = svg(() => <><path d="M18 6 6 18" /><path d="m6 6 12 12" /></>);
export const MicOffIcon = svg(() => <><path d="M12 19v3" /><path d="M15 9.34V5a3 3 0 0 0-5.68-1.33" /><path d="M16.95 16.95A7 7 0 0 1 5 12v-2" /><path d="M18.89 13.23A7 7 0 0 0 19 12v-2" /><path d="m2 2 20 20" /><path d="M9 9v3a3 3 0 0 0 5.12 2.12" /></>);
export const PhoneOffIcon = svg(() => <><path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-3.33-2.67m-2.67-3.34a19.79 19.79 0 0 1-3.07-8.63A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91" /><line x1="22" x2="2" y1="2" y2="22" /></>);
export const VideoOffIcon = svg(() => <><path d="m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.87a.5.5 0 0 0-.752-.432L16 10.5" /><rect x="2" y="6" width="14" height="12" rx="2" /><path d="m2 2 20 20" /></>);
export const Volume2Icon = svg(() => <><path d="M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z" /><path d="M16 9a5 5 0 0 1 0 6" /><path d="M19.364 18.364a9 9 0 0 0 0-12.728" /></>);
export const MaximizeIcon = svg(() => <><path d="M15 3h6v6" /><path d="m21 3-7 7" /><path d="m3 21 7-7" /><path d="M9 21H3v-6" /></>);
export const MinimizeIcon = svg(() => <><path d="m14 10 7-7" /><path d="M20 10h-6V4" /><path d="m3 21 7-7" /><path d="M4 14h6v6" /></>);
export const UserIcon = svg(() => <><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" /></>);
export const CheckIcon = svg(() => <path d="M20 6 9 17l-5-5" />);
export const PanelIcon = svg(() => <><rect width="18" height="18" x="3" y="3" rx="2" /><path d="M9 3v18" /></>);
export const EmojiIcon = svg(() => <><circle cx="12" cy="12" r="10" /><path d="M8 14s1.5 2 4 2 4-2 4-2" /><line x1="9" x2="9.01" y1="9" y2="9" /><line x1="15" x2="15.01" y1="9" y2="9" /></>);
export const PlusIcon = svg(() => <><path d="M5 12h14" /><path d="M12 5v14" /></>);
export const MicIcon = svg(() => <><path d="M12 19v3" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><rect x="9" y="2" width="6" height="13" rx="3" /></>);
export const SendIcon = svg(() => <><path d="M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z" /><path d="m21.854 2.147-10.94 10.939" /></>);
export const DotsIcon = svg(() => <><circle cx="12" cy="12" r="1" /><circle cx="19" cy="12" r="1" /><circle cx="5" cy="12" r="1" /></>);
export const VideoIcon = svg(() => <><path d="m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.87a.5.5 0 0 0-.752-.432L16 10.5" /><rect x="2" y="6" width="14" height="12" rx="2" /></>);
export const ChevronRightIcon = svg(() => <path d="m9 6 6 6-6 6" />);
export const PhonePlusIcon = svg(() => <><path d="M19 2v6" /><path d="M16 5h6" /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" /></>);
export const ChevronLeftIcon = svg(() => <path d="m15 18-6-6 6-6" />);
export const ImageIcon = svg(() => <><rect width="18" height="18" x="3" y="3" rx="2" ry="2" /><circle cx="9" cy="9" r="2" /><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" /></>);
export const FileTextIcon = svg(() => <><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" /><path d="M14 2v4a2 2 0 0 0 2 2h4" /><path d="M10 9H8" /><path d="M16 13H8" /><path d="M16 17H8" /></>);
export const ExternalLinkIcon = svg(() => <><path d="M15 3h6v6" /><path d="M10 14 21 3" /><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /></>);
export const ArrowUpRightIcon = svg(() => <><path d="M7 17 17 7" /><path d="M9 7h8v8" /></>);
export const BackIcon = svg(() => <><path d="M19 12H5" /><path d="m11 6-6 6 6 6" /></>);
export const ChevronDownIcon = svg(() => <path d="m6 9 6 6 6-6" />);
export const MuteIcon = svg(() => <><path d="M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z" /><line x1="22" x2="16" y1="9" y2="15" /><line x1="16" x2="22" y1="9" y2="15" /></>);
export const PinIcon = svg(() => <><path d="M12 17v5" /><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" /></>);

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
