// The Znga chat (REQ-ZCHAT-001, 002): a conversation with the AI that
// lives only on this computer. The person's questions and instructions on
// the right, Znga's answers and findings on the left; nothing written here
// reaches WhatsApp.
import { createEffect, createSignal, For, on, Show } from "solid-js";
import { commands, state, type ZngaChat as Chat } from "../../lib/bridge";
import { clock, dayKey, dayLabel } from "../../lib/time";
import { t } from "../../lib/i18n";
import ZngaAvatar from "../../components/ZngaAvatar";
import { SendIcon } from "../../components/Icons";
import ZngaCard from "./ZngaCard";

function Bubble(p: { out: boolean; children: import("solid-js").JSX.Element }) {
  return (
    <div class="flex" style={{ "justify-content": p.out ? "flex-end" : "flex-start", padding: "0 var(--bubble-gutter)", "margin-top": "var(--bubble-gap-run)" }}>
      <div
        class="selectable"
        style={{
          "max-width": "var(--bubble-max)",
          padding: "var(--bubble-pad-y) var(--bubble-pad-x)",
          "border-radius": "var(--bubble-radius)",
          background: p.out ? "var(--bubble-out)" : "var(--bubble-in)",
          "box-shadow": "var(--shadow-bubble)",
          "font-size": "var(--fs-msg)",
          "line-height": "var(--lh-msg)",
          color: "var(--text-primary)",
        }}
      >
        {p.children}
      </div>
    </div>
  );
}

const SUGGESTIONS = ["Do I owe anyone a reply?", "Brief me on today", "Tell … I'll call at 5", "Remind me to … on Friday"];

export default function ZngaChat(p: { chat: Chat }) {
  let list!: HTMLDivElement;
  let field!: HTMLTextAreaElement;
  const [text, setText] = createSignal("");
  createEffect(
    on(
      () => [p.chat.entries.length, p.chat.writing, p.chat.entries.at(-1)?.text],
      () => queueMicrotask(() => (list.scrollTop = list.scrollHeight)),
    ),
  );
  const send = () => {
    const value = text().trim();
    if (!value) return;
    void commands.zngaSend(value);
    setText("");
    field.style.height = "auto";
  };
  const grow = () => {
    field.style.height = "auto";
    field.style.height = `${Math.min(field.scrollHeight, 120)}px`;
  };
  return (
    <section class="flex h-full min-w-0 flex-1 flex-col" aria-label="Znga">
      <header
        data-tauri-drag-region
        class="flex shrink-0 items-center"
        style={{
          height: "var(--conv-header-h)",
          "padding-left": "var(--bubble-gutter)",
          background: "var(--conversation-header-bg)",
          "border-bottom": "var(--hairline) solid var(--conversation-header-border)",
        }}
      >
        <ZngaAvatar size={30} />
        <div data-tauri-drag-region class="min-w-0" style={{ "margin-left": "7.5px" }}>
          <div style={{ "font-size": "var(--fs-header-title)", "font-weight": 600, "line-height": "16px", color: "var(--text-primary)" }}>Znga</div>
          <div style={{ "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>{t("Your AI companion. This chat stays on this computer.")}</div>
        </div>
      </header>
      <div class="relative flex min-h-0 flex-1 flex-col" style={{ background: "var(--conversation-bg)" }}>
        <div ref={list} class="scroll relative min-h-0 flex-1" style={{ "padding-bottom": "8px" }} role="log" aria-label="Messages">
          <Show when={p.chat.entries.length === 0 && p.chat.writing === null}>
            <div class="flex flex-col items-center" style={{ padding: "48px 24px", gap: "10px", color: "var(--text-secondary)", "text-align": "center" }}>
              <ZngaAvatar size={56} />
              <div style={{ "font-size": "var(--fs-row-text)" }}>
                {t("Ask about your chats, or tell Znga what to prepare. Znga drafts; you send.")}
              </div>
              <div class="flex flex-wrap justify-center" style={{ gap: "6px" }}>
                <For each={SUGGESTIONS}>
                  {(s) => (
                    <button
                      type="button"
                      class="rounded-full"
                      onClick={() => {
                        setText(s.replace("…", "").replace("  ", " "));
                        field.focus();
                      }}
                      style={{ height: "26px", padding: "0 12px", background: "var(--round-button-bg)", color: "var(--text-primary)", "font-size": "var(--fs-meta)" }}
                    >
                      {t(s)}
                    </button>
                  )}
                </For>
              </div>
            </div>
          </Show>
          <For each={p.chat.entries}>
            {(entry, i) => {
              const prev = () => p.chat.entries[i() - 1];
              const newDay = () => !prev() || dayKey(prev()!.at) !== dayKey(entry.at);
              return (
                <>
                  <Show when={newDay()}>
                    <div class="flex justify-center" style={{ margin: "var(--date-chip-gap) 0 0" }}>
                      <span
                        class="flex items-center justify-center"
                        style={{
                          height: "var(--date-chip-h)",
                          padding: "0 12px",
                          "border-radius": "calc(var(--date-chip-h) / 2)",
                          background: "var(--date-chip-bg)",
                          "box-shadow": "var(--shadow-bubble)",
                          "font-size": "var(--fs-date-chip)",
                          "font-weight": 600,
                          color: "var(--text-primary)",
                        }}
                      >
                        {dayLabel(entry.at)}
                      </span>
                    </div>
                  </Show>
                  <Bubble out={entry.fromMe}>
                    <Show when={entry.unasked}>
                      <div style={{ "font-size": "var(--fs-meta)", "font-weight": 600, color: "var(--accent-text)" }}>✨ {t("Znga noticed")}</div>
                    </Show>
                    <span style={{ "white-space": "pre-wrap" }}>{entry.text}</span>
                    <For each={entry.cards}>{(card, index) => <ZngaCard card={card} entry={entry.id} index={index()} />}</For>
                    <div style={{ "text-align": "right", "font-size": "var(--fs-meta)", color: "var(--text-secondary)", "margin-top": "2px" }}>{clock(entry.at)}</div>
                  </Bubble>
                </>
              );
            }}
          </For>
          <Show when={p.chat.writing !== null}>
            <Bubble out={false}>
              <span style={{ "white-space": "pre-wrap" }}>{p.chat.writing || t("Znga is thinking…")}</span>
              <div style={{ "margin-top": "6px" }}>
                <button
                  type="button"
                  onClick={() => void commands.zngaStop()}
                  class="rounded-full"
                  style={{ height: "22px", padding: "0 10px", "font-size": "var(--fs-meta)", background: "var(--round-button-bg)", color: "var(--text-primary)" }}
                >
                  {t("Stop")}
                </button>
              </div>
            </Bubble>
          </Show>
        </div>
        <footer class="flex shrink-0 items-end" style={{ gap: "var(--composer-gap)", padding: "4px var(--composer-pad-x) 3px" }}>
          <div
            class="flex flex-1 items-end"
            style={{
              "min-height": "var(--composer-input-h)",
              "margin-bottom": "3px",
              "border-radius": "calc(var(--composer-input-h) / 2)",
              background: "var(--input-bg)",
              border: "var(--hairline) solid var(--input-border)",
              padding: "0 12px",
            }}
          >
            <textarea
              ref={field}
              rows={1}
              aria-label="Message Znga"
              placeholder={p.chat.asking ? t("Answer Znga's question") : t("Ask Znga or tell it what to prepare")}
              value={text()}
              onInput={(e) => {
                setText(e.currentTarget.value);
                grow();
              }}
              onKeyDown={(e) => {
                const sends = state.settings.enterSends ? !e.shiftKey : e.metaKey || e.ctrlKey;
                if (e.key === "Enter" && sends) {
                  e.preventDefault();
                  send();
                }
              }}
              class="selectable flex-1 resize-none bg-transparent outline-none"
              style={{ padding: "5px 0", "font-size": "var(--fs-input)", "line-height": "var(--lh-msg)", color: "var(--text-primary)", "max-height": "120px", "caret-color": "var(--accent)" }}
            />
          </div>
          <button
            type="button"
            aria-label="Send to Znga"
            onClick={send}
            class="flex shrink-0 items-center justify-center"
            style={{ width: "var(--composer-button)", height: "var(--composer-button)", "border-radius": "50%", background: "var(--accent)", color: "var(--accent-fg)", opacity: text().trim() ? 1 : 0.5 }}
          >
            <SendIcon size={17} />
          </button>
        </footer>
      </div>
    </section>
  );
}
