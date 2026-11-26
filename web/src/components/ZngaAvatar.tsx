// Znga's picture: SuperWhatsapp's own sparkle on green, never WhatsApp
// artwork (specs/znga-chat.md).
import { SparkleIcon } from "./Icons";

export default function ZngaAvatar(p: { size: number }) {
  return (
    <div
      class="flex shrink-0 items-center justify-center rounded-full"
      aria-hidden="true"
      style={{ width: `${p.size}px`, height: `${p.size}px`, background: "var(--accent)", color: "var(--accent-fg)" }}
    >
      <SparkleIcon size={Math.round(p.size * 0.55)} />
    </div>
  );
}
