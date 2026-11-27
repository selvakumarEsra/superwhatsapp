// The code that opens the locked chats, as egui asks for it
// (src/ui/dialogs.rs:unlock_locked_chats): choose and confirm a local code
// the first time, then enter it. The code goes to the core and is never kept
// here.
import { createSignal, Show } from "solid-js";
import { commands } from "../../lib/bridge";
import { t } from "../../lib/i18n";

export default function LockedPrompt(p: { setup: boolean; error: boolean }) {
  const [code, setCode] = createSignal("");
  const [confirm, setConfirm] = createSignal("");
  const ready = () => code().trim() !== "" && (!p.setup || code().trim() === confirm().trim());
  const submit = () => {
    if (!ready()) return;
    void commands.unlockLocked(code());
    setCode("");
    setConfirm("");
  };
  const field = (value: () => string, set: (v: string) => void, label: string, focus: boolean) => (
    <input
      type="password"
      autocomplete="off"
      placeholder={t(label)}
      aria-label={t(label)}
      value={value()}
      ref={(el) => focus && queueMicrotask(() => el.focus())}
      onInput={(e) => set(e.currentTarget.value)}
      onKeyDown={(e) => e.key === "Enter" && submit()}
      class="selectable w-full outline-none"
      style={{
        height: "36px",
        "margin-top": "10px",
        padding: "0 12px",
        "border-radius": "8px",
        border: "1px solid var(--input-border)",
        background: "var(--search-bg)",
        color: "var(--text-primary)",
        "font-size": "var(--fs-input)",
      }}
    />
  );
  const button = (label: string, primary: boolean, run: () => void, enabled = true) => (
    <button
      type="button"
      disabled={!enabled}
      onClick={run}
      class="rounded-full"
      style={{
        height: "30px",
        padding: "0 16px",
        "font-size": "var(--fs-row-text)",
        "font-weight": 600,
        opacity: enabled ? 1 : 0.45,
        background: primary ? "var(--accent)" : "var(--round-button-bg)",
        color: primary ? "var(--accent-fg)" : "var(--text-primary)",
      }}
    >
      {t(label)}
    </button>
  );
  return (
    <div
      class="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: "var(--overlay)" }}
      onClick={(e) => e.target === e.currentTarget && commands.closeDialog()}
      onKeyDown={(e) => e.key === "Escape" && commands.closeDialog()}
    >
      <div
        role="dialog"
        aria-label={t(p.setup ? "Set up locked chats" : "Locked chats")}
        style={{ width: "380px", padding: "20px", background: "var(--menu-bg)", "border-radius": "14px", "box-shadow": "var(--shadow-menu)" }}
      >
        <h2 style={{ margin: 0, "font-size": "var(--fs-row-title)", "font-weight": 600, color: "var(--text-primary)" }}>
          {t(p.setup ? "Set up locked chats" : "Locked chats")}
        </h2>
        <p style={{ margin: "8px 0 4px", "font-size": "var(--fs-row-text)", "line-height": "17px", color: "var(--text-secondary)" }}>
          {p.setup
            ? t("Choose a local code to open the Locked tab. It is separate from your phone's code. This hides chats in SuperWhatsapp, it does not add another encryption layer.")
            : t("Enter your local SuperWhatsapp code. Leaving the Locked tab or closing the window locks it again.")}
        </p>
        {field(code, setCode, "Local code", true)}
        <Show when={p.setup}>{field(confirm, setConfirm, "Confirm code", false)}</Show>
        <Show when={p.error}>
          <p role="alert" style={{ margin: "8px 0 0", "font-size": "var(--fs-row-text)", color: "var(--danger)" }}>
            {t("That code did not match. Try again.")}
          </p>
        </Show>
        <div class="flex justify-end" style={{ gap: "8px", "margin-top": "18px" }}>
          {button("Cancel", false, () => commands.closeDialog())}
          {button(p.setup ? "Set code and open" : "Open locked chats", true, submit, ready())}
        </div>
      </div>
    </div>
  );
}
