// What the right side shows with no chat open, as the egui app draws it
// (src/ui/conversation.rs, the empty state): the chat colour, a chat icon
// on a soft plate, the app's name, and where to start.
import { ChatsIcon } from "../components/Icons";
import { t } from "../lib/i18n";

export default function EmptyPane() {
  return (
    <section class="flex h-full min-w-0 flex-1 flex-col items-center" style={{ background: "var(--conversation-bg)" }}>
      <div class="flex flex-col items-center text-center" style={{ "margin-top": "calc(50vh - 126px)" }}>
        <span
          class="flex items-center justify-center"
          style={{ width: "72px", height: "72px", "border-radius": "16px", background: "var(--empty-plate)", color: "var(--text-secondary)" }}
        >
          <ChatsIcon size={40} />
        </span>
        <h2 style={{ margin: "12px 0 0", "font-size": "26px", "line-height": "32px", "font-weight": 700, color: "var(--text-primary)" }}>
          SuperWhatsapp
        </h2>
        <p style={{ margin: "6px 0 0", "font-size": "14.5px", "line-height": "18px", color: "var(--text-secondary)" }}>
          {t("Select a chat on the left.")}
        </p>
        <p style={{ margin: "8px 0 0", "font-size": "12.5px", "line-height": "16px", color: "var(--text-meta)" }}>
          {t("⌘+K to search · ? for keyboard shortcuts")}
        </p>
      </div>
    </section>
  );
}
