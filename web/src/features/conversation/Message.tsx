// One message bubble, as the egui app draws it in WhatsApp's style
// (src/ui/widgets.rs:bubble_shape, src/ui/conversation.rs): radius 10, the
// first bubble of a run with a tail at its top corner, pictures, documents,
// voice notes and link previews drawn in place, WhatsApp's *bold* _italic_
// ~strike~ formatting, and the reactions in a pill under the bubble. Text is
// always text, never markup (REQ-WEBUI-003.A4).
import { createEffect, createSignal, For, on, onCleanup, Show, type JSX } from "solid-js";
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
  // conversation.rs:fit_picture within PICTURE_WIDTH × PICTURE_HEIGHT
  // (320 × 440): no upscaling, at least 120 wide and 90 tall.
  const fit = () => {
    if (p.sticker) return [160, 160];
    const w = p.media.width || 4, h = p.media.height || 3;
    let scale = Math.min(Math.min(320 / w, 440 / h), 1);
    if (w * scale < 120) scale = Math.min(120 / w, 320 / w);
    return [Math.round(w * scale), Math.round(Math.max(h * scale, 90))];
  };
  const width = () => fit()[0];
  const height = () => fit()[1];
  const src = () => p.media.url ?? p.media.thumb;
  return (
    <div
      class="relative overflow-hidden"
      style={{ width: `${width()}px`, height: `${height()}px`, "border-radius": p.sticker ? "0" : "6px", background: p.sticker ? "transparent" : "var(--quote-bg)" }}
    >
      <Show when={src()}>
        <img src={src()!} alt="" class="h-full w-full" style={{ "object-fit": p.sticker ? "contain" : "fill" }} />
      </Show>
      {/* conversation.rs:picture, not downloaded: the preview stretched to
          the frame under a black wash (60), a 44-point disc (black 120) with
          the download icon, and the size 34 under the middle. */}
      <Show when={!p.media.url && !p.sticker}>
        <span class="absolute inset-0" style={{ background: "rgba(0,0,0,0.235)" }} />
        <span
          class="absolute flex items-center justify-center rounded-full"
          style={{ width: "44px", height: "44px", left: "calc(50% - 22px)", top: "calc(50% - 22px)", background: "rgba(0,0,0,0.47)", color: "#fff" }}
        >
          <DownloadIcon />
        </span>
        <span
          class="absolute w-full text-center"
          style={{ top: "calc(50% + 34px - 7px)", "font-size": "11.5px", "line-height": "14px", color: "#fff" }}
        >
          {bytes(p.media.size)}
        </span>
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
    // conversation.rs:attachment: margins 10 × 8 around a 52-point row, the
    // 36-point disc, then the name medium 14 over the detail regular 12.
    <div class="flex items-center" style={{ width: "320px", height: "68px", gap: "8px", padding: "8px 10px", "border-radius": "8px", background: "var(--quote-bg)" }}>
      <span class="flex shrink-0 items-center justify-center rounded-full" style={{ width: "36px", height: "36px", background: "color-mix(in srgb, var(--accent) 25%, transparent)", color: "var(--accent)" }}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" aria-hidden="true">
          <path d="M6.5 3.5h8l4 4v13h-12z" /><path d="M14.5 3.5v4h4" /><path d="M9 12h6M9 15.5h6" />
        </svg>
      </span>
      <span class="flex min-w-0 flex-1 flex-col self-start">
        <span class="truncate" style={{ "font-size": "14px", "line-height": "16.5px", "font-weight": 500, color: "var(--text-primary)" }}>{p.media.name}</span>
        <span style={{ "font-size": "12px", "line-height": "14px", "margin-top": "1px", color: "var(--text-secondary)" }}>
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
  const samples = () => (p.media.waveform.length ? p.media.waveform : Array.from({ length: 64 }, (_, i) => 30 + ((i * 37) % 60)));
  // egui resamples the waveform to one 2-point bar every 3 points across
  // the 274 points beside the button.
  const bars = () => {
    const s = samples();
    const count = Math.floor(274 / 3);
    return Array.from({ length: count }, (_, i) => s[Math.floor((i * s.length) / count)]);
  };
  return (
    // conversation.rs:voice_player: a 36-point row, the button and the
    // 30-point waveform centred on it; the length sits right under the
    // waveform, overlapping the row's foot.
    <div class="flex items-start" style={{ width: "320px", gap: "10px" }}>
      <span class="flex shrink-0 items-center justify-center rounded-full" style={{ width: "36px", height: "36px", background: "var(--chip-on-bg)", color: "var(--accent)" }}>
        <DownloadIcon />
      </span>
      <span class="flex min-w-0 flex-1 flex-col" style={{ "padding-top": "3px" }}>
        <span class="flex items-center" style={{ height: "30px", gap: "1px" }}>
          <For each={bars()}>
            {(bar) => <span class="shrink-0" style={{ width: "2px", height: `${2 + (bar / 100) * 26}px`, "border-radius": "1px", background: "color-mix(in srgb, var(--text-secondary) 70%, transparent)" }} />}
          </For>
        </span>
        <span class="flex items-center justify-between" style={{ height: "13px" }}>
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
      // conversation.rs:preview_card / preview_layout: margins 8, radius 6,
      // a 64-point thumbnail, then the title semibold 13.5, up to two lines
      // of description regular 12.5 and the domain regular 12, 2 apart.
      class="flex items-start overflow-hidden"
      // settled_width: the bubble's width comes from its text (at least the
      // card floor); the card fills it and never widens it.
      style={{ contain: "inline-size", gap: "8px", margin: "2px 6px 0", padding: "8px", "border-radius": "6px", background: "var(--quote-bg)", color: "inherit", "text-decoration": "none" }}
    >
      <Show when={p.preview.thumb}>
        <img src={p.preview.thumb!} alt="" class="shrink-0" style={{ width: "64px", height: "64px", "border-radius": "4px", "object-fit": "cover" }} />
      </Show>
      <span class="flex min-w-0 flex-1 flex-col">
        <Show when={p.preview.title}>
          <span class="truncate" style={{ "font-size": "13.5px", "line-height": "16px", "font-weight": 600, color: "var(--text-primary)" }}>{p.preview.title}</span>
        </Show>
        <Show when={p.preview.description}>
          <span class="line-clamp-2" style={{ "font-size": "12.5px", "line-height": "14.5px", "margin-top": "2px", color: "var(--text-secondary)" }}>{p.preview.description}</span>
        </Show>
        <span class="truncate" style={{ "font-size": "12px", "line-height": "14px", "margin-top": "2px", color: "var(--text-meta)" }}>{site()}</span>
      </span>
    </a>
  );
}

// One window-size signal shared by every bubble, for re-measuring.
const [viewport, setViewport] = createSignal(0);
if (typeof window !== "undefined") window.addEventListener("resize", () => setViewport((n) => n + 1));

export default function Message(p: { msg: Msg; first: boolean; last: boolean; group: boolean }) {
  const out = () => p.msg.fromMe;
  const reactions = () => p.msg.reactions.reduce((n, [, c]) => n + c, 0);
  const media = () => p.msg.media;
  const sticker = () => p.msg.kind === "sticker";
  const picture = () => !!media() && ["image", "video", "sticker", "gif"].includes(p.msg.kind);
  const document = () => p.msg.kind === "document" && !!media();
  const voice = () => !!media() && (p.msg.kind === "audio" || p.msg.kind === "voice" || media()!.voice);
  // Media and link previews sit inset in the bubble. Media puts its time
  // on a line under it; text, also under a link preview, keeps the time
  // beside its last line, as egui does.
  const inset = () => !!media() || !!p.msg.preview;
  const block = () => !!media() || (!!p.msg.preview && !p.msg.text);
  const meta = () => (
    <span
      class="flex items-center gap-[3px] whitespace-nowrap"
      style={{ "font-size": "var(--fs-msg-meta)", "line-height": "13px", color: "var(--text-meta)" }}
    >
      <Show when={p.msg.edited}><span data-edited>edited</span></Show>
      <Show when={p.msg.starred}>
        <svg width="11" height="11" viewBox="0 0 24 24" aria-label="Starred" style={{ color: "var(--star)" }}>
          <path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" fill="currentColor" />
        </svg>
      </Show>
      <span data-clock>{p.msg.clock}</span>
      <Show when={out()}>
        <span style={{ color: "var(--text-meta)" }}><Ticks status={p.msg.status} size={15} /></span>
      </Show>
    </span>
  );
  // Room in the last line for the time (and ticks, and "edited").
  // Room in the last line for the time: egui's footer_width plus the 8
  // points before it, from the time as drawn once it is measured.
  const [measuredRoom, setMeasuredRoom] = createSignal<number | null>(null);
  const room = () => measuredRoom() ?? (p.msg.edited ? 38 : 0) + (out() ? 76 : 58);
  // conversation.rs:natural_text_width: a bubble whose text runs to more
  // than one line is as wide as its widest line (the time's line counts
  // when the time fits after the text), not as wide as it may grow.
  const [fit, setFit] = createSignal<number | null>(null);
  // egui gives a time that cannot follow the text a row of its own, 3
  // taller than a line of text with the row spacing.
  const [ownLine, setOwnLine] = createSignal(false);
  let bubbleEl: HTMLDivElement | undefined;
  let textEl: HTMLSpanElement | undefined;
  let metaEl: HTMLSpanElement | undefined;
  let frame = 0;
  const measureLines = () => {
    if (!bubbleEl || !textEl) return;
    const range = window.document.createRange();
    range.selectNodeContents(textEl);
    const lines = new Map<number, [number, number]>();
    for (const r of range.getClientRects()) {
      if (r.width === 0) continue;
      const key = Math.round(r.top + r.height / 2);
      const found = [...lines.keys()].find((k) => Math.abs(k - key) < 6);
      const at = found ?? key;
      const [l, rr] = lines.get(at) ?? [Infinity, -Infinity];
      lines.set(at, [Math.min(l, r.left), Math.max(rr, r.right)]);
    }
    const spacer = textEl.querySelector<HTMLElement>("[data-room]");
    if (spacer) {
      const room = spacer.getBoundingClientRect();
      const before = window.document.createRange();
      before.setStart(textEl, 0);
      before.setEndBefore(spacer);
      const text = [...before.getClientRects()].filter((r) => r.width > 0);
      setOwnLine(text.length > 0 && room.top >= Math.max(...text.map((r) => r.bottom)) - 2);
    }
    if (lines.size < 2) return;
    const widest = Math.max(...[...lines.values()].map(([l, r]) => r - l));
    // Everything around the text's lines: the bubble's margins and the text
    // element's own side padding.
    const pad = getComputedStyle(textEl);
    const inner = textEl.getBoundingClientRect().width - parseFloat(pad.paddingLeft) - parseFloat(pad.paddingRight);
    const chrome = bubbleEl.getBoundingClientRect().width - inner;
    setFit(Math.ceil(widest + chrome) + 1);
  };
  const measure = () => {
    cancelAnimationFrame(frame);
    setFit(null);
    setOwnLine(false);
    frame = requestAnimationFrame(() => {
      // conversation.rs:footer_width: the clock, "edited" and 4, 19 for the
      // ticks, a 21-point slot per mark; then the 8 before it. egui's
      // "edited" measures about 2 wider than WebKit's, so 2 more with it.
      const width = (sel: string) => metaEl?.querySelector(sel)?.getBoundingClientRect().width ?? 0;
      if (metaEl && !block())
        setMeasuredRoom(
          Math.ceil(width("[data-clock]") + (p.msg.edited ? width("[data-edited]") + 4 : 0) + (out() ? 19 : 0) + (p.msg.starred ? 21 : 0)) + 8 + (p.msg.edited ? 2 : 0),
        );
      frame = requestAnimationFrame(measureLines);
    });
  };
  createEffect(on(() => [p.msg.text, p.msg.clock, p.msg.edited, p.msg.starred, viewport()], measure));
  // The room a bubble may take changes with its row (the info panel
  // opening, the list hidden) and with the fonts once they load.
  let rowEl: HTMLDivElement | undefined;
  let rowWidth = 0;
  const rows = new ResizeObserver(([entry]) => {
    const width = Math.round(entry.contentRect.width);
    if (width !== rowWidth) {
      rowWidth = width;
      measure();
    }
  });
  queueMicrotask(() => rowEl && rows.observe(rowEl));
  void window.document.fonts?.ready.then(() => measure());
  onCleanup(() => rows.disconnect());
  onCleanup(() => cancelAnimationFrame(frame));
  const card = () => !!p.msg.quoted || !!p.msg.preview || document() || voice();
  const plain = () => p.msg.kind !== "text" && !media();
  return (
    <div
      ref={rowEl}
      class="flex flex-col"
      style={{
        "align-items": out() ? "flex-end" : "flex-start",
        "margin-top": p.first ? "var(--bubble-gap-run)" : "var(--bubble-gap)",
        "padding-left": "var(--bubble-gutter)",
        "padding-right": "var(--bubble-gutter)",
      }}
    >
      <div
        ref={bubbleEl}
        class="relative"
        style={{
          width: fit() ? `${fit()}px` : undefined,
          // conversation.rs: (width × WHATSAPP_WIDTH_SHARE).min(560).
          "max-width": "min(var(--bubble-max), 560px)",
          // conversation.rs:settled_width: quotes, previews, documents and
          // audio draw as cards at least CARD_WIDTH (320) plus the margins.
          "min-width": card() ? "min(340px, var(--bubble-max))" : undefined,
          background: sticker() ? "transparent" : out() ? "var(--bubble-out)" : "var(--bubble-in)",
          "border-radius": "var(--bubble-radius)",
          "border-top-left-radius": p.first && !out() ? "0" : undefined,
          "border-top-right-radius": p.first && out() ? "0" : undefined,
          "box-shadow": sticker() ? "none" : "var(--shadow-bubble)",
          padding: inset() ? "4px 4px 6px" : "var(--bubble-pad-y) var(--bubble-pad-x) var(--bubble-pad-bottom)",
          "font-size": "var(--fs-msg)",
          "line-height": "var(--lh-msg)",
        }}
      >
        <Show when={p.first && !sticker()}><Tail out={out()} /></Show>
        <div style={{ padding: block() ? "0 0 0" : undefined }}>
          <Show when={p.group && p.first && p.msg.sender && !out()}>
            <div
              class="truncate"
              style={{ "font-size": "var(--fs-sender)", "line-height": "15px", "font-weight": 600, padding: inset() ? "2px 6px 4px" : undefined, color: senderColor(p.msg.senderHue) }}
            >
              {p.msg.sender}
            </div>
          </Show>
          <Show when={p.msg.forwarded}>
            <div style={{ "font-size": "var(--fs-sender)", "font-style": "italic", padding: inset() ? "0 6px" : undefined, color: "var(--text-meta)" }}>Forwarded</div>
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
                  style={{ contain: "inline-size", margin: inset() ? "4px 4px 4px" : "0 0 4px", "border-radius": "6px", background: "var(--quote-bg)", padding: "5px 10px 6px 11px" }}
                >
                  <span class="absolute inset-y-0 left-0" style={{ width: "4px", background: tint() }} />
                  <div style={{ "font-size": "12.5px", "line-height": "14.5px", "font-weight": 600, color: tint() }}>{q().author}</div>
                  <div class="line-clamp-3" style={{ "font-size": "12.5px", "line-height": "14.5px", "margin-top": "1px", color: "var(--text-secondary)" }}>{q().text}</div>
                </div>
              );
            }}
          </Show>
          {/* bubble_margin: a picture keeps the bubble's 10 × 6 margins. */}
          <Show when={picture()}><div style={{ padding: sticker() ? undefined : "2px 6px 0" }}><Picture media={media()!} sticker={sticker()} /></div></Show>
          <Show when={document()}><div style={{ padding: "2px 6px 0" }}><Document media={media()!} /></div></Show>
          <Show when={voice()}><div style={{ padding: "2px 6px 0" }}><Voice media={media()!} meta={meta()} /></div></Show>
          <Show when={p.msg.preview}><Preview preview={p.msg.preview!} /></Show>
          <Show when={p.msg.text}>
            <span
              ref={textEl}
              class="selectable block whitespace-pre-wrap break-words"
              style={{
                "font-size": "var(--fs-msg)",
                "line-height": "var(--lh-msg)",
                // egui's row spacing: text sits 3 under its picture or card.
                padding: inset() ? `3px 6px ${ownLine() ? 3 : 0}px` : ownLine() ? "0 0 3px" : undefined,
                color: plain() ? "var(--text-meta)" : "var(--text-primary)",
                "font-style": p.msg.kind === "revoked" ? "italic" : undefined,
              }}
            >
              {plain() ? p.msg.text : formatted(p.msg.text)}
              <Show when={!block()}><span data-room class="inline-block" style={{ width: `${room()}px` }} /></Show>
            </span>
          </Show>
        </div>
        <Show
          when={block() && !voice()}
          fallback={
            <Show when={!voice()}>
              {/* Level with the last line: the 13-point meta centred on the
                  17-point row, 2 above the bottom margin. */}
              <span ref={metaEl} class="absolute" style={{ right: inset() ? "10px" : "var(--bubble-pad-x)", bottom: inset() ? "7px" : "calc(var(--bubble-pad-bottom) + 2px)" }}>{meta()}</span>
            </Show>
          }
        >
          <div class="flex justify-end" style={{ padding: "6px 6px 0" }}>{meta()}</div>
        </Show>
      </div>
      <Show when={reactions() > 0}>
        {/* src/ui/conversation.rs:reactions: up to three emoji and the
            total, regular 13 on the overlay colour, outlined in the accent
            when we reacted, else in the chat colour. */}
        <div
          class="relative flex items-center rounded-full"
          style={{
            // -7 under the bubble, 2 and the row spacing below (egui's row).
            margin: "1px 0 9px",
            padding: "2px 5px",
            background: "var(--reaction-bg)",
            border: `1px solid ${p.msg.reacted ? "var(--accent)" : "var(--conversation-bg)"}`,
            "box-shadow": "0 1px 1px rgba(11, 20, 26, 0.08)",
            "font-size": "13px",
            "line-height": "13px",
            color: "var(--text-primary)",
          }}
        >
          {/* egui draws the emoji at the text's 13 points, smaller than a
              colour emoji font sets them at that size. */}
          <span style={{ "font-size": "11px" }}>{p.msg.reactions.slice(0, 3).map(([emoji]) => emoji).join(" ")}</span>&nbsp;{reactions()}
        </div>
      </Show>
    </div>
  );
}
