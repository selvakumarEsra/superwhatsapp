// One message bubble, as the egui app draws it in WhatsApp's style
// (src/ui/widgets.rs:bubble_shape, src/ui/conversation.rs): radius 10, the
// first bubble of a run with a tail at its top corner, pictures, documents,
// voice notes and link previews drawn in place, WhatsApp's *bold* _italic_
// ~strike~ formatting, and the reactions in a pill under the bubble. Text is
// always text, never markup (REQ-WEBUI-003.A4).
import { For, Show, type JSX } from "solid-js";
import type { MediaView, Msg, PreviewView } from "../../lib/bridge";
import { Ticks } from "../../components/Icons";

/** The tail at the top corner of a run's first bubble: 8 wide, 11 tall. */
function Tail(p: { out: boolean }) {
  return (
    <svg
      width="11"
      height="15"
      viewBox="0 0 11 15"
      class="absolute top-0"
      style={{ [p.out ? "right" : "left"]: "-8px", color: p.out ? "var(--bubble-out)" : "var(--bubble-in)" }}
      aria-hidden="true"
    >
      {p.out ? <path d="M0 0H11L0 15z" fill="currentColor" /> : <path d="M11 0H0L11 15z" fill="currentColor" />}
    </svg>
  );
}

// egui's Palette::sender: the sender's hue at its light or dark saturation
// and lightness (tokens.css --sender-s / --sender-l).
const senderColor = (hue: number) => `hsl(${hue} var(--sender-s) var(--sender-l))`;

/** A size as egui writes it: 470.8 KB. */
export function bytes(size: number): string {
  if (size < 1024) return `${size} B`;
  const units = ["KB", "MB", "GB"];
  let value = size / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(1)} ${units[unit]}`;
}

const duration = (seconds: number) => `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;

const LINK = /https?:\/\/[^\s<>"]+[^\s<>".,;:!?)\]'"]/g;
const MARKS: [string, JSX.CSSProperties][] = [
  ["*", { "font-weight": 700 }],
  ["_", { "font-style": "italic" }],
  ["~", { "text-decoration": "line-through" }],
];

/** WhatsApp's formatting: *bold*, _italic_, ~strike~ and `code`, each
 *  around a word boundary, as egui's markup module reads it. */
function formatted(text: string): JSX.Element[] {
  const out: JSX.Element[] = [];
  const pattern = /([*_~])(?=\S)([^\n]*?\S)\1(?![\w*_~])|`([^`\n]+)`/g;
  let at = 0;
  for (const m of text.matchAll(pattern)) {
    const start = m.index ?? 0;
    // Formatting starts a word, never inside one (2*3*4 stays as it is).
    if (start > 0 && /\w/.test(text[start - 1])) continue;
    if (start > at) out.push(...linked(text.slice(at, start)));
    if (m[3] !== undefined) {
      out.push(<code style={{ "font-family": "ui-monospace, Menlo, monospace", "font-size": "0.92em" }}>{m[3]}</code>);
    } else {
      const style = MARKS.find(([mark]) => mark === m[1])![1];
      out.push(<span style={style}>{formatted(m[2])}</span>);
    }
    at = start + m[0].length;
  }
  if (at < text.length) out.push(...linked(text.slice(at)));
  return out;
}

function linked(text: string): JSX.Element[] {
  const out: JSX.Element[] = [];
  let at = 0;
  for (const m of text.matchAll(LINK)) {
    const start = m.index ?? 0;
    if (start > at) out.push(text.slice(at, start));
    const url = m[0];
    out.push(
      <a
        href={url}
        target="_blank"
        rel="noreferrer noopener"
        style={{ color: "var(--link)", "text-decoration": "underline" }}
        onClick={(e) => e.stopPropagation()}
      >
        {url}
      </a>,
    );
    at = start + url.length;
  }
  if (at < text.length) out.push(text.slice(at));
  return out;
}

/** A picture, video or sticker at egui's size: 300 wide at most. */
function Picture(p: { media: MediaView; sticker: boolean }) {
  const width = () => (p.sticker ? 160 : 300);
  const ratio = () => (p.media.width && p.media.height ? p.media.height / p.media.width : 0.75);
  const height = () => Math.round(Math.min(width() * ratio(), p.sticker ? 160 : 360));
  const src = () => p.media.url ?? p.media.thumb;
  return (
    <div
      class="relative overflow-hidden"
      style={{ width: `${width()}px`, height: `${height()}px`, "border-radius": p.sticker ? "0" : "7px", background: p.sticker ? "transparent" : "var(--quote-bg)" }}
    >
      <Show when={src()}>
        <img src={src()!} alt="" class="h-full w-full" style={{ "object-fit": p.sticker ? "contain" : "cover", filter: p.media.url ? undefined : "blur(1.5px)" }} />
      </Show>
      <Show when={p.media.seconds !== null && !p.sticker}>
        <span
          class="absolute"
          style={{ left: "8px", bottom: "6px", "font-size": "var(--fs-msg-meta)", color: "#fff", "text-shadow": "0 1px 2px rgba(0,0,0,.6)" }}
        >
          {p.media.gif ? "GIF" : duration(p.media.seconds!)}
        </span>
      </Show>
    </div>
  );
}

const DownloadIcon = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 4v11" /><path d="m7 10.5 5 5 5-5" /><path d="M5 19.5h14" />
  </svg>
);

/** A document: its icon, name, pages and size, and the download button. */
function Document(p: { media: MediaView }) {
  return (
    <div class="flex items-center" style={{ width: "300px", gap: "12px", padding: "10px 12px", "border-radius": "7px", background: "var(--quote-bg)" }}>
      <span class="flex shrink-0 items-center justify-center rounded-full" style={{ width: "34px", height: "34px", background: "var(--chip-on-bg)", color: "var(--accent)" }}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" aria-hidden="true">
          <path d="M6.5 3.5h8l4 4v13h-12z" /><path d="M14.5 3.5v4h4" /><path d="M9 12h6M9 15.5h6" />
        </svg>
      </span>
      <span class="flex min-w-0 flex-1 flex-col">
        <span class="truncate" style={{ "font-size": "var(--fs-msg)", "font-weight": 500, color: "var(--text-primary)" }}>{p.media.name}</span>
        <span style={{ "font-size": "var(--fs-hint)", color: "var(--text-secondary)" }}>
          {[p.media.pages ? `${p.media.pages} pages` : null, bytes(p.media.size)].filter(Boolean).join(" · ")}
        </span>
      </span>
      <Show when={!p.media.url}>
        <span style={{ color: "var(--icon)" }}><DownloadIcon /></span>
      </Show>
    </div>
  );
}

/** A voice note or audio: the button, the waveform, the length. */
function Voice(p: { media: MediaView; meta: JSX.Element }) {
  const bars = () => (p.media.waveform.length ? p.media.waveform : Array.from({ length: 64 }, (_, i) => 30 + ((i * 37) % 60)));
  return (
    <div class="flex items-center" style={{ width: "300px", gap: "10px" }}>
      <span class="flex shrink-0 items-center justify-center rounded-full" style={{ width: "34px", height: "34px", background: "var(--chip-on-bg)", color: "var(--accent)" }}>
        <DownloadIcon />
      </span>
      <span class="flex min-w-0 flex-1 flex-col">
        <span class="flex items-center" style={{ height: "24px", gap: "1.5px" }}>
          <For each={bars()}>
            {(bar) => <span style={{ flex: "1", "min-width": "1px", height: `${Math.max(3, (bar / 100) * 22)}px`, "border-radius": "1px", background: "var(--text-meta)" }} />}
          </For>
        </span>
        <span class="flex items-center justify-between" style={{ "margin-top": "4px" }}>
          <span style={{ "font-size": "var(--fs-msg-meta)", color: "var(--text-secondary)" }}>{duration(p.media.seconds ?? 0)}</span>
          {p.meta}
        </span>
      </span>
    </div>
  );
}

/** A link preview: the picture, the title, the description, the site. */
function Preview(p: { preview: PreviewView }) {
  const site = () => {
    try {
      return new URL(p.preview.url).hostname.replace(/^www\./, "");
    } catch {
      return p.preview.url;
    }
  };
  return (
    <a
      href={p.preview.url}
      target="_blank"
      rel="noreferrer noopener"
      onClick={(e) => e.stopPropagation()}
      class="flex overflow-hidden"
      style={{ gap: "8px", margin: "2px 0 6px", padding: "6px", "border-radius": "7px", background: "var(--quote-bg)", color: "inherit", "text-decoration": "none" }}
    >
      <Show when={p.preview.thumb}>
        <img src={p.preview.thumb!} alt="" class="shrink-0" style={{ width: "60px", height: "60px", "border-radius": "5px", "object-fit": "cover" }} />
      </Show>
      <span class="flex min-w-0 flex-col" style={{ "line-height": "16px" }}>
        <Show when={p.preview.title}>
          <span class="truncate" style={{ "font-size": "var(--fs-row-text)", "font-weight": 600, color: "var(--text-primary)" }}>{p.preview.title}</span>
        </Show>
        <Show when={p.preview.description}>
          <span class="line-clamp-2" style={{ "font-size": "var(--fs-hint)", "margin-top": "2px", color: "var(--text-secondary)" }}>{p.preview.description}</span>
        </Show>
        <span class="truncate" style={{ "font-size": "var(--fs-hint)", "margin-top": "2px", color: "var(--text-meta)" }}>{site()}</span>
      </span>
    </a>
  );
}

export default function Message(p: { msg: Msg; first: boolean; last: boolean; group: boolean }) {
  const out = () => p.msg.fromMe;
  const reactions = () => p.msg.reactions.reduce((n, [, c]) => n + c, 0);
  const media = () => p.msg.media;
  const sticker = () => p.msg.kind === "sticker";
  const picture = () => !!media() && ["image", "video", "sticker", "gif"].includes(p.msg.kind);
  const document = () => p.msg.kind === "document" && !!media();
  const voice = () => !!media() && (p.msg.kind === "audio" || p.msg.kind === "voice" || media()!.voice);
  // Media sits on its own and its time goes on a line under it; a text
  // bubble keeps the time beside its last line.
  const block = () => !!media() || !!p.msg.preview;
  const meta = () => (
    <span
      class="flex items-center gap-[3px] whitespace-nowrap"
      style={{ "font-size": "var(--fs-msg-meta)", "line-height": "13px", color: "var(--text-meta)" }}
    >
      <Show when={p.msg.edited}>edited </Show>
      <Show when={p.msg.starred}>
        <svg width="11" height="11" viewBox="0 0 24 24" aria-label="Starred" style={{ color: "var(--star)" }}>
          <path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" fill="currentColor" />
        </svg>
      </Show>
      {p.msg.clock}
      <Show when={out()}>
        <span style={{ color: "var(--text-meta)" }}><Ticks status={p.msg.status} size={15} /></span>
      </Show>
    </span>
  );
  // Room in the last line for the time (and ticks, and "edited").
  const room = () => (p.msg.edited ? 38 : 0) + (out() ? 76 : 58);
  const plain = () => p.msg.kind !== "text" && !media();
  return (
    <div
      class="flex flex-col"
      style={{
        "align-items": out() ? "flex-end" : "flex-start",
        "margin-top": p.first ? "var(--bubble-gap-run)" : "var(--bubble-gap)",
        "padding-left": "var(--bubble-gutter)",
        "padding-right": "var(--bubble-gutter)",
      }}
    >
      <div
        class="relative"
        style={{
          "max-width": "var(--bubble-max)",
          background: sticker() ? "transparent" : out() ? "var(--bubble-out)" : "var(--bubble-in)",
          "border-radius": "var(--bubble-radius)",
          "border-top-left-radius": p.first && !out() ? "0" : undefined,
          "border-top-right-radius": p.first && out() ? "0" : undefined,
          "box-shadow": sticker() ? "none" : "var(--shadow-bubble)",
          padding: block() ? "4px 4px 6px" : "var(--bubble-pad-y) var(--bubble-pad-x)",
          "font-size": "var(--fs-msg)",
          "line-height": "var(--lh-msg)",
        }}
      >
        <Show when={p.first && !sticker()}><Tail out={out()} /></Show>
        <div style={{ padding: block() ? "0 0 0" : undefined }}>
          <Show when={p.group && p.first && p.msg.sender && !out()}>
            <div
              class="truncate"
              style={{ "font-size": "var(--fs-sender)", "line-height": "15px", "font-weight": 600, padding: block() ? "2px 6px 4px" : undefined, color: senderColor(p.msg.senderHue) }}
            >
              {p.msg.sender}
            </div>
          </Show>
          <Show when={p.msg.forwarded}>
            <div style={{ "font-size": "var(--fs-sender)", "font-style": "italic", padding: block() ? "0 6px" : undefined, color: "var(--text-meta)" }}>Forwarded</div>
          </Show>
          <Show when={p.msg.quoted}>
            {(q) => {
              // src/ui/conversation.rs:quote_block: a 4-point bar and the
              // name in the sender's colour (the accent for our own), the
              // summary in secondary, on the window colour at 35%.
              const tint = () => (q().mine ? "var(--accent)" : senderColor(q().hue));
              return (
                <div
                  class="relative overflow-hidden"
                  style={{ margin: block() ? "4px 4px 4px" : "2px 0 4px", "border-radius": "6px", background: "var(--quote-bg)", padding: "5px 10px 6px 11px" }}
                >
                  <span class="absolute inset-y-0 left-0" style={{ width: "4px", background: tint() }} />
                  <div style={{ "font-size": "12.5px", "line-height": "16px", "font-weight": 600, color: tint() }}>{q().author}</div>
                  <div class="line-clamp-3" style={{ "font-size": "12.5px", "line-height": "16px", "margin-top": "1px", color: "var(--text-secondary)" }}>{q().text}</div>
                </div>
              );
            }}
          </Show>
          <Show when={picture()}><Picture media={media()!} sticker={sticker()} /></Show>
          <Show when={document()}><Document media={media()!} /></Show>
          <Show when={voice()}><div style={{ padding: "6px 6px 0" }}><Voice media={media()!} meta={meta()} /></div></Show>
          <Show when={p.msg.preview}><Preview preview={p.msg.preview!} /></Show>
          <Show when={p.msg.text}>
            <span
              class="selectable block whitespace-pre-wrap break-words"
              style={{
                "font-size": "var(--fs-msg)",
                "line-height": "var(--lh-msg)",
                padding: block() ? "6px 6px 0" : undefined,
                color: plain() ? "var(--text-meta)" : "var(--text-primary)",
                "font-style": p.msg.kind === "revoked" ? "italic" : undefined,
              }}
            >
              {plain() ? p.msg.text : formatted(p.msg.text)}
              <Show when={!block()}><span class="inline-block" style={{ width: `${room()}px` }} /></Show>
            </span>
          </Show>
        </div>
        <Show
          when={block() && !voice()}
          fallback={<Show when={!voice()}><span class="absolute" style={{ right: "var(--bubble-pad-x)", bottom: "5px" }}>{meta()}</span></Show>}
        >
          <div class="flex justify-end" style={{ padding: "4px 6px 0" }}>{meta()}</div>
        </Show>
      </div>
      <Show when={reactions() > 0}>
        {/* src/ui/conversation.rs:reactions: up to three emoji and the
            total, regular 13 on the overlay colour, outlined in the accent
            when we reacted, else in the chat colour. */}
        <div
          class="relative flex items-center rounded-full"
          style={{
            margin: "-4px 10px 0",
            padding: "3px 6px",
            background: "var(--reaction-bg)",
            border: `1px solid ${p.msg.reacted ? "var(--accent)" : "var(--conversation-bg)"}`,
            "box-shadow": "0 1px 1px rgba(11, 20, 26, 0.08)",
            "font-size": "13px",
            "line-height": "16px",
            color: "var(--text-primary)",
          }}
        >
          {p.msg.reactions.slice(0, 3).map(([emoji]) => emoji).join(" ")} {reactions()}
        </div>
      </Show>
    </div>
  );
}
