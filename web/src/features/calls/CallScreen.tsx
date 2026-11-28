// The call screen (REQ-WEBUI-010), as egui draws it (src/ui/call.rs): the
// window's colour over everything, the picture, the name in bold 26 and
// the status under it, and the round 56-point controls along the bottom.
// Ringing offers Decline and Accept; a live call offers mute, the camera
// when the call can take video, and hang up. Audio and camera stay in the
// core: the page never asks for the microphone or camera.
import { createEffect, createSignal, For, type JSX, onCleanup, Show } from "solid-js";
import { Caret, combo } from "../settings/parts";
import { commands, state, type CallView } from "../../lib/bridge";
import { t } from "../../lib/i18n";
import Avatar from "../../components/Avatar";
import { BackIcon, CallsIcon, MaximizeIcon, MinimizeIcon, MicIcon, Volume2Icon, MicOffIcon, PhoneOffIcon, VideoIcon, VideoOffIcon } from "../../components/Icons";

function Control(p: { label: string; fill: string; tint: string; enabled?: boolean; onClick: () => void; children: JSX.Element }) {
  const enabled = () => p.enabled !== false;
  return (
    <button
      type="button"
      title={t(p.label)}
      aria-label={t(p.label)}
      disabled={!enabled()}
      onClick={p.onClick}
      class="call-control flex items-center justify-center rounded-full"
      style={{ width: "56px", height: "56px", background: p.fill, color: p.tint, opacity: enabled() ? 1 : 0.6 }}
    >
      {p.children}
    </button>
  );
}

/** egui's elapsed(): m:ss, or h:mm:ss past an hour. */
const elapsed = (s: number) => {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const ss = String(s % 60).padStart(2, "0");
  return h ? `${h}:${String(m).padStart(2, "0")}:${ss}` : `${m}:${ss}`;
};

/** The status line; while active, the page counts the seconds on. */
function Status(p: { call: CallView }) {
  const [tick, setTick] = createSignal(0);
  let base = Date.now();
  createEffect(() => {
    void p.call.seconds;
    base = Date.now();
    setTick(0);
  });
  const timer = setInterval(() => setTick(Math.floor((Date.now() - base) / 1000)), 1000);
  onCleanup(() => clearInterval(timer));
  return <>{p.call.seconds !== null ? elapsed(p.call.seconds + tick()) : p.call.status}</>;
}

/** devices(): Microphone, Speaker and Camera, each with System default. */
function Picker(p: { label: string; kind: "microphone" | "speaker" | "camera"; list: [string, string][]; chosen: string | null }) {
  return (
    <label class="flex flex-col" style={{ gap: "4px", "font-size": "12.5px", "font-weight": 500, color: "var(--text-secondary)" }}>
      {t(p.label)}
      <span class="relative inline-flex items-center" style={{ width: "220px" }}>
        <select
          value={p.chosen ?? ""}
          onChange={(e) => commands.callDevice(p.kind, e.currentTarget.value || null)}
          class="w-full"
          style={{ ...combo, background: "var(--panel-bg)" }}
        >
          <option value="">{t("System default")}</option>
          <For each={p.list}>{([id, name]) => <option value={id}>{name}</option>}</For>
        </select>
        <Caret />
      </span>
    </label>
  );
}

function Screen(p: { call: CallView }) {
  const [devices, setDevices] = createSignal(false);
  const [full, setFull] = createSignal(false);
  const statusColor = () => (p.call.failed ? "var(--danger)" : p.call.phase === "live" ? "var(--accent-text)" : "var(--text-secondary)");
  return (
    <div
      role="dialog"
      aria-label={t(p.call.video ? "Video call" : "Voice call")}
      class="fixed inset-0 z-40 flex flex-col items-center"
      style={{ background: "var(--window-bg)", padding: "24px", top: "28px" }}
      onKeyDown={(e) => e.key === "Escape" && p.call.phase !== "incoming" && commands.callControl("hide")}
    >
      <div data-tauri-drag-region class="absolute inset-x-0 top-0" style={{ height: "40px" }} />
      <Show when={p.call.phase !== "incoming"}>
        <button
          type="button"
          title={t("Hide the call screen")}
          aria-label={t("Hide the call screen")}
          onClick={() => commands.callControl("hide")}
          class="icon-button absolute flex items-center justify-center rounded-full"
          style={{ top: "26px", left: "24px", width: "32px", height: "32px", color: "var(--text-secondary)" }}
        >
          <BackIcon size={20} />
        </button>
        <button
          type="button"
          title={t(full() ? "Exit full screen" : "Full screen")}
          aria-label={t(full() ? "Exit full screen" : "Full screen")}
          onClick={async () => setFull(await commands.callFullscreen())}
          class="icon-button absolute flex items-center justify-center rounded-full"
          style={{ top: "26px", right: "24px", width: "32px", height: "32px", color: "var(--text-secondary)" }}
        >
          <Show when={full()} fallback={<MaximizeIcon size={20} />}><MinimizeIcon size={20} /></Show>
        </button>
      </Show>
      <Show when={devices() && p.call.phase === "live"}>
        <div class="absolute flex flex-wrap justify-center" style={{ bottom: "100px", left: 0, right: 0, gap: "16px" }}>
          <Picker label="Microphone" kind="microphone" list={p.call.microphones} chosen={p.call.microphone} />
          <Picker label="Speaker" kind="speaker" list={p.call.speakers} chosen={p.call.speaker} />
          <Show when={p.call.camera}>
            <Picker label="Camera" kind="camera" list={p.call.cameras} chosen={p.call.cameraDevice} />
          </Show>
        </div>
      </Show>
      <div class="flex flex-col items-center text-center" style={{ "margin-top": "16vh" }}>
        <Avatar src={p.call.avatar} size={Math.round(Math.min(168, Math.max(96, window.innerHeight * 0.26)))} seed={p.call.chat} />
        <h2 class="selectable" style={{ margin: "18px 0 0", "font-size": "26px", "font-weight": 700, color: "var(--text-primary)" }}>{p.call.name}</h2>
        <p aria-live="polite" style={{ margin: "6px 0 0", "font-size": "15px", "font-weight": 500, color: statusColor() }}><Status call={p.call} /></p>
      </div>
      <div class="absolute flex justify-center" style={{ bottom: "24px", left: 0, right: 0, gap: "16px" }}>
        <Show
          when={p.call.phase === "incoming"}
          fallback={
            <Show when={p.call.phase === "live"}>
              <Control
                label={p.call.muted ? "Unmute" : "Mute"}
                fill={p.call.muted ? "var(--text-primary)" : "var(--chip-border)"}
                tint={p.call.muted ? "var(--window-bg)" : "var(--text-primary)"}
                enabled={p.call.connected}
                onClick={() => commands.callControl("mute", !p.call.muted)}
              >
                <Show when={p.call.muted} fallback={<MicIcon size={24} />}><MicOffIcon size={24} /></Show>
              </Control>
              <Show when={p.call.camera}>
                <Control
                  label={p.call.cameraOn ? "Turn the camera off" : p.call.video ? "Turn the camera on" : "Start video"}
                  fill={p.call.cameraOn || !p.call.video ? "var(--chip-border)" : "var(--text-primary)"}
                  tint={p.call.cameraOn || !p.call.video ? "var(--text-primary)" : "var(--window-bg)"}
                  enabled={p.call.connected}
                  onClick={() => commands.callControl("camera", !p.call.cameraOn)}
                >
                  <Show when={p.call.cameraOn || !p.call.video} fallback={<VideoOffIcon size={24} />}><VideoIcon size={24} /></Show>
                </Control>
              </Show>
              <Control
                label="Devices"
                fill={devices() ? "var(--text-primary)" : "var(--chip-border)"}
                tint={devices() ? "var(--window-bg)" : "var(--text-primary)"}
                onClick={() => setDevices(!devices())}
              >
                <Volume2Icon size={24} />
              </Control>
              <Control label="Hang up" fill="var(--danger)" tint="#fff" onClick={() => commands.callControl("hangup")}>
                <PhoneOffIcon size={24} />
              </Control>
            </Show>
          }
        >
          {/* ringing(): the two answers, each named under its button. */}
          <div class="flex flex-col items-center" style={{ gap: "6px", "margin-bottom": "4px" }}>
            <Control label="Decline" fill="var(--danger)" tint="#fff" onClick={() => commands.callControl("decline")}>
              <PhoneOffIcon size={24} />
            </Control>
            <span style={{ "font-size": "13px", "font-weight": 500, color: "var(--text-secondary)" }}>{t("Decline")}</span>
          </div>
          <div class="flex flex-col items-center" style={{ gap: "6px", "margin-left": "52px", "margin-bottom": "4px" }}>
            <Control label="Accept" fill="var(--accent)" tint="#fff" onClick={() => commands.callControl("answer")}>
              <Show when={p.call.video} fallback={<CallsIcon size={24} />}><VideoIcon size={24} /></Show>
            </Control>
            <span style={{ "font-size": "13px", "font-weight": 500, color: "var(--text-secondary)" }}>{t("Accept")}</span>
          </div>
        </Show>
      </div>
    </div>
  );
}

/** call::strip: the call set aside, along the bottom in the accent. */
function Strip(p: { call: CallView }) {
  return (
    <div
      role="status"
      class="fixed inset-x-0 bottom-0 z-30 flex items-center"
      style={{ height: "30px", padding: "0 14px", gap: "8px", background: "var(--accent)", color: "var(--accent-fg)", "font-size": "13px" }}
    >
      <Show when={p.call.video} fallback={<CallsIcon size={15} />}><VideoIcon size={15} /></Show>
      <span style={{ "font-weight": 600 }}>{p.call.name}</span>
      <span style={{ "font-weight": 500 }}><Status call={p.call} /></span>
      <span class="flex-1" />
      <Show when={p.call.phase !== "over"}>
        <button type="button" onClick={() => commands.callControl("return")} style={{ "font-weight": 600, color: "inherit" }}>
          {t("Return to the call")}
        </button>
        <button type="button" title={t("Hang up")} aria-label={t("Hang up")} onClick={() => commands.callControl("hangup")} class="flex" style={{ color: "inherit" }}>
          <PhoneOffIcon size={16} />
        </button>
      </Show>
    </div>
  );
}

export default function CallScreen() {
  return (
    <Show when={state.call}>
      {(call) => (
        <Show when={!call().hidden} fallback={<Strip call={call()} />}>
          <Screen call={call()} />
        </Show>
      )}
    </Show>
  );
}
