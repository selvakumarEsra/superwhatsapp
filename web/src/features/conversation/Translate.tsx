// Translate on someone else's message (REQ-ZANS-004): a hover button
// beside a message that looks like another language, and Znga's
// translation in a popup under it. The chat and WhatsApp are unchanged.
import { createSignal, Show } from "solid-js";
import { commands, state, type Msg } from "../../lib/bridge";
import { t } from "../../lib/i18n";
import { CloseIcon, CopyIcon, TranslateIcon } from "../../components/Icons";

export default function Translate(p: { chat: string; msg: Msg; part: "button" | "popup" }) {
  const [copied, setCopied] = createSignal(false);
  const shown = () => {
    const x = state.translation;
    return x && x.chat === p.chat && x.message === p.msg.id ? x : null;
  };
  return (
    <>
      <Show when={p.part === "button" && p.msg.translatable && !shown()}>
        <button
          type="button"
          class="msg-actions absolute flex items-center justify-center rounded-full"
          title={t("Translate with Znga")}
          aria-label={t("Translate with Znga")}
          onClick={() => void commands.zngaTranslate(p.chat, p.msg.id, "open")}
          style={{ left: "calc(100% + 6px)", top: "50%", transform: "translateY(-50%)", width: "28px", height: "28px", background: "var(--round-button-bg)", color: "var(--text-primary)", "box-shadow": "var(--shadow-bubble)" }}
        >
          <TranslateIcon size={15} />
        </button>
      </Show>
      <Show when={p.part === "popup" && shown()}>
        {(x) => (
          <div
            role="dialog"
            aria-label={t("Translation")}
            class="selectable"
            style={{ "margin-top": "6px", width: "min(340px, 80vw)", padding: "10px 12px", "border-radius": "10px", background: "var(--surface)", "box-shadow": "0 6px 24px rgba(0,0,0,0.22)", border: "var(--hairline) solid var(--divider)", "font-size": "14px", "line-height": "19px" }}
          >
            <div class="flex items-center" style={{ gap: "6px", "margin-bottom": "4px" }}>
              <span class="flex-1" style={{ "font-size": "var(--fs-meta)", "font-weight": 600, color: "var(--accent-text)" }}>
                {x().state === "done" && x().from ? `${t("Translated from")} ${x().from}` : t("Znga translation")}
              </span>
              <Show when={x().state === "done"}>
                <button
                  type="button"
                  aria-label={t(copied() ? "Copied" : "Copy translation")}
                  title={t(copied() ? "Copied" : "Copy")}
                  onClick={() => {
                    void navigator.clipboard?.writeText(x().text);
                    setCopied(true);
                    setTimeout(() => setCopied(false), 1500);
                  }}
                  style={{ color: "var(--text-secondary)" }}
                >
                  <CopyIcon size={14} />
                </button>
              </Show>
              <button type="button" aria-label={t("Close")} onClick={() => void commands.zngaTranslate(p.chat, p.msg.id, "close")} style={{ color: "var(--text-secondary)" }}>
                <CloseIcon size={15} />
              </button>
            </div>
            <Show when={x().state === "pending"}>
              <div style={{ color: "var(--text-secondary)" }}>{t("Znga is translating…")}</div>
            </Show>
            <Show when={x().state === "done"}>
              <div style={{ "white-space": "pre-wrap", color: "var(--text-primary)" }}>{x().text}</div>
            </Show>
            <Show when={x().state === "failed"}>
              <div style={{ color: "var(--danger)" }}>{x().text}</div>
              <button
                type="button"
                onClick={() => void commands.zngaTranslate(p.chat, p.msg.id, "retry")}
                class="rounded-full"
                style={{ "margin-top": "6px", height: "24px", padding: "0 11px", "font-size": "var(--fs-meta)", "font-weight": 600, background: "var(--round-button-bg)", color: "var(--text-primary)" }}
              >
                {t("Try again")}
              </button>
            </Show>
          </div>
        )}
      </Show>
    </>
  );
}
