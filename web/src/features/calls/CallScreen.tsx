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
  const ringing = () => p.call.phase === "incoming";
  const statusColor = () => (p.call.failed ? "var(--danger)" : p.call.phase === "live" ? "var(--accent-text)" : "var(--text-secondary)");
  return (
    <div
      role="dialog"
      aria-label={t(p.call.video ? "Video call" : "Voice call")}
      class="fixed inset-0 z-40 flex flex-col items-center"
      style={{ background: p.call.remoteFrame ? "#000" : "var(--window-bg)", padding: "24px", top: "28px" }}
      onKeyDown={(e) => e.key === "Escape" && p.call.phase !== "incoming" && commands.callControl("hide")}
    >
      {/* call.rs:screen: the peer's picture fills the window, fitted
          inside it, and everything else is drawn over it. */}
      <Show when={p.call.remoteFrame}>
        <img src={p.call.remoteFrame!} alt="" class="absolute inset-0 h-full w-full" style={{ "object-fit": "contain" }} />
      </Show>
      {/* call.rs:own_preview: our camera in the top-right corner, at its own
          shape within 24% of the screen, 48 down, rounded 12, outlined. */}
      <Show when={p.call.localFrame}>
        <img
          src={p.call.localFrame!}
          alt={t("Your camera")}
          class="absolute"
          style={{ top: "72px", right: "24px", "max-width": "calc((100% - 48px) * 0.24)", "max-height": "calc((100% - 48px) * 0.24)", "border-radius": "12px", outline: "1px solid var(--chip-border)" }}
        />
      </Show>
      {/* Over a picture, the name (semibold 15) and the length (medium 13)
          sit on a dark plate of their own, as wide as they are. */}
      <Show when={p.call.remoteFrame}>
        <div
          class="absolute flex flex-col items-center"
          style={{ top: "28px", left: "50%", transform: "translateX(-50%)", padding: "6px 14px", "border-radius": "14px", background: "rgba(0,0,0,0.59)", color: "#fff", "line-height": "18px" }}
        >
          <span style={{ "font-size": "15px", "font-weight": 600 }}>{p.call.name}</span>
          <span style={{ "font-size": "13px", "font-weight": 500 }}><Status call={p.call} /></span>
        </div>
      </Show>
      <div data-tauri-drag-region class="absolute inset-x-0 top-0" style={{ height: "40px" }} />
      <Show when={p.call.phase !== "incoming"}>
        <button
          type="button"
          title={t("Hide the call screen")}
          aria-label={t("Hide the call screen")}
          onClick={() => commands.callControl("hide")}
          class="icon-button absolute flex items-center justify-center rounded-full"
          style={{ top: "26px", left: "24px", width: "32px", height: "32px", color: p.call.remoteFrame ? "rgba(255,255,255,0.78)" : "var(--text-secondary)" }}
        >
          <BackIcon size={20} />
        </button>
        <button
          type="button"
          title={t(full() ? "Exit full screen" : "Full screen")}
          aria-label={t(full() ? "Exit full screen" : "Full screen")}
          onClick={async () => setFull(await commands.callFullscreen())}
          class="icon-button absolute flex items-center justify-center rounded-full"
          style={{ top: "26px", right: "24px", width: "32px", height: "32px", color: p.call.remoteFrame ? "rgba(255,255,255,0.78)" : "var(--text-secondary)" }}
        >
          {/* call.rs:corner_buttons: white over a picture, the secondary ink otherwise. */}
          <Show when={full()} fallback={<MaximizeIcon size={18} />}><MinimizeIcon size={18} /></Show>
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
      {/* call.rs: live() at 16% of the area inside the padding, ringing() at 18%. */}
      <div class="relative flex flex-col items-center text-center" style={{ "margin-top": `calc(${ringing() ? 0.18 : 0.16} * (100vh - 76px))`, visibility: p.call.remoteFrame ? "hidden" : undefined }}>
        <Avatar src={p.call.avatar} size={Math.round(Math.min(ringing() ? 160 : 168, Math.max(96, (window.innerHeight - 76) * (ringing() ? 0.24 : 0.26))))} seed={p.call.chat} />
        <h2 class="selectable" style={{ margin: "20px 0 0", "font-size": "26px", "font-weight": 700, color: "var(--text-primary)" }}>{p.call.name}</h2>
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
          <div class="flex flex-col items-center" style={{ width: "56px", gap: "6px", "margin-bottom": "1px" }}>
            <Control label="Decline" fill="var(--danger)" tint="#fff" onClick={() => commands.callControl("decline")}>
              <PhoneOffIcon size={24} />
            </Control>
            <span style={{ "font-size": "13px", "font-weight": 500, color: "var(--text-secondary)" }}>{t("Decline")}</span>
          </div>
          <div class="flex flex-col items-center" style={{ width: "56px", gap: "6px", "margin-left": "56px", "margin-bottom": "1px" }}>
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
