// A pop-up menu at a point, and a confirmation dialog, drawn as WhatsApp
// Desktop for macOS draws them (REQ-WEBUI-011).
import { For, type JSX, Show } from "solid-js";
import { t } from "../lib/i18n";

export interface Entry {
  label: string;
  run: () => void;
  danger?: boolean;
  disabled?: boolean;
}

/** A menu at (x, y); it stays inside the window. */
export function Menu(p: { x: number; y: number; entries: (Entry | "-")[]; onClose: () => void }) {
  const width = 210;
  const left = () => Math.min(p.x, window.innerWidth - width - 8);
  const top = () => Math.min(p.y, window.innerHeight - p.entries.length * 27 - 20);
  return (
    <>
      <div
        class="fixed inset-0 z-30"
        onClick={p.onClose}
        onContextMenu={(e) => {
          e.preventDefault();
          p.onClose();
        }}
      />
      <div
        role="menu"
        class="fixed z-40"
        onKeyDown={(e) => e.key === "Escape" && p.onClose()}
        style={{
          left: `${left()}px`,
          top: `${top()}px`,
          width: `${width}px`,
          padding: "5px",
          background: "var(--menu-bg)",
          "border-radius": "10px",
          "box-shadow": "var(--shadow-menu)",
        }}
      >
        <For each={p.entries}>
          {(entry) =>
            entry === "-" ? (
              <div style={{ height: "var(--hairline)", margin: "4px 8px", background: "var(--divider)" }} />
            ) : (
              <button
                type="button"
                role="menuitem"
                disabled={entry.disabled}
                aria-disabled={entry.disabled}
                onClick={() => {
                  if (entry.disabled) return;
                  p.onClose();
                  entry.run();
                }}
                class="menu-item flex w-full items-center text-left"
                style={{
                  height: "26px",
                  padding: "0 10px",
                  "border-radius": "6px",
                  "font-size": "var(--fs-row-text)",
                  color: entry.danger ? "var(--danger)" : "var(--text-primary)",
                  opacity: entry.disabled ? 0.4 : 1,
                }}
              >
                {t(entry.label)}
              </button>
            )
          }
        </For>
      </div>
    </>
  );
}

/** Asks before something that cannot be undone. */
export function Confirm(p: {
  title: string;
  body?: JSX.Element;
  action: string;
  danger?: boolean;
  onConfirm: () => void;
  onClose: () => void;
}) {
  const button = (label: string, primary: boolean, run: () => void) => (
    <button
      type="button"
      onClick={run}
      class="rounded-full"
      style={{
        height: "30px",
        padding: "0 16px",
        "font-size": "var(--fs-row-text)",
        "font-weight": 600,
        background: primary ? (p.danger ? "var(--danger)" : "var(--accent)") : "var(--round-button-bg)",
        color: primary ? (p.danger ? "#fff" : "var(--accent-fg)") : "var(--text-primary)",
      }}
    >
      {t(label)}
    </button>
  );
  return (
    <div
      class="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: "var(--overlay)" }}
      onClick={(e) => e.target === e.currentTarget && p.onClose()}
      onKeyDown={(e) => e.key === "Escape" && p.onClose()}
    >
      <div
        role="alertdialog"
        aria-label={t(p.title)}
        style={{
          width: "340px",
          padding: "20px",
          background: "var(--menu-bg)",
          "border-radius": "14px",
          "box-shadow": "var(--shadow-menu)",
        }}
      >
        <h2 style={{ margin: 0, "font-size": "var(--fs-row-title)", "font-weight": 600, color: "var(--text-primary)" }}>{t(p.title)}</h2>
        <Show when={p.body}>
          <p style={{ margin: "8px 0 0", "font-size": "var(--fs-row-text)", "line-height": "16px", color: "var(--text-secondary)" }}>{typeof p.body === "string" ? t(p.body) : p.body}</p>
        </Show>
        <div class="flex justify-end" style={{ gap: "8px", "margin-top": "18px" }}>
          {button("Cancel", false, p.onClose)}
          {button(p.action, true, () => {
            p.onClose();
            p.onConfirm();
          })}
        </div>
      </div>
    </div>
  );
}
