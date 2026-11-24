// The message field (REQ-WEBUI-008), floating over the wallpaper as on
// WhatsApp Desktop for macOS: a round + button, a pill field with the
// emoji button inside, and a green round mic or send button.
import { createSignal, Show } from "solid-js";
import { commands } from "../../lib/bridge";
import { EmojiIcon, MicIcon, PlusIcon, SendIcon } from "../../components/Icons";

const round = {
  width: "var(--composer-button)",
  height: "var(--composer-button)",
  "border-radius": "50%",
};

export default function Composer(p: { chat: string }) {
  const [text, setText] = createSignal("");
  let field!: HTMLTextAreaElement;
  const send = () => {
    if (!text().trim()) return;
    commands.sendText(p.chat, text());
    setText("");
    field.style.height = "auto";
  };
  const grow = () => {
    field.style.height = "auto";
    field.style.height = `${Math.min(field.scrollHeight, 120)}px`;
  };
  return (
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
            grow();
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
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
  );
}
