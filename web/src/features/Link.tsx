// Linking a phone by QR code or pairing code (REQ-WEBUI-005).
import { Show } from "solid-js";
import { encode } from "uqr";
import type { Link as LinkState } from "../lib/bridge";

function Qr(p: { text: string }) {
  const code = () => encode(p.text, { border: 0 });
  return (
    <svg
      width="264"
      height="264"
      viewBox={`0 0 ${code().size} ${code().size}`}
      shape-rendering="crispEdges"
      role="img"
      aria-label="QR code to link your phone"
    >
      <rect width="100%" height="100%" fill="#ffffff" />
      <path
        fill="#122e31"
        d={code()
          .data.flatMap((row, y) => row.map((on, x) => (on ? `M${x} ${y}h1v1h-1z` : "")))
          .join("")}
      />
    </svg>
  );
}

export default function Link(p: { link: LinkState }) {
  return (
    <div class="flex h-full w-full items-center justify-center" style={{ background: "var(--app-bg)" }}>
      <div
        class="flex w-[1000px] max-w-[calc(100%-64px)] items-start justify-between gap-[48px] rounded-[3px] p-[64px]"
        style={{ background: "var(--panel-bg)", "box-shadow": "0 17px 50px rgba(11,20,26,.19), 0 12px 15px rgba(11,20,26,.24)" }}
      >
        <div>
          <h1 style={{ "font-size": "28px", "font-weight": 300, color: "var(--text-primary)", margin: 0 }}>Use SuperWhatsapp on your computer</h1>
          <ol class="mt-[40px] list-decimal pl-[24px]" style={{ "font-size": "18px", "line-height": "28px", color: "var(--text-primary)" }}>
            <li>Open WhatsApp on your phone</li>
            <li>Tap Menu or Settings and select Linked devices</li>
            <li>Tap Link a device</li>
            <li>Point your phone at this screen to scan the code</li>
          </ol>
        </div>
        <div class="flex h-[264px] w-[264px] shrink-0 items-center justify-center">
          <Show
            when={p.link.state === "unlinked" && p.link.qr}
            fallback={
              <span style={{ color: "var(--text-secondary)" }}>
                {p.link.state === "failed" ? (p.link as { reason: string }).reason : "Loading…"}
              </span>
            }
          >
            <Qr text={(p.link as { qr: string }).qr} />
          </Show>
        </div>
      </div>
    </div>
  );
}
