// @vitest-environment node
import { describe, expect, it } from "vitest";
import { blocks, inline, plain } from "./format";

function formatsHeadingsListsAndBold() {
  const out = blocks("## Today\n\n- one\n- **9 am** call\n- three\n\nDone.");
  expect(out.map((b) => b.kind)).toEqual(["heading", "list", "para"]);
  const list = out[1];
  if (list.kind !== "list") throw new Error("list");
  expect(list.items).toHaveLength(3);
  expect(list.items[1][0]).toEqual({ kind: "bold", parts: [{ kind: "text", text: "9 am" }] });
  // No mark is left as text.
  expect(JSON.stringify(out)).not.toMatch(/#|\*\*/);
}

function markupStaysText() {
  const out = blocks('<img src=x onerror="alert(1)"> and <script>x</script>');
  expect(out).toEqual([{ kind: "para", inline: [{ kind: "text", text: '<img src=x onerror="alert(1)"> and <script>x</script>' }] }]);
}

function plainAnswersAreUnchanged() {
  expect(blocks("Hello\nthere")).toEqual([{ kind: "para", inline: [{ kind: "text", text: "Hello\nthere" }] }]);
}

function citationsAndChatsBecomeLinks() {
  const parts = inline("Ada Lovelace sent it [1] and Ben too [2, 9]", [{ id: "a", title: "Ada Lovelace" }, { id: "x", title: "Ada" }], new Set([1, 2]));
  expect(parts).toEqual([
    { kind: "chat", id: "a", title: "Ada Lovelace" },
    { kind: "text", text: " sent it" },
    { kind: "cite", n: 1 },
    { kind: "text", text: " and Ben too " },
    { kind: "text", text: "[2, 9]" },
  ]);
  // A title inside another word is not a link.
  expect(inline("Adam", [{ id: "x", title: "Ada" }])).toEqual([{ kind: "text", text: "Adam" }]);
}

function copyDropsMarks() {
  expect(plain("## Emails\n- **Ada**: a@x.co [1]\n- Ben [2, 3]")).toBe("Emails\n• Ada: a@x.co\n• Ben");
}

describe("Znga answer format (REQ-ZANS-002)", () => {
  it("formats headings, lists and bold (REQ-ZANS-002.A1)", formatsHeadingsListsAndBold);
  it("shows markup as text (REQ-ZANS-002.A2)", markupStaysText);
  it("leaves plain answers unchanged (REQ-ZANS-002.A3)", plainAnswersAreUnchanged);
  it("links citations and chats (REQ-ZANS-001.A1, A2, A3)", citationsAndChatsBecomeLinks);
  it("copies without marks (REQ-ZANS-003.A1)", copyDropsMarks);
});
