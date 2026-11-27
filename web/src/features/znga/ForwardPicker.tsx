// Picks the chat an answer is forwarded to (REQ-ZANS-003.A2): only chats
// Znga may see are listed, and choosing one puts the answer in that chat's
// composer as a draft. Nothing is sent.
import { createSignal, For, Show } from "solid-js";
import { commands, type ZEntry, type ZLink } from "../../lib/bridge";
import { t } from "../../lib/i18n";
import { CloseIcon } from "../../components/Icons";

export default function ForwardPicker(p: { entry: ZEntry; chats: ZLink[]; onClose: () => void }) {
  const [query, setQuery] = createSignal("");
  const shown = () => {
    const q = query().trim().toLowerCase();
    return p.chats.filter((chat) => !q || chat.title.toLowerCase().includes(q)).slice(0, 60);
  };
  const pick = (chat: ZLink) => {
    void commands.zngaForward(p.entry.id, chat.id);
    p.onClose();
  };
  return (
    <div
      class="fixed inset-0 flex items-center justify-center"
      style={{ background: "rgba(0,0,0,0.35)", "z-index": 50 }}
      onClick={(e) => e.target === e.currentTarget && p.onClose()}
      onKeyDown={(e) => e.key === "Escape" && p.onClose()}
    >
      <div role="dialog" aria-label={t("Forward to")} class="flex flex-col" style={{ width: "340px", "max-height": "70vh", background: "var(--surface)", "border-radius": "12px", "box-shadow": "0 8px 30px rgba(0,0,0,0.25)" }}>
        <div class="flex items-center" style={{ padding: "12px 14px 6px", gap: "8px" }}>
          <div class="flex-1" style={{ "font-weight": 600, color: "var(--text-primary)" }}>{t("Forward to")}</div>
          <button type="button" aria-label={t("Close")} onClick={p.onClose} style={{ color: "var(--text-secondary)" }}><CloseIcon size={16} /></button>
        </div>
        <div style={{ padding: "0 14px 4px", "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>
          {t("The answer goes into the chat's message box. You send it.")}
        </div>
        <input
          aria-label={t("Search chats")}
          placeholder={t("Search chats")}
          value={query()}
          ref={(el) => queueMicrotask(() => el.focus())}
          onInput={(e) => setQuery(e.currentTarget.value)}
          onKeyDown={(e) => e.key === "Enter" && shown()[0] && pick(shown()[0])}
          class="selectable outline-none"
          style={{ margin: "6px 14px", height: "30px", padding: "0 10px", "border-radius": "8px", border: "1px solid var(--divider)", background: "var(--input-bg)", color: "var(--text-primary)" }}
        />
        <div class="scroll min-h-0 flex-1" style={{ padding: "0 6px 8px" }}>
          <For each={shown()}>
            {(chat) => (
              <button type="button" onClick={() => pick(chat)} class="menu-item block w-full truncate text-left" style={{ padding: "7px 10px", "border-radius": "7px", color: "var(--text-primary)", "font-size": "14px" }}>
                {chat.title}
              </button>
            )}
          </For>
          <Show when={shown().length === 0}>
            <div style={{ padding: "10px", "font-size": "13px", color: "var(--text-secondary)" }}>{t("No chats found")}</div>
          </Show>
        </div>
      </div>
    </div>
  );
}
