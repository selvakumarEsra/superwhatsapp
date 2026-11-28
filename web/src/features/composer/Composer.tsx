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
    <footer
      class="relative flex shrink-0 items-end"
      style={{ gap: "var(--composer-gap)", padding: "4px var(--composer-pad-x) 3px", color: "var(--icon)" }}
    >
      <button
        type="button"
        aria-label="Attach"
        class="flex shrink-0 items-center justify-center"
        style={{ ...round, background: "var(--composer-button-bg)", border: "var(--hairline) solid var(--input-border)", color: "var(--text-primary)" }}
      >
        <PlusIcon size={19} />
      </button>
      <div
        class="flex flex-1 items-end"
        style={{
          "min-height": "var(--composer-input-h)",
          "margin-bottom": "3px",
          "border-radius": "calc(var(--composer-input-h) / 2)",
          background: "var(--input-bg)",
          border: "var(--hairline) solid var(--input-border)",
          "padding-left": "12px",
          "padding-right": "6px",
        }}
      >
        <textarea
          ref={field}
          rows={1}
          aria-label="Type a message"
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
            padding: "5px 0",
            "font-size": "var(--fs-input)",
            "line-height": "var(--lh-msg)",
            color: "var(--text-primary)",
            "max-height": "120px",
            "caret-color": "var(--accent)",
          }}
        />
        <Show when={p.znga}>
          <button
            type="button"
            aria-label="Znga"
            aria-haspopup="menu"
            onClick={(e) => {
              const box = e.currentTarget.getBoundingClientRect();
              setMenu({ x: box.left - 90, y: box.top - 8 * 27 - 20 });
            }}
            class="flex items-center justify-center"
            style={{ height: "var(--composer-input-h)", width: "26px", color: "var(--accent-text)" }}
          >
            <SparkleIcon size={18} />
          </button>
        </Show>
        <button type="button" aria-label="Emoji" class="flex items-center justify-center" style={{ height: "var(--composer-input-h)", width: "24px" }}>
          <EmojiIcon size={18} />
        </button>
      </div>
      <button
        type="button"
        aria-label={text().trim() ? "Send" : "Voice message"}
        onClick={send}
        class="flex shrink-0 items-center justify-center"
        style={{ ...round, background: "var(--accent)", color: "var(--accent-fg)" }}
      >
        <Show when={text().trim()} fallback={<MicIcon size={19} />}>
          <SendIcon size={17} />
        </Show>
      </button>
    </footer>
    </>
  );
}
