// The window: sidebar, chat list and conversation, as on WhatsApp Desktop
// for macOS (REQ-WEBUI-004).
import { createSignal, onMount, Show } from "solid-js";
import { connect, state } from "./lib/bridge";
import Sidebar, { type Section } from "./components/Sidebar";
import ChatList from "./features/chats/ChatList";
import Conversation from "./features/conversation/Conversation";
import EmptyPane from "./features/EmptyPane";
import Link from "./features/Link";

export default function App() {
  onMount(() => void connect());
  const [section, setSection] = createSignal<Section>("chats");
  const linked = () => state.chats.length > 0 || ["connected", "connecting", "disconnected"].includes(state.link.state);
  const unread = () => state.chats.reduce((n, c) => n + c.unread, 0);
  return (
    <Show when={linked()} fallback={<Link link={state.link} />}>
      <div class="flex h-full w-full">
        <Sidebar section={section()} onSection={setSection} unread={unread()} />
        <ChatList />
        <Show when={state.open} fallback={<EmptyPane />}>
          <Conversation chat={state.open!} />
        </Show>
      </div>
    </Show>
  );
}
