// One message bubble, as WhatsApp Desktop for macOS draws it: rounded,
// with the tail at the bottom of the last bubble in a run. Text is always
// text, never markup (REQ-WEBUI-003.A4).
import { For, Show } from "solid-js";
import type { Msg } from "../../lib/bridge";
import { clock } from "../../lib/time";
import { Ticks } from "../../components/Icons";

/** The tail at the bottom corner of the last bubble in a run. */
function Tail(p: { out: boolean }) {
  return (
    <svg
      width="7"
      height="13"
      viewBox="0 0 7 13"
      class="absolute bottom-0"
      style={{ [p.out ? "right" : "left"]: "-6px", color: p.out ? "var(--bubble-out)" : "var(--bubble-in)" }}
      aria-hidden="true"
    >
      {p.out ? <path d="M0 0C0 6 1.5 10.5 7 13H0z" fill="currentColor" /> : <path d="M7 0C7 6 5.5 10.5 0 13h7z" fill="currentColor" />}
    </svg>
  );
}

const senderColor = (id: string) => {
  const palette = ["#06cf9c", "#e542a3", "#a5b337", "#ff6b4a", "#53bdeb", "#7f66ff", "#ffbc38", "#02a698"];
  let h = 0;
  for (const c of id) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return palette[h % palette.length];
};

export default function Message(p: { msg: Msg; first: boolean; last: boolean; group: boolean }) {
  const out = () => p.msg.fromMe;
  const reactions = () => p.msg.reactions.reduce((n, [, c]) => n + c, 0);
  // Room in the last line for the time (and ticks, and "Edited").
  const room = () => (p.msg.edited ? 30 : 0) + (out() ? 58 : 42);
  return (
    <div
      class="flex"
      style={{
        "justify-content": out() ? "flex-end" : "flex-start",
        "margin-top": p.first ? "var(--bubble-gap-run)" : "var(--bubble-gap)",
        "margin-bottom": reactions() > 0 ? "15px" : undefined,
        "padding-left": "var(--bubble-gutter)",
        "padding-right": "var(--bubble-gutter)",
      }}
    >
      <div
        class="relative"
        style={{
          "max-width": "var(--bubble-max)",
          background: out() ? "var(--bubble-out)" : "var(--bubble-in)",
          "border-radius": "var(--bubble-radius)",
          "border-bottom-left-radius": p.last && !out() ? "0" : undefined,
          "border-bottom-right-radius": p.last && out() ? "0" : undefined,
          "box-shadow": "var(--shadow-bubble)",
          padding: "var(--bubble-pad-y) var(--bubble-pad-x)",
          "font-size": "var(--fs-msg)",
          "line-height": "var(--lh-msg)",
        }}
      >
        <Show when={p.last}><Tail out={out()} /></Show>
        <Show when={p.group && p.first && p.msg.sender && !out()}>
          <div
            class="truncate"
            style={{ "font-size": "var(--fs-sender)", "line-height": "15px", "font-weight": 600, color: senderColor(p.msg.senderId) }}
          >
            {p.msg.sender}
          </div>
        </Show>
        <Show when={p.msg.forwarded}>
          <div style={{ "font-size": "var(--fs-sender)", "font-style": "italic", color: "var(--text-meta)" }}>Forwarded</div>
        </Show>
        <Show when={p.msg.quoted}>
          <div
            class="overflow-hidden"
            style={{
              margin: "2px 0 4px",
              "border-radius": "7px",
              background: "var(--quote-bg)",
              "border-left": "3px solid var(--quote-bar)",
              padding: "4px 8px 5px",
            }}
          >
            <div style={{ "font-size": "var(--fs-sender)", "font-weight": 600, color: "var(--quote-bar)" }}>{p.msg.quoted!.author}</div>
            <div class="line-clamp-3" style={{ "font-size": "var(--fs-msg)", color: "var(--text-meta)" }}>{p.msg.quoted!.text}</div>
          </div>
        </Show>
        <span
          class="selectable whitespace-pre-wrap break-words"
          style={{ "font-size": "var(--fs-msg)", "line-height": "var(--lh-msg)", color: "var(--text-primary)" }}
        >
          <Show when={p.msg.kind !== "text"} fallback={p.msg.text}>
            <span style={{ color: "var(--text-meta)" }}>{p.msg.text}</span>
          </Show>
          <span class="inline-block" style={{ width: `${room()}px` }} />
        </span>
        <span
          class="absolute flex items-center gap-[3px] whitespace-nowrap"
          style={{ right: "var(--bubble-pad-x)", bottom: "3.5px", "font-size": "var(--fs-msg-meta)", "line-height": "13px", color: "var(--text-meta)" }}
        >
          <Show when={p.msg.edited}>Edited </Show>
          {clock(p.msg.time)}
          <Show when={out()}>
            <span style={{ color: "var(--text-meta)" }}><Ticks status={p.msg.status} size={15} /></span>
          </Show>
        </span>
        <Show when={reactions() > 0}>
          <div
            class="absolute flex items-center gap-[2px] rounded-full"
            style={{
              bottom: "-17px",
              [out() ? "right" : "left"]: "6px",
              height: "22px",
              padding: "0 6px",
              background: "var(--bubble-in)",
              border: "1.5px solid var(--reaction-border)",
              "font-size": "13px",
            }}
          >
            <For each={p.msg.reactions}>{([emoji]) => <span>{emoji}</span>}</For>
            <Show when={reactions() > 1}>
              <span style={{ "font-size": "var(--fs-msg-meta)", color: "var(--text-meta)" }}>{reactions()}</span>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
}
