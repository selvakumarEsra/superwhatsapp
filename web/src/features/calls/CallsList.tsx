// The call log, newest first (REQ-WEBUI-010): who, which way, missed or
// not, voice or video, and when. Clicking a call opens its chat.
import { For, Show } from "solid-js";
import { commands, state, type CallRow } from "../../lib/bridge";
import { listTime } from "../../lib/time";
import Avatar from "../../components/Avatar";
import { ArrowUpRightIcon, CallsIcon, VideoIcon } from "../../components/Icons";

const length = (seconds: number) => {
  if (!seconds) return "";
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return m >= 60 ? `${Math.floor(m / 60)}:${String(m % 60).padStart(2, "0")}:${String(s).padStart(2, "0")}` : `${m}:${String(s).padStart(2, "0")}`;
};

function Row(p: { call: CallRow }) {
  const kind = () => (p.call.missed ? "Missed" : p.call.outgoing ? "Outgoing" : "Incoming");
  return (
    <div
      role="option"
      tabIndex={0}
      onClick={() => commands.openChat(p.call.chat)}
      onKeyDown={(e) => e.key === "Enter" && commands.openChat(p.call.chat)}
      class="chat-row flex cursor-default items-center"
      style={{ height: "var(--row-h)", margin: "0 var(--row-inset)", "padding-left": "var(--row-pad-l)", "border-radius": "var(--row-radius)" }}
    >
      <Avatar src={p.call.avatar} size={40} group={p.call.group} seed={p.call.chat} />
      <div
        class="flex h-full min-w-0 flex-1 items-center justify-between"
        style={{ "margin-left": "var(--row-gap)", "padding-right": "var(--row-pad-r)", "border-bottom": "var(--hairline) solid var(--divider)" }}
      >
        <div class="min-w-0">
          <div
            class="truncate"
            style={{
              "font-size": "var(--fs-row-title)",
              "font-weight": "var(--fw-row-title)",
              "line-height": "15px",
              color: p.call.missed ? "var(--danger)" : "var(--text-primary)",
            }}
          >
            {p.call.title}
          </div>
          <div class="mt-[2px] flex items-center gap-[4px]" style={{ "font-size": "var(--fs-row-text)", "line-height": "14px", color: "var(--text-secondary)" }}>
            <span
              style={{
                display: "inline-flex",
                transform: p.call.outgoing ? undefined : "rotate(180deg)",
                color: p.call.missed ? "var(--danger)" : "var(--accent-text)",
              }}
            >
              <ArrowUpRightIcon size={12} />
            </span>
            {kind()}
            <Show when={p.call.duration > 0}> · {length(p.call.duration)}</Show>
          </div>
        </div>
        <div class="flex shrink-0 flex-col items-end gap-[3px]" style={{ "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>
          {listTime(p.call.time)}
          <span aria-label={p.call.video ? "Video call" : "Voice call"}>
            <Show when={p.call.video} fallback={<CallsIcon size={13} />}>
              <VideoIcon size={13} />
            </Show>
          </span>
        </div>
      </div>
    </div>
  );
}

export default function CallsList() {
  return (
    <Show
      when={state.calls.length > 0}
      fallback={
        <div class="flex flex-col items-center px-[32px] pt-[48px] text-center" style={{ color: "var(--text-secondary)" }}>
          <CallsIcon size={40} />
          <p style={{ "font-size": "var(--fs-row-text)", "line-height": "16px", margin: "14px 0 0" }}>No calls yet.</p>
        </div>
      }
    >
      <div role="listbox" aria-label="Calls">
        <For each={state.calls}>{(call) => <Row call={call} />}</For>
      </div>
    </Show>
  );
}
