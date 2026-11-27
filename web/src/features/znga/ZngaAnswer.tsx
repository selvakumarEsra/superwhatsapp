// One of Znga's answers (REQ-ZANS-001..003): formatted for reading, with
// the messages it cites as links to the exact message, chats it names as
// links to the chat, and Copy, Reply and Forward on hover.
import { createSignal, For, Match, Show, Switch, type JSX } from "solid-js";
import { commands, type ZEntry } from "../../lib/bridge";
import { t } from "../../lib/i18n";
import { CopyIcon, ForwardIcon, ReplyIcon } from "../../components/Icons";
import { blocks, plain, type Inline } from "./format";
import ZngaCard from "./ZngaCard";

function HoverButton(p: { label: string; onClick: () => void; children: JSX.Element }) {
  return (
    <button
      type="button"
      title={t(p.label)}
      aria-label={t(p.label)}
      onClick={p.onClick}
      class="flex items-center justify-center rounded-full"
      style={{ width: "26px", height: "26px", background: "var(--round-button-bg)", color: "var(--text-primary)", "box-shadow": "var(--shadow-bubble)" }}
    >
      {p.children}
    </button>
  );
}

export default function ZngaAnswer(p: {
  entry: ZEntry;
  onReply: (entry: ZEntry) => void;
  onForward: (entry: ZEntry) => void;
}) {
  const [copied, setCopied] = createSignal(false);
  // The cited messages, by number, and where their cards are.
  const cites = () => {
    const found = new Map<number, number>();
    p.entry.cards.forEach((card, index) => {
      if (card.kind === "cite" && card.n !== null) found.set(card.n, index);
    });
    return found;
  };
  const open = (n: number) => {
    const index = cites().get(n);
    if (index !== undefined) void commands.zngaCard(p.entry.id, index, "open");
  };
  const parsed = () => blocks(p.entry.text, p.entry.chats, new Set(cites().keys()));
  const copy = () => {
    void navigator.clipboard?.writeText(plain(p.entry.text));
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const Parts = (q: { parts: Inline[] }): JSX.Element => (
    <For each={q.parts}>
      {(part) => (
        <Switch>
          <Match when={part.kind === "text" && part}>{(x) => <>{x().text}</>}</Match>
          <Match when={part.kind === "bold" && part}>{(x) => <strong style={{ "font-weight": 600 }}><Parts parts={x().parts} /></strong>}</Match>
          <Match when={part.kind === "cite" && part}>
            {(x) => (
              <button
                type="button"
                onClick={() => open(x().n)}
                aria-label={`${t("Open message")} ${x().n}`}
                title={p.entry.cards[cites().get(x().n)!]?.title}
                class="znga-cite"
                style={{
                  "margin-left": "3px",
                  padding: "0 5px",
                  "min-width": "18px",
                  height: "16px",
                  "border-radius": "8px",
                  "font-size": "11px",
                  "font-weight": 600,
                  "vertical-align": "1px",
                  background: "var(--accent)",
                  color: "var(--accent-fg)",
                }}
              >
                {x().n}
              </button>
            )}
          </Match>
          <Match when={part.kind === "chat" && part}>
            {(x) => (
              <button type="button" onClick={() => void commands.openChat(x().id)} style={{ color: "var(--accent-text)", "font-weight": 500 }}>
                {x().title}
              </button>
            )}
          </Match>
          <Match when={part.kind === "url" && part}>
            {(x) => (
              <button type="button" onClick={() => void commands.openUrl(x().url)} style={{ color: "var(--accent-text)", "text-decoration": "underline", "word-break": "break-all" }}>
                {x().url}
              </button>
            )}
          </Match>
        </Switch>
      )}
    </For>
  );

  const citeCards = () => p.entry.cards.map((card, index) => ({ card, index })).filter((c) => c.card.kind === "cite");
  const otherCards = () => p.entry.cards.map((card, index) => ({ card, index })).filter((c) => c.card.kind !== "cite");

  return (
    <div class="znga-answer group relative">
      <div class="znga-answer-actions" role="toolbar" aria-label={t("Answer actions")}>
        <HoverButton label={copied() ? "Copied" : "Copy"} onClick={copy}><CopyIcon size={14} /></HoverButton>
        <HoverButton label="Reply" onClick={() => p.onReply(p.entry)}><ReplyIcon size={14} /></HoverButton>
        <HoverButton label="Forward" onClick={() => p.onForward(p.entry)}><ForwardIcon size={14} /></HoverButton>
      </div>
      <div class="znga-prose">
        <For each={parsed()}>
          {(block) => (
            <Switch>
              <Match when={block.kind === "heading" && block}>
                {(b) => (
                  <div role="heading" aria-level={Math.min(6, b().level + 1)} style={{ "font-weight": 700, "font-size": b().level <= 2 ? "15px" : "14px", margin: "8px 0 3px" }}>
                    <Parts parts={b().inline} />
                  </div>
                )}
              </Match>
              <Match when={block.kind === "list" && block}>
                {(b) => {
                  const items = () => b().items;
                  const body = () => (
                    <For each={items()}>{(item) => <li style={{ margin: "2px 0" }}><Parts parts={item} /></li>}</For>
                  );
                  return b().ordered ? (
                    <ol style={{ margin: "4px 0", "padding-left": "20px", "list-style": "decimal" }}>{body()}</ol>
                  ) : (
                    <ul style={{ margin: "4px 0", "padding-left": "18px", "list-style": "disc" }}>{body()}</ul>
                  );
                }}
              </Match>
              <Match when={block.kind === "para" && block}>
                {(b) => <p style={{ margin: "4px 0", "white-space": "pre-wrap" }}><Parts parts={b().inline} /></p>}
              </Match>
            </Switch>
          )}
        </For>
      </div>
      <Show when={citeCards().length > 0}>
        <div style={{ "margin-top": "8px", "border-top": "var(--hairline) solid var(--divider)", "padding-top": "6px" }}>
          <div style={{ "font-size": "var(--fs-meta)", "font-weight": 600, color: "var(--text-secondary)", "margin-bottom": "2px" }}>{t("Messages")}</div>
          <For each={citeCards()}>
            {({ card, index }) => (
              <button
                type="button"
                onClick={() => void commands.zngaCard(p.entry.id, index, "open")}
                aria-label={`${t("Open message")} ${card.n}: ${card.title}`}
                class="menu-item flex w-full items-start text-left"
                style={{ gap: "8px", padding: "5px 6px", "border-radius": "7px" }}
              >
                <span style={{ "min-width": "18px", height: "16px", "border-radius": "8px", "font-size": "11px", "font-weight": 600, background: "var(--accent)", color: "var(--accent-fg)", "text-align": "center", "line-height": "16px", "margin-top": "1px" }}>
                  {card.n}
                </span>
                <span class="min-w-0 flex-1">
                  <span class="flex" style={{ gap: "6px" }}>
                    <span class="min-w-0 flex-1 truncate" style={{ "font-size": "13px", "font-weight": 600, color: "var(--text-primary)" }}>{card.title}</span>
                    <span style={{ "font-size": "11.5px", color: "var(--text-meta)", "white-space": "nowrap" }}>{card.meta}</span>
                  </span>
                  <span class="block truncate" style={{ "font-size": "13px", color: "var(--text-secondary)" }}>{card.body}</span>
                </span>
              </button>
            )}
          </For>
        </div>
      </Show>
      <For each={otherCards()}>{({ card, index }) => <ZngaCard card={card} entry={p.entry.id} index={index} />}</For>
    </div>
  );
}
