// Znga's light answer formatting (REQ-ZANS-002): `##` headings, `-` or
// `1.` lists, `**bold**`, blank lines between paragraphs, `[n]` citations
// (REQ-ZANS-001) and chat titles as links. The result is data that the
// answer renders as Solid nodes; nothing here is ever HTML, so markup in
// an answer can only show as text.

export interface ChatLink { id: string; title: string }

export type Inline =
  | { kind: "text"; text: string }
  | { kind: "bold"; parts: Inline[] }
  | { kind: "cite"; n: number }
  | { kind: "chat"; id: string; title: string }
  | { kind: "url"; url: string };

export type Block =
  | { kind: "heading"; level: number; inline: Inline[] }
  | { kind: "list"; ordered: boolean; items: Inline[][] }
  | { kind: "para"; inline: Inline[] };

const TOKEN = /\*\*(.+?)\*\*|\[(\d+(?:\s*,\s*\d+)*)\]|(https?:\/\/[^\s<>()"']+[^\s<>()"'.,;:!?])/g;

const isWord = (c: string | undefined) => !!c && /[\p{L}\p{N}]/u.test(c);

/** Plain text with chat titles that appear as whole words turned into links. */
function withChats(text: string, chats: ChatLink[]): Inline[] {
  const sorted = [...chats].filter((c) => c.title.length >= 3).sort((a, b) => b.title.length - a.title.length);
  const out: Inline[] = [];
  let rest = text;
  while (rest) {
    let best: { at: number; chat: ChatLink } | null = null;
    for (const chat of sorted) {
      let from = 0;
      for (;;) {
        const at = rest.indexOf(chat.title, from);
        if (at < 0) break;
        if (!isWord(rest[at - 1]) && !isWord(rest[at + chat.title.length])) {
          if (!best || at < best.at) best = { at, chat };
          break;
        }
        from = at + 1;
      }
    }
    if (!best) {
      out.push({ kind: "text", text: rest });
      break;
    }
    if (best.at > 0) out.push({ kind: "text", text: rest.slice(0, best.at) });
    out.push({ kind: "chat", id: best.chat.id, title: best.chat.title });
    rest = rest.slice(best.at + best.chat.title.length);
  }
  return out;
}

/** One line's inline parts. `cites` are the numbers that have a message. */
export function inline(text: string, chats: ChatLink[] = [], cites: ReadonlySet<number> = new Set()): Inline[] {
  const out: Inline[] = [];
  let last = 0;
  for (const match of text.matchAll(TOKEN)) {
    const at = match.index ?? 0;
    if (at > last) out.push(...withChats(text.slice(last, at), chats));
    if (match[1] !== undefined) {
      out.push({ kind: "bold", parts: inline(match[1], chats, cites) });
    } else if (match[2] !== undefined) {
      const numbers = match[2].split(",").map((n) => Number(n.trim()));
      if (numbers.every((n) => cites.has(n))) {
        // A space before a citation is dropped: the chip sits on the word.
        const prev = out.at(-1);
        if (prev?.kind === "text") prev.text = prev.text.replace(/\s+$/, "");
        for (const n of numbers) out.push({ kind: "cite", n });
      } else {
        out.push({ kind: "text", text: match[0] });
      }
    } else if (match[3] !== undefined) {
      out.push({ kind: "url", url: match[3] });
    }
    last = at + match[0].length;
  }
  if (last < text.length) out.push(...withChats(text.slice(last), chats));
  return out.filter((part) => part.kind !== "text" || part.text !== "");
}

/** The answer as headings, lists and paragraphs. */
export function blocks(text: string, chats: ChatLink[] = [], cites: ReadonlySet<number> = new Set()): Block[] {
  const out: Block[] = [];
  let para: string[] = [];
  const flush = () => {
    if (para.length) out.push({ kind: "para", inline: inline(para.join("\n"), chats, cites) });
    para = [];
  };
  for (const line of text.replace(/\r\n/g, "\n").split("\n")) {
    const heading = /^\s{0,3}(#{1,6})\s+(.*)$/.exec(line);
    const bullet = /^\s*[-*•]\s+(.*)$/.exec(line);
    const numbered = /^\s*\d+[.)]\s+(.*)$/.exec(line);
    if (heading) {
      flush();
      out.push({ kind: "heading", level: heading[1].length, inline: inline(heading[2].replace(/\s*#+\s*$/, ""), chats, cites) });
    } else if (bullet || numbered) {
      flush();
      const ordered = !bullet;
      const item = inline((bullet ?? numbered)![1], chats, cites);
      const prev = out.at(-1);
      if (prev?.kind === "list" && prev.ordered === ordered) prev.items.push(item);
      else out.push({ kind: "list", ordered, items: [item] });
    } else if (!line.trim()) {
      flush();
    } else {
      para.push(line);
    }
  }
  flush();
  return out;
}

/** The answer's words without marks or citations, for Copy (REQ-ZANS-003.A1). */
export function plain(text: string): string {
  return text
    .replace(/\r\n/g, "\n")
    .split("\n")
    .map((line) => {
      const heading = /^\s{0,3}#{1,6}\s+(.*)$/.exec(line);
      const bullet = /^\s*[-*]\s+(.*)$/.exec(line);
      const out = heading ? heading[1] : bullet ? `• ${bullet[1]}` : line;
      return out
        .replace(/\*\*|__/g, "")
        .replace(/\s*\[\d+(?:\s*,\s*\d+)*\]/g, "");
    })
    .join("\n")
    .trim();
}
