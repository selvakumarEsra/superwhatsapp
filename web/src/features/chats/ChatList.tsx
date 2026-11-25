// The chat list column (REQ-WEBUI-006), as WhatsApp Desktop for macOS
// draws it: a round menu button, the centred title and a green new-chat
// button, then a pill search field, the Archived row and the chats. The
// full list is grouped under Favourites and Chats headings (REQ-WEBUI-006.A4).
import { createMemo, createSignal, For, Show } from "solid-js";
import { commands, state, type ChatRow as Row } from "../../lib/bridge";
import { ArchiveIcon, ChevronDownIcon, ChevronRightIcon, DotsIcon, PlusIcon, SearchIcon } from "../../components/Icons";
import ChatRow from "./ChatRow";
import StarredList from "../starred/StarredList";
import CallsList from "../calls/CallsList";

const TITLES: Record<string, string> = {
  chats: "Chats",
  favourites: "Favourites",
  archived: "Archived",
  starred: "Starred",
  calls: "Calls",
};

const FILTERS: [string, string][] = [
  ["all", "All chats"],
  ["unread", "Unread"],
  ["favorites", "Favourites"],
  ["groups", "Groups"],
];

const roundButton = {
  width: "var(--round-button)",
  height: "var(--round-button)",
};

/** Which headings are folded, kept on this computer. */
const FOLDS = "zapfast.chatList.folded";
function readFolds(): string[] {
  try {
    const value = JSON.parse(localStorage.getItem(FOLDS) ?? "[]");
    return Array.isArray(value) ? value : [];
  } catch {
    return [];
  }
}
function writeFolds(folds: string[]) {
  try {
    localStorage.setItem(FOLDS, JSON.stringify(folds));
  } catch {
    // Folds are a convenience; without storage they last until quit.
  }
}

/** A collapsible heading over a group of chats. */
function GroupHeading(p: { label: string; open: boolean; onToggle: () => void }) {
  return (
    <button
      type="button"
      aria-expanded={p.open}
      onClick={p.onToggle}
      class="flex w-full items-center"
      style={{
        height: "30px",
        "margin-top": "4px",
        "padding-left": "calc(var(--row-inset) + 2px)",
        gap: "5px",
        "font-size": "var(--fs-archived)",
        color: "var(--text-secondary)",
      }}
    >
      <Show when={p.open} fallback={<ChevronRightIcon size={12} />}>
        <ChevronDownIcon size={12} />
      </Show>
      {p.label}
    </button>
  );
}

/** The menu behind the round button: the list filters. */
function FilterMenu(p: { onClose: () => void }) {
  return (
    <>
      <div class="fixed inset-0 z-10" onClick={p.onClose} />
      <div
        role="menu"
        class="absolute z-20 overflow-hidden"
        style={{
          top: "40px",
          left: "8px",
          "min-width": "180px",
          padding: "5px",
          background: "var(--menu-bg)",
          "border-radius": "10px",
          "box-shadow": "var(--shadow-menu)",
        }}
      >
        <For each={FILTERS}>
          {([id, label]) => (
            <button
              type="button"
              role="menuitemradio"
              aria-checked={state.filter === id}
              onClick={() => {
                commands.setFilter(id);
                p.onClose();
              }}
              class="menu-item flex w-full items-center justify-between text-left"
              style={{
                height: "26px",
                padding: "0 10px",
                "border-radius": "6px",
                "font-size": "var(--fs-row-text)",
                color: "var(--text-primary)",
              }}
            >
              {label}
              <Show when={state.filter === id}>
                <span style={{ color: "var(--accent-text)" }}>✓</span>
              </Show>
            </button>
          )}
        </For>
      </div>
    </>
  );
}

export default function ChatList() {
  const [menu, setMenu] = createSignal(false);
  const [folded, setFolded] = createSignal(readFolds());
  const toggle = (group: string) => {
    const next = folded().includes(group) ? folded().filter((g) => g !== group) : [...folded(), group];
    setFolded(next);
    writeFolds(next);
  };
  const grouped = () => state.section === "chats" && state.filter === "all" && !state.search.trim();
  const favourites = createMemo(() =>
    state.chats.filter((c) => c.favorite).sort((a, b) => a.favoritePosition - b.favoritePosition),
  );
  const rest = createMemo(() => state.chats.filter((c) => !c.favorite));
  const rows = (list: Row[], compact = false) => (
    <For each={list}>
      {(row) => <ChatRow row={row} compact={compact} selected={state.open?.id === row.id} onOpen={() => commands.openChat(row.id)} />}
    </For>
  );
  return (
    <section
      class="relative flex h-full shrink-0 flex-col"
      style={{
        width: "var(--list-w)",
        background: "var(--panel-bg)",
        "border-right": "var(--hairline) solid var(--panel-border)",
      }}
      aria-label="Chats"
    >
      <header
        data-tauri-drag-region
        class="flex shrink-0 items-center justify-between"
        style={{ height: "var(--list-header-h)", "padding-left": "8.5px", "padding-right": "8px" }}
      >
        <button
          type="button"
          aria-label="Filter chats"
          aria-haspopup="menu"
          onClick={() => setMenu(!menu())}
          class="flex items-center justify-center rounded-full"
          style={{ ...roundButton, background: "var(--round-button-bg)", color: "var(--icon)" }}
        >
          <DotsIcon size={17} />
        </button>
        <h1
          data-tauri-drag-region
          style={{ margin: 0, "font-size": "var(--fs-title)", "font-weight": "var(--fw-title)", color: "var(--text-primary)" }}
        >
          {TITLES[state.section] ?? "Chats"}
        </h1>
        <button
          type="button"
          aria-label="New chat"
          class="flex items-center justify-center rounded-full"
          style={{ ...roundButton, background: "var(--new-chat-bg)", color: "var(--accent-fg)" }}
        >
          <PlusIcon size={18} />
        </button>
      </header>
      <Show when={menu()}>
        <FilterMenu onClose={() => setMenu(false)} />
      </Show>
      <div class="shrink-0" style={{ padding: "2.5px var(--search-inset) 0" }}>
        <label
          class="flex items-center"
          style={{ height: "var(--search-h)", "border-radius": "calc(var(--search-h) / 2)", background: "var(--search-bg)", "padding-left": "11px" }}
        >
          <span style={{ color: "var(--text-secondary)" }}><SearchIcon size={16} /></span>
          <input
            type="text"
            placeholder="Search"
            value={state.search}
            onInput={(e) => commands.search(e.currentTarget.value)}
            class="selectable h-full min-w-0 flex-1 bg-transparent outline-none"
            style={{ "padding-left": "7px", "padding-right": "12px", "font-size": "var(--fs-search)", color: "var(--text-primary)" }}
          />
        </label>
      </div>
      <div class="scroll min-h-0 flex-1" style={{ "padding-top": state.section === "chats" ? "15px" : "8px" }}>
        <Show when={state.section === "starred"}>
          <StarredList />
        </Show>
        <Show when={state.section === "calls"}>
          <CallsList />
        </Show>
        <Show when={state.section === "chats"}>
          <button
            type="button"
            onClick={() => commands.showSection("archived")}
            class="chat-row flex w-[calc(100%-2*var(--row-inset))] items-center text-left"
            style={{ height: "37px", margin: "0 var(--row-inset)", "padding-right": "var(--row-pad-r)", "border-radius": "var(--row-radius)" }}
          >
            <span class="flex items-center justify-center" style={{ width: "calc(var(--row-pad-l) + var(--avatar-row) + var(--row-gap))", color: "var(--text-secondary)" }}>
              <ArchiveIcon size={19} />
            </span>
            <span
              class="flex h-full flex-1 items-center"
              style={{ "font-size": "var(--fs-archived)", color: "var(--text-secondary)", "border-bottom": "var(--hairline) solid var(--divider)" }}
            >
              Archived
            </span>
          </button>
        </Show>
        <Show when={state.section !== "starred" && state.section !== "calls"}>
        <Show when={grouped()} fallback={<div role="listbox" aria-label="Chat list">{rows(state.chats)}</div>}>
          <Show when={favourites().length > 0}>
            <GroupHeading label="Favourites" open={!folded().includes("favourites")} onToggle={() => toggle("favourites")} />
            <Show when={!folded().includes("favourites")}>
              <div role="listbox" aria-label="Favourites">{rows(favourites(), true)}</div>
            </Show>
          </Show>
          <GroupHeading label="Chats" open={!folded().includes("chats")} onToggle={() => toggle("chats")} />
          <Show when={!folded().includes("chats")}>
            <div role="listbox" aria-label="Chats">{rows(rest())}</div>
          </Show>
        </Show>
        </Show>
      </div>
    </section>
  );
}
