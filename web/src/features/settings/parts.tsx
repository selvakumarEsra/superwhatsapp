// The Settings page's building blocks, as egui draws them
// (src/ui/widgets.rs:setting_row, widgets::switch, the combo box and
// theme::soft_button), shared by every section including Znga's.
import { type JSX, Show } from "solid-js";
import { t } from "../../lib/i18n";

export function Row(p: { title: string; description?: string; top?: boolean; children?: JSX.Element }) {
  // widgets::setting_row: the words take all but 260 points, the control
  // sits on the right, and each row ends with a 10-point gap.
  return (
    <div class="flex justify-between" style={{ "align-items": p.top ? "flex-start" : "center", "padding-bottom": "18px", gap: "12px" }}>
      <div class="min-w-0 shrink-0" style={{ width: "max(120px, calc(100% - 260px))" }}>
        <div style={{ "font-size": "14px", "line-height": "18px", "font-weight": 500, color: "var(--text-primary)" }}>{t(p.title)}</div>
        <Show when={p.description}>
          <div style={{ "margin-top": "2px", "font-size": "12.5px", "line-height": "15px", color: "var(--text-secondary)" }}>{t(p.description!)}</div>
        </Show>
      </div>
      <div class="flex shrink-0 flex-col items-end" style={{ gap: "6px" }}>{p.children}</div>
    </div>
  );
}

/** widgets::switch: 40×22, a small grey knob off, a white one with a check on. */
export function Switch(p: { label: string; on: boolean; onChange: (on: boolean) => void; disabled?: boolean }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={p.on}
      aria-label={t(p.label)}
      disabled={p.disabled}
      onClick={() => p.onChange(!p.on)}
      class="relative inline-block shrink-0 rounded-full"
      style={{
        width: "40px",
        height: "22px",
        background: p.on ? "var(--accent)" : "var(--switch-off)",
        "box-shadow": p.on ? "none" : "inset 0 0 0 1.5px var(--text-secondary)",
        opacity: p.disabled ? 0.45 : 1,
        transition: "background .15s",
      }}
    >
      <span
        class="absolute flex items-center justify-center rounded-full"
        style={{
          top: "50%",
          left: p.on ? "29px" : "11px",
          width: p.on ? "16px" : "12px",
          height: p.on ? "16px" : "12px",
          transform: "translate(-50%, -50%)",
          background: p.on ? "#fff" : "var(--text-secondary)",
          color: "var(--accent)",
          transition: "left .15s, width .15s, height .15s",
        }}
      >
        <Show when={p.on}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
        </Show>
      </span>
    </button>
  );
}

/** egui's combo box: 200 wide, the choice on the left, ▼ on the right. */
export const combo = {
  height: "28px",
  padding: "0 28px 0 8px",
  "border-radius": "6px",
  background: "var(--search-bg)",
  color: "var(--text-primary)",
  "font-size": "14px",
  border: "none",
  appearance: "none",
  "-webkit-appearance": "none",
} as const;

export function Caret() {
  return (
    <svg class="pointer-events-none absolute" style={{ right: "12px" }} width="10" height="7" viewBox="0 0 10 7" aria-hidden="true">
      <path d="M0 0h10L5 7z" fill="var(--text-primary)" />
    </svg>
  );
}

/** egui's text field: a bordered 28-point box. */
export const field = {
  height: "28px",
  padding: "0 8px",
  "border-radius": "6px",
  background: "var(--input-bg)",
  border: "1px solid var(--panel-border)",
  color: "var(--text-primary)",
  "font-size": "14px",
} as const;

export function Button(p: { label: string; danger?: boolean; icon?: JSX.Element; onClick: () => void; disabled?: boolean }) {
  return (
    <button
      type="button"
      onClick={p.onClick}
      disabled={p.disabled}
      class="rounded-full"
      style={{
        height: "28px",
        padding: "0 12px",
        "font-size": "14px",
        "font-weight": 500,
        background: "var(--round-button-bg)",
        color: p.danger ? "var(--danger)" : "var(--text-primary)",
        opacity: p.disabled ? 0.45 : 1,
      }}
    >
      <span class="inline-flex items-center" style={{ gap: "6px" }}>
        {p.icon}
        {t(p.label)}
      </span>
    </button>
  );
}
