// What the right side shows with no chat open.
import { LockIcon } from "../components/Icons";

export default function EmptyPane() {
  return (
    <section
      class="flex h-full min-w-0 flex-1 flex-col items-center justify-center"
      style={{ background: "var(--empty-bg)", "border-left": "1px solid var(--divider)", "border-bottom": "6px solid var(--accent)" }}
    >
      <div class="flex max-w-[560px] flex-col items-center px-[32px] text-center">
        <svg width="250" height="160" viewBox="0 0 250 160" aria-hidden="true">
          <rect x="40" y="20" width="170" height="110" rx="10" fill="none" stroke="var(--text-secondary)" stroke-width="3" opacity=".35" />
          <rect x="95" y="130" width="60" height="8" rx="2" fill="var(--text-secondary)" opacity=".35" />
          <path d="M100 60h50M100 75h35" stroke="var(--accent)" stroke-width="5" stroke-linecap="round" />
          <circle cx="78" cy="68" r="14" fill="var(--accent)" opacity=".8" />
        </svg>
        <h2 class="mt-[28px]" style={{ "font-size": "32px", "font-weight": 300, color: "var(--text-primary)", margin: "28px 0 0" }}>
          ZapFast
        </h2>
        <p class="mt-[18px]" style={{ "font-size": "14px", "line-height": "20px", color: "var(--text-secondary)" }}>
          Send and receive messages from your computer, linked to your phone.
          <br />
          Pick a chat on the left to start.
        </p>
      </div>
      <p class="absolute bottom-[40px] flex items-center gap-[6px]" style={{ "font-size": "14px", color: "var(--text-secondary)" }}>
        <LockIcon size={12} /> Your messages stay on this computer, encrypted.
      </p>
    </section>
  );
}
