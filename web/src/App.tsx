// The window: sidebar, chat list, conversation and info panel, as on
// WhatsApp Desktop for macOS (REQ-WEBUI-004).
import { createEffect, createSignal, onMount, Show } from "solid-js";
import { commands, connect, state, ZNGA_CHAT } from "./lib/bridge";
import ZngaChat from "./features/znga/ZngaChat";
import { ask, asking } from "./lib/ui";
import Sidebar, { type Section } from "./components/Sidebar";
import { Confirm } from "./components/Menu";
import ChatList from "./features/chats/ChatList";
import Conversation from "./features/conversation/Conversation";
import InfoPanel from "./features/info/InfoPanel";
import EmptyPane from "./features/EmptyPane";
import Settings from "./features/settings/Settings";
import Link from "./features/Link";
import LockScreen from "./features/LockScreen";

/** Insights, which the new interface does not draw yet. */
function Pending(p: { name: string }) {
  return (
    <section
      class="flex h-full shrink-0 flex-col items-center justify-center px-[32px] text-center"
      style={{ width: "var(--list-w)", background: "var(--panel-bg)", "border-right": "var(--hairline) solid var(--panel-border)" }}
    >
      <p style={{ "font-size": "var(--fs-row-text)", "line-height": "16px", color: "var(--text-secondary)" }}>
        {p.name} is not in the new interface yet. It still works in the current SuperWhatsapp app.
      </p>
    </section>
  );
}

export default function App() {
  onMount(() => void connect());
  // Znga and Insights are chosen here; the other sections come from the core.
  const [panel, setPanel] = createSignal<"insights" | "settings" | null>(null);
  // Outside Tauri, `#settings` opens Settings, for screenshots.
  if (!("__TAURI_INTERNALS__" in window) && location.hash.startsWith("#settings")) setPanel("settings");
  const section = (): Section | "settings" => panel() ?? (state.znga ? "znga" : state.section);
  // The chosen theme and zoom apply to the whole page (REQ-WEBUI-012).
  createEffect(() => {
    const root = document.documentElement;
    if (state.settings.theme === "system") root.removeAttribute("data-theme");
    else root.setAttribute("data-theme", state.settings.theme);
    root.style.setProperty("zoom", String(state.settings.zoom));
  });
  // A custom theme's colours override the tokens while it is in use.
  let applied: string[] = [];
  createEffect(() => {
    const root = document.documentElement;
    for (const name of applied) root.style.removeProperty(name);
    applied = [];
    for (const [name, value] of state.settings.palette ?? []) {
      root.style.setProperty(name, value);
      applied.push(name);
    }
  });
  // Keyboard and pointer use restarts the app lock's idle timer; at most
  // one report every few seconds.
  onMount(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    let last = 0;
    const used = () => {
      const now = Date.now();
      if (now - last < 5000) return;
      last = now;
      void commands.activity();
    };
    for (const kind of ["keydown", "pointerdown", "pointermove", "wheel"]) {
      window.addEventListener(kind, used, { passive: true });
    }
  });
  const choose = (next: Section | "settings") => {
    // ✨ opens the Znga chat; there is no separate Znga window
    // (REQ-ZCHAT-001.A6).
    if (next === "znga") {
      setPanel(null);
      void commands.showSection("chats");
      void commands.openChat(ZNGA_CHAT);
      return;
    }
    if (next === "insights" || next === "settings") {
      setPanel(next);
      return;
    }
    setPanel(null);
    void commands.showSection(next);
  };
  const linked = () => state.chats.length > 0 || ["connected", "connecting", "disconnected"].includes(state.link.state);
  return (
    <Show when={!state.lock} fallback={<LockScreen lock={state.lock!} />}>
    <Show when={linked()} fallback={<Link link={state.link} />}>
      <div class="flex h-full w-full">
        <Sidebar section={section()} onSection={choose} unread={state.unreadChats} />
        <Show
          when={panel() === "settings"}
          fallback={
            <>
              <Show when={panel()} fallback={<ChatList />}>
                <Pending name="Insights" />
              </Show>
              <Show when={state.znga} fallback={
                <Show when={state.open} fallback={<EmptyPane />}>
                  <Conversation chat={state.open!} />
                </Show>
              }>
                {(chat) => <ZngaChat chat={chat()} />}
              </Show>
              <InfoPanel />
            </>
          }
        >
          <Settings />
        </Show>
      </div>
      <Show when={asking()}>
        {(q) => <Confirm title={q().title} body={q().body} action={q().action} danger onConfirm={q().run} onClose={() => ask(null)} />}
      </Show>
    </Show>
    </Show>
  );
}
