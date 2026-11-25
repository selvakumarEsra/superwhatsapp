// The window: sidebar, chat list, conversation and info panel, as on
// WhatsApp Desktop for macOS (REQ-WEBUI-004).
import { createSignal, onMount, Show } from "solid-js";
import { commands, connect, state } from "./lib/bridge";
import { ask, asking } from "./lib/ui";
import Sidebar, { type Section } from "./components/Sidebar";
import { Confirm } from "./components/Menu";
import ChatList from "./features/chats/ChatList";
import Conversation from "./features/conversation/Conversation";
import InfoPanel from "./features/info/InfoPanel";
import EmptyPane from "./features/EmptyPane";
import Link from "./features/Link";

/** Znga and Insights, which the new interface does not draw yet. */
function Pending(p: { name: string }) {
  return (
    <section
      class="flex h-full shrink-0 flex-col items-center justify-center px-[32px] text-center"
      style={{ width: "var(--list-w)", background: "var(--panel-bg)", "border-right": "var(--hairline) solid var(--panel-border)" }}
    >
      <p style={{ "font-size": "var(--fs-row-text)", "line-height": "16px", color: "var(--text-secondary)" }}>
        {p.name} is not in the new interface yet. It still works in the current ZapFast app.
      </p>
    </section>
  );
}

export default function App() {
  onMount(() => void connect());
  // Znga and Insights are chosen here; the other sections come from the core.
  const [panel, setPanel] = createSignal<"znga" | "insights" | null>(null);
  const section = (): Section => panel() ?? state.section;
  const choose = (next: Section) => {
    if (next === "znga" || next === "insights") {
      setPanel(next);
      return;
    }
    setPanel(null);
    void commands.showSection(next);
  };
  const linked = () => state.chats.length > 0 || ["connected", "connecting", "disconnected"].includes(state.link.state);
  const unread = () => state.chats.reduce((n, c) => n + c.unread, 0);
  return (
    <Show when={linked()} fallback={<Link link={state.link} />}>
      <div class="flex h-full w-full">
        <Sidebar section={section()} onSection={choose} unread={unread()} />
        <Show when={panel()} fallback={<ChatList />}>
          {(name) => <Pending name={name() === "znga" ? "Znga" : "Insights"} />}
        </Show>
        <Show when={state.open} fallback={<EmptyPane />}>
          <Conversation chat={state.open!} />
        </Show>
        <InfoPanel />
      </div>
      <Show when={asking()}>
        {(q) => <Confirm title={q().title} body={q().body} action={q().action} danger onConfirm={q().run} onClose={() => ask(null)} />}
      </Show>
    </Show>
  );
}
