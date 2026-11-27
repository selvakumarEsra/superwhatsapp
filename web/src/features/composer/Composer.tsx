// The message field (REQ-WEBUI-008), floating over the wallpaper as on
// WhatsApp Desktop for macOS: a round + button, a pill field with the
// emoji button inside, and a green round mic or send button.
import { createEffect, createSignal, For, on, Show } from "solid-js";
import { commands, state } from "../../lib/bridge";
import { EmojiIcon, MicIcon, PlusIcon, SendIcon, SparkleIcon } from "../../components/Icons";
import { Menu, type Entry } from "../../components/Menu";
import { t } from "../../lib/i18n";

const LANGUAGES = ["English", "Spanish", "French", "German", "Portuguese", "Italian", "Dutch", "Hindi", "Tamil", "Arabic", "Chinese", "Japanese"];

/** A thin bar above the field: Znga's offer, or Undo after taking one. */
function Bar(p: { children: import("solid-js").JSX.Element }) {
  return (
    <div
      class="flex items-center"
      style={{
        gap: "8px",
        margin: "0 var(--composer-pad-x) 4px",
        padding: "6px 10px",
        "border-radius": "10px",
        background: "var(--input-bg)",
        border: "var(--hairline) solid var(--input-border)",
        "font-size": "var(--fs-row-text)",
        color: "var(--text-primary)",
      }}
    >
      {p.children}
    </div>
  );
}

const small = { height: "22px", padding: "0 10px", "border-radius": "11px", "font-size": "var(--fs-meta)", "font-weight": 600 } as const;

const round = {
  width: "var(--composer-button)",
  height: "var(--composer-button)",
  "border-radius": "50%",
};

export default function Composer(p: { chat: string; znga?: boolean }) {
  const [text, setText] = createSignal("");
  const [menu, setMenu] = createSignal<{ x: number; y: number } | null>(null);
  const [translating, setTranslating] = createSignal(false);
  const [language, setLanguage] = createSignal("English");
  // The text before Znga replaced it, while Undo is offered.
  const [undo, setUndo] = createSignal<string | null>(null);
  let field!: HTMLTextAreaElement;
  const send = () => {
    if (!text().trim()) return;
    commands.sendText(p.chat, text());
    setText("");
    setUndo(null);
    field.style.height = "auto";
  };
  // Znga's suggestion or rewrite for this chat (REQ-ZCHAT-004). A
  // suggested reply fills an empty field; anything else waits for Use.
  const offer = () => (state.offer && state.offer.chat === p.chat ? state.offer : null);
  const take = (replacement: string, id: number) => {
    setUndo(text());
    setText(replacement);
    void commands.zngaOfferDone(id);
    queueMicrotask(grow);
  };
  createEffect(
    on(offer, (o) => {
      if (o && !o.rewrite && !text().trim()) take(o.text, o.id);
    }),
  );
  // As in egui, opening a chat puts the cursor in its composer.
  createEffect(on(() => p.chat, () => queueMicrotask(() => field?.focus({ preventScroll: true }))));
  const tool = (id: string) => void commands.zngaTool(p.chat, id, text(), id === "translate" ? language() : "");
  const empty = () => !text().trim();
  const entries = (): Entry[] => [
    { label: "Suggest reply", run: () => tool("suggest") },
    { label: "Catch me up", run: () => tool("catchUp") },
    { label: "Find plans", run: () => tool("plans") },
    { label: "Friendlier", run: () => tool("friendlier"), disabled: empty() },
    { label: "Shorter", run: () => tool("shorter"), disabled: empty() },
    { label: "More formal", run: () => tool("formal"), disabled: empty() },
    { label: "Fix spelling", run: () => tool("spelling"), disabled: empty() },
    { label: "Translate…", run: () => setTranslating(true), disabled: empty() },
  ];
  const grow = () => {
    field.style.height = "auto";
    field.style.height = `${Math.min(field.scrollHeight, 120)}px`;
  };
  return (
    <>
    <Show when={offer() && (offer()!.rewrite || !empty())}>
      <Bar>
        <span style={{ color: "var(--accent-text)" }}><SparkleIcon size={15} /></span>
        <span class="selectable min-w-0 flex-1" style={{ "white-space": "pre-wrap" }}>{offer()!.text}</span>
        <button type="button" onClick={() => take(offer()!.text, offer()!.id)} style={{ ...small, background: "var(--accent)", color: "var(--accent-fg)" }}>{t("Use")}</button>
        <button type="button" onClick={() => void commands.zngaOfferDone(offer()!.id)} style={{ ...small, background: "var(--round-button-bg)" }}>{t("Dismiss")}</button>
      </Bar>
    </Show>
    <Show when={undo() !== null && !offer()}>
      <Bar>
        <span style={{ color: "var(--accent-text)" }}><SparkleIcon size={15} /></span>
        <span class="flex-1" style={{ color: "var(--text-secondary)" }}>{t("Znga changed your message. Nothing was sent.")}</span>
        <button type="button" onClick={() => { setText(undo()!); setUndo(null); queueMicrotask(grow); }} style={{ ...small, background: "var(--round-button-bg)" }}>{t("Undo")}</button>
      </Bar>
    </Show>
    <Show when={translating()}>
      <Bar>
        <span>{t("Translate into")}</span>
        <select value={language()} onChange={(e) => setLanguage(e.currentTarget.value)} aria-label="Language" style={{ height: "22px", "border-radius": "6px", background: "var(--search-bg)", color: "var(--text-primary)", border: "none" }}>
          <For each={LANGUAGES}>{(l) => <option value={l}>{l}</option>}</For>
        </select>
        <span class="flex-1" />
        <button type="button" disabled={empty()} onClick={() => { tool("translate"); setTranslating(false); }} style={{ ...small, background: "var(--accent)", color: "var(--accent-fg)", opacity: empty() ? 0.4 : 1 }}>{t("Translate")}</button>
        <button type="button" onClick={() => setTranslating(false)} style={{ ...small, background: "var(--round-button-bg)" }}>{t("Cancel")}</button>
      </Bar>
    </Show>
    <Show when={menu()}>
      {(at) => <Menu x={at().x} y={at().y} entries={entries()} onClose={() => setMenu(null)} />}
    </Show>
    <footer class="relative shrink-0" style={{ padding: "0 8px 0 9px", color: "var(--icon)" }}>
      {/* egui's composer (src/ui/conversation.rs): one white pill holding
          attach, emoji, the field, Znga and the mic, outlined in the accent
          while it has focus; the shortcut line under it. */}
      <div
        class="composer-pill flex items-end"
        style={{
          "min-height": "41px",
          "border-radius": "20.5px",
          background: "var(--input-bg)",
          border: "1.5px solid var(--input-border)",
          padding: "0 4px 0 8px",
        }}
      >
        <button type="button" title={t("Attach")} aria-label="Attach" class="icon-button flex shrink-0 items-center justify-center rounded-full" style={{ ...round, margin: "4px 0", color: "var(--icon)" }}>
          <PlusIcon size={20} />
        </button>
        <button type="button" title={t("Emoji")} aria-label="Emoji" class="icon-button flex shrink-0 items-center justify-center rounded-full" style={{ ...round, margin: "4px 0", color: "var(--icon)" }}>
          <EmojiIcon size={21} />
        </button>
        <textarea
          ref={field}
          rows={1}
          aria-label="Type a message"
          placeholder={t("Type a message")}
          value={text()}
          onInput={(e) => {
            setText(e.currentTarget.value);
            setUndo(null);
            grow();
          }}
          onKeyDown={(e) => {
            // Settings › Chats › Enter sends; otherwise ⌘/Ctrl+Enter sends.
            const sends = state.settings.enterSends ? !e.shiftKey : e.metaKey || e.ctrlKey;
            if (e.key === "Enter" && sends) {
              e.preventDefault();
              send();
            }
          }}
          class="selectable flex-1 resize-none bg-transparent outline-none"
          style={{
            padding: "9.5px 0 9.5px 4px",
            "font-size": "var(--fs-input)",
            "line-height": "19px",
            color: "var(--text-primary)",
            "max-height": "140px",
            "caret-color": "var(--accent)",
          }}
        />
        <Show when={p.znga}>
          <button
            type="button"
            title="Znga"
            aria-label="Znga"
            aria-haspopup="menu"
            onClick={(e) => {
              const box = e.currentTarget.getBoundingClientRect();
              setMenu({ x: box.left - 90, y: box.top - 8 * 27 - 20 });
            }}
            class="icon-button flex shrink-0 items-center justify-center rounded-full"
            style={{ ...round, margin: "4px 0", color: "var(--accent-text)" }}
          >
            <SparkleIcon size={19} />
          </button>
        </Show>
        <button
          type="button"
          title={text().trim() ? t("Send") : t("Voice message")}
          aria-label={text().trim() ? "Send" : "Voice message"}
          onClick={send}
          class="flex shrink-0 items-center justify-center rounded-full"
          style={{
            ...round,
            margin: "4px 0",
            background: text().trim() ? "var(--accent)" : "var(--search-bg)",
            color: text().trim() ? "var(--accent-fg)" : "var(--icon)",
          }}
        >
          <Show when={text().trim()} fallback={<MicIcon size={19} />}>
            <SendIcon size={17} />
          </Show>
        </button>
      </div>
      <Show
        when={state.settings.showShortcutHints}
        fallback={<div style={{ height: "9px" }} />}
      >
        <div class="flex items-center" style={{ height: "24px", margin: "10px 0", gap: "6px", padding: "0 2px 0 6px", "font-size": "11px", color: "var(--text-meta)" }}>
          <button
            type="button"
            title={t("Hide shortcut hints (bring them back from Keyboard shortcuts)")}
            aria-label="Hide shortcut hints"
            onClick={() => commands.setSetting("showShortcutHints", false)}
            class="flex items-center justify-center"
            style={{ width: "16px", height: "16px", color: "var(--text-meta)" }}
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
          </button>
          <span class="min-w-0 flex-1 truncate">
            {state.settings.enterSends
              ? t("Enter sends · Shift+Enter for a new line · *bold* _italic_ ~strike~ · ⌘V pastes a picture")
              : t("⌘Enter sends · *bold* _italic_ ~strike~ · ⌘V pastes a picture")}
          </span>
          <span aria-hidden="true" style={{ color: "var(--text-meta)" }}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"><rect x="2.5" y="6" width="19" height="12" rx="2" /><path d="M6 10h.01M9.5 10h.01M13 10h.01M16.5 10h.01M8 14h8" stroke-linecap="round" /></svg>
          </span>
        </div>
      </Show>
    </footer>
    </>
  );
}
