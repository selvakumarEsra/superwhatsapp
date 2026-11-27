// The Calls tab's main pane with no chat open, as egui draws it
// (src/ui/calls.rs:start): on the panel colour, a 72-point accent circle
// with the phone-plus icon at half the height less 110, "Start call"
// semibold 15 under it, and the explanation regular 13.5 on one line
// (egui sets 360 but its centred layout lets the line run ~440).
import { PhonePlusIcon } from "../../components/Icons";
import { commands } from "../../lib/bridge";
import { t } from "../../lib/i18n";

export default function StartCall() {
  return (
    <section class="flex h-full min-w-0 flex-1 flex-col items-center text-center" style={{ background: "var(--panel-bg)" }}>
      <div class="flex flex-col items-center" style={{ "padding-top": "max(24px, calc(50vh - 110px))" }}>
        <button
          type="button"
          aria-label={t("Start call")}
          onClick={() => commands.showNewCall(true)}
          class="call-control flex items-center justify-center rounded-full"
          style={{ width: "72px", height: "72px", background: "var(--accent)", color: "var(--on-accent, #fff)" }}
        >
          <PhonePlusIcon size={30} />
        </button>
        <span style={{ "margin-top": "10px", "font-size": "15px", "font-weight": 600, color: "var(--text-primary)" }}>{t("Start call")}</span>
        <p style={{ margin: "18px 0 0", "max-width": "480px", "font-size": "13.5px", "line-height": "18px", color: "var(--text-secondary)" }}>
          {t("Call one person by voice or video, or choose a call in the list to see it.")}
        </p>
      </div>
    </section>
  );
}
