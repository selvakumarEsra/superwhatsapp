// The call history (REQ-WEBUI-010), as the egui app draws it
// (src/ui/calls.rs:history_row): 68-point rows with the 48-point picture,
// the name (red when missed) and the time, then the way the call went and
// egui's own summary ("Missed voice call", "Video call · 12 min 34 s"),
// and what it carried on the right. Alike calls fold into one row with a
// count. Clicking a call opens its chat.
import { For, Show } from "solid-js";
import { commands, state, type CallRow } from "../../lib/bridge";
import Avatar from "../../components/Avatar";
import { CallsIcon, PhoneIncomingIcon, PhoneMissedIcon, PhoneOutgoingIcon, VideoIcon } from "../../components/Icons";
import { t } from "../../lib/i18n";

function Row(p: { call: CallRow }) {
  const Glyph = () => (p.call.missed ? <PhoneMissedIcon size={15} /> : p.call.outgoing ? <PhoneOutgoingIcon size={15} /> : <PhoneIncomingIcon size={15} />);
  return (
    <div
      role="option"
      tabIndex={0}
      onClick={() => commands.openChat(p.call.chat)}
      onKeyDown={(e) => e.key === "Enter" && commands.openChat(p.call.chat)}
      class="chat-row relative flex cursor-default items-center"
      style={{ height: "var(--row-h)" }}
      aria-label={`${p.call.title}, ${p.call.summary}, ${p.call.stamp}`}
    >
      <div class="row-card absolute" style={{ inset: "2px var(--row-inset)", "border-radius": "var(--row-radius)" }} />
      <div class="relative flex shrink-0 items-center justify-center" style={{ width: "76px" }}>
        <Avatar src={p.call.avatar} size={48} group={p.call.group} seed={p.call.chat} />
      </div>
      <div class="relative flex h-full min-w-0 flex-1 flex-col" style={{ "padding-top": "13px", "padding-right": "6px" }}>
        <div class="flex items-center justify-between" style={{ gap: "8px", height: "20px" }}>
          <span class="truncate" style={{ "font-size": "14.5px", "font-weight": 500, color: p.call.missed ? "var(--danger)" : "var(--text-primary)" }}>
            {p.call.title}
          </span>
          <span class="shrink-0" style={{ "font-size": "var(--fs-meta)", color: "var(--text-meta)" }}>{p.call.stamp}</span>
        </div>
        <div class="flex items-center" style={{ gap: "6px", height: "18px", "margin-top": "4px", "font-size": "13px", color: "var(--text-secondary)" }}>
          <span class="flex shrink-0" style={{ color: p.call.bad ? "var(--danger)" : "var(--accent)" }}><Glyph /></span>
          <span class="truncate">{p.call.summary}</span>
        </div>
      </div>
      {/* Call back, as the egui row's button: the way the call went. */}
      <button
        type="button"
        title={t(p.call.video ? "Video call" : "Voice call")}
        aria-label={t(p.call.video ? "Video call" : "Voice call")}
        onClick={(e) => { e.stopPropagation(); void commands.startCall(p.call.chat, p.call.video); }}
        class="icon-button relative flex shrink-0 items-center justify-center rounded-full"
        style={{ width: "36px", height: "36px", "margin-right": "12px", color: "var(--text-secondary)" }}
      >
        <Show when={p.call.video} fallback={<CallsIcon size={18} />}>
          <VideoIcon size={18} />
        </Show>
      </button>
    </div>
  );
}

export default function CallsList() {
  return (
    <Show
      when={state.calls.length > 0}
      fallback={
        <div class="flex flex-col items-center px-[24px] pt-[48px] text-center" style={{ color: "var(--text-secondary)" }}>
          <Show when={state.callFilter === "missed"} fallback={<CallsIcon size={40} />}>
            <PhoneMissedIcon size={40} />
          </Show>
          <p style={{ "font-size": "15px", "font-weight": 600, margin: "14px 0 0", color: "var(--text-primary)" }}>
            {t(state.callFilter === "missed" ? "No missed calls" : state.callsSearch.trim() ? "No matching calls" : "No calls yet")}
          </p>
          <Show when={!state.callsSearch.trim()}>
            <p style={{ "font-size": "13px", "line-height": "17px", margin: "6px 0 0" }}>
              {t(state.callFilter === "missed" ? "Choose All to see every call." : "Calls from your phone's call log and the ones you make here appear here.")}
            </p>
          </Show>
        </div>
      }
    >
      <div role="listbox" aria-label="Calls">
        <For each={state.calls}>{(call) => <Row call={call} />}</For>
      </div>
    </Show>
  );
}
