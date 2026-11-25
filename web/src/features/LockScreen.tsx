// The app lock's screen (REQ-WEBUI-005): a password field, how long to wait
// after several wrong tries, and "Forgot password?", which can only unlink.
// The page gets no chats or messages while it shows.
import { createSignal, Show } from "solid-js";
import { commands, type LockView } from "../lib/bridge";
import { LockIcon } from "../components/Icons";

export default function LockScreen(p: { lock: LockView }) {
  const [password, setPassword] = createSignal("");
  const submit = (e: Event) => {
    e.preventDefault();
    if (!password() || p.lock.wait || p.lock.checking) return;
    void commands.unlock(password());
    setPassword("");
  };
  const button = (label: string, run: () => void, primary = false, danger = false) => (
    <button
      type="button"
      onClick={run}
      class="rounded-full"
      style={{
        height: "30px",
        padding: "0 16px",
        "font-size": "var(--fs-row-text)",
        "font-weight": 600,
        background: primary ? (danger ? "var(--danger)" : "var(--accent)") : "var(--round-button-bg)",
        color: primary ? (danger ? "#fff" : "var(--accent-fg)") : "var(--text-primary)",
      }}
    >
      {label}
    </button>
  );
  return (
    <main
      data-tauri-drag-region
      class="flex h-full w-full flex-col items-center justify-center"
      style={{ background: "var(--panel-bg)", color: "var(--text-primary)" }}
    >
      <span style={{ color: "var(--text-secondary)" }}><LockIcon size={36} /></span>
      <h1 style={{ margin: "14px 0 0", "font-size": "17px", "font-weight": 600 }}>ZapFast is locked</h1>
      <Show
        when={p.lock.forgetting === "no"}
        fallback={
          <div class="flex flex-col items-center text-center" style={{ "max-width": "320px", "margin-top": "12px" }}>
            <Show
              when={p.lock.forgetting === "confirming"}
              fallback={<p style={{ "font-size": "var(--fs-row-text)", color: "var(--text-secondary)" }}>Unlinking this computer…</p>}
            >
              <h2 style={{ margin: 0, "font-size": "var(--fs-row-title)", "font-weight": 600 }}>Unlink this computer?</h2>
              <p style={{ "font-size": "var(--fs-row-text)", "line-height": "16px", color: "var(--text-secondary)" }}>
                Without the password, the only way back in is to unlink. This removes the device from WhatsApp, deletes the chats stored here, and turns the app lock off. You can link again with a new code.
              </p>
              <div class="flex" style={{ gap: "8px", "margin-top": "14px" }}>
                {button("Cancel", () => commands.forgotPassword(false))}
                {button("Unlink", () => commands.forgotPassword(true, true), true, true)}
              </div>
            </Show>
          </div>
        }
      >
        <form onSubmit={submit} class="flex flex-col items-center" style={{ "margin-top": "16px", gap: "8px" }}>
          <input
            type="password"
            autofocus
            aria-label="Password"
            placeholder="Password"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
            disabled={!!p.lock.wait}
            class="selectable outline-none"
            style={{ width: "240px", height: "30px", padding: "0 12px", "border-radius": "15px", background: "var(--search-bg)", color: "var(--text-primary)", "font-size": "var(--fs-row-text)" }}
          />
          <button
            type="submit"
            disabled={!password() || !!p.lock.wait || p.lock.checking}
            class="rounded-full"
            style={{ height: "30px", padding: "0 20px", "font-size": "var(--fs-row-text)", "font-weight": 600, background: "var(--accent)", color: "var(--accent-fg)", opacity: password() ? 1 : 0.5 }}
          >
            Unlock
          </button>
          <p style={{ margin: 0, height: "16px", "font-size": "var(--fs-meta)", color: p.lock.wrong ? "var(--danger)" : "var(--text-secondary)" }}>
            {p.lock.wait ? `Too many wrong passwords. Try again in ${p.lock.wait} s.` : p.lock.wrong ? "Wrong password. Try again." : "Enter your password to unlock."}
          </p>
          <button type="button" onClick={() => commands.forgotPassword(true)} style={{ "font-size": "var(--fs-meta)", color: "var(--accent-text)" }}>
            Forgot password? Unlink this computer
          </button>
        </form>
      </Show>
    </main>
  );
}
