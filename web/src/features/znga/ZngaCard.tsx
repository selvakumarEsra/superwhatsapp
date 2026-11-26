// Something Znga found, with its buttons (REQ-ZCHAT-003). Once acted on,
// the card shows what happened and its buttons are disabled.
import { For, Show } from "solid-js";
import { commands, type ZCard } from "../../lib/bridge";
import { t } from "../../lib/i18n";

const LABELS: Record<ZCard["kind"], string> = {
  needsYou: "Needs you",
  briefing: "Briefing",
  draft: "Draft",
  catchUp: "Catch-up",
  plan: "Plan",
  voice: "Voice note",
  promise: "Promise",
  drift: "Stay close",
};

export default function ZngaCard(p: { card: ZCard; entry: number; index: number }) {
  const settled = () => p.card.outcome !== null;
  const press = (action: string) => {
    if (settled()) return;
    // Copy happens here: the page owns the clipboard.
    if (action === "copy") void navigator.clipboard?.writeText(p.card.body);
    void commands.zngaCard(p.entry, p.index, action);
  };
  return (
    <div
      class="znga-card"
      style={{
        "margin-top": "6px",
        padding: "8px 10px",
        "border-radius": "9px",
        background: "var(--znga-card-bg, var(--search-bg))",
        "border-left": `3px solid ${p.card.kind === "needsYou" ? "var(--unread)" : "var(--accent)"}`,
        opacity: settled() ? 0.75 : 1,
      }}
    >
      <div style={{ "font-size": "var(--fs-meta)", "font-weight": 600, color: "var(--accent-text)", "text-transform": "uppercase", "letter-spacing": "0.02em" }}>
        {t(LABELS[p.card.kind])}
      </div>
      <Show when={p.card.title}>
        <div style={{ "margin-top": "2px", "font-weight": 600, color: "var(--text-primary)" }}>{p.card.title}</div>
      </Show>
      <Show when={p.card.body}>
        <div class="selectable" style={{ "margin-top": "2px", "white-space": "pre-wrap", color: "var(--text-primary)" }}>{p.card.body}</div>
      </Show>
      <Show when={p.card.meta}>
        <div style={{ "margin-top": "2px", "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>{p.card.meta}</div>
      </Show>
      <div class="flex flex-wrap items-center" style={{ gap: "6px", "margin-top": "8px" }}>
        <Show
          when={!settled()}
          fallback={<span style={{ "font-size": "var(--fs-meta)", "font-weight": 600, color: "var(--text-secondary)" }}>{t(p.card.outcome!)}</span>}
        >
          <For each={p.card.actions}>
            {(action, i) => (
              <button
                type="button"
                onClick={() => press(action.id)}
                class="rounded-full"
                style={{
                  height: "24px",
                  padding: "0 11px",
                  "font-size": "var(--fs-meta)",
                  "font-weight": 600,
                  background: i() === 0 ? "var(--accent)" : "var(--round-button-bg)",
                  color: i() === 0 ? "var(--accent-fg)" : "var(--text-primary)",
                }}
              >
                {t(action.label)}
              </button>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
