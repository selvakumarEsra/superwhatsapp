// The chat list column (REQ-WEBUI-006), as the egui app draws it
// (src/ui/chats.rs): a large title with the side panel and new chat
// buttons, a pill search field, the filter chips with their unread counts,
// then one ungrouped list, the Znga chat first while Znga is on.
import { createEffect, For, on, Show } from "solid-js";
import { commands, state } from "../../lib/bridge";
import { BackIcon, NewChatIcon, PanelIcon, PhonePlusIcon, SearchIcon } from "../../components/Icons";
import ChatRow from "./ChatRow";
import StarredList from "../starred/StarredList";
import CallsList, { NewCallList } from "../calls/CallsList";
import { t } from "../../lib/i18n";

const TITLES: Record<string, string> = {
  chats: "Chats",
  favourites: "Favourites",
  archived: "Archived",
  starred: "Starred",
  calls: "Calls",
  locked: "Locked chats",
};

function HeaderButton(p: { label: string; children: import("solid-js").JSX.Element; onClick?: () => void }) {
  return (
    <button
      type="button"
      title={t(p.label)}
      aria-label={t(p.label)}
      onClick={() => p.onClick?.()}
      class="icon-button flex items-center justify-center rounded-full"
      style={{ width: "var(--round-button)", height: "var(--round-button)", color: "var(--icon)" }}
    >
      {p.children}
    </button>
  );
}

/** One filter chip: the chosen one tinted in the accent, the rest outlined. */
function Chip(p: { label: string; count: number; on: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      role="tab"
      aria-selected={p.on}
      onClick={p.onClick}
      class="chip flex shrink-0 items-center"
      classList={{ on: p.on }}
      style={{
        height: "var(--chip-h)",
        padding: "0 9px",
        gap: "5px",
        "border-radius": "calc(var(--chip-h) / 2)",
        border: `1px solid ${p.on ? "transparent" : "var(--chip-border)"}`,
        background: p.on ? "var(--chip-on-bg)" : "var(--chip-bg)",
        color: p.on ? "var(--chip-on-fg)" : "var(--text-secondary)",
        "font-size": "var(--fs-chip)",
        "font-weight": 500,
      }}
    >
      {p.label}
      <Show when={p.count > 0}>
        <span style={{ "font-size": "var(--fs-meta)", "font-weight": 400 }}>{p.count}</span>
      </Show>
    </button>
  );
}

export default function ChatList() {
  // The Znga chat stays first while Znga is on (REQ-ZCHAT-001.A1); a
  // search shows it only when it matches.
  const zngaRow = () => {
    const row = state.zngaRow;
    const query = state.search.trim().toLowerCase();
    return row && (!query || "znga".includes(query) || query.includes("znga")) ? row : null;
  };
  // Starred, Archived and Locked open from the rail with a way back.
  const sub = () => ["starred", "archived", "locked"].includes(state.section);
  // Calls › New call: a back arrow, "New call" bold 20, its own search.
  const newCall = () => state.section === "calls" && !!state.newCall;
  // chats.rs:search_and_filters: the New call list opens ready for a name.
  let searchEl: HTMLInputElement | undefined;
  createEffect(on(newCall, (open) => open && queueMicrotask(() => searchEl?.focus())));
  const listing = () => state.section !== "starred" && state.section !== "calls";
  // egui (src/ui/chats.rs:filter_chips) keeps the chips in Archived and
  // Locked with none lit, and hides them while searching the chat list.
  const folder = () => state.section === "archived" || state.section === "locked";
  const chips = () => listing() && (state.section === "locked" || !state.search.trim());
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
        class="flex shrink-0 items-center"
        style={{ height: "var(--list-header-h)", "padding-left": "var(--list-title-l)", "padding-right": "11px", gap: "4px" }}
      >
        <Show when={sub() || newCall()}>
          <button
            type="button"
            title={t(newCall() ? "Back to the calls" : "Back to chats")}
            aria-label={t(newCall() ? "Back to the calls" : "Back to chats")}
            onClick={() => (newCall() ? commands.showNewCall(false) : commands.showSection("chats"))}
            class="icon-button flex shrink-0 items-center justify-center rounded-full"
            style={{ width: "30px", height: "30px", "margin-right": "4px", color: "var(--text-secondary)" }}
          >
            <BackIcon size={18} />
          </button>
        </Show>
        <h1
          data-tauri-drag-region
          class="flex-1 truncate"
          style={{ margin: 0, "font-size": sub() ? "16px" : "var(--fs-title)", "font-weight": "var(--fw-title)", color: "var(--text-primary)" }}
        >
          {t(newCall() ? "New call" : TITLES[state.section] ?? "Chats")}
        </h1>
        {/* egui shows no buttons over the starred messages. */}
        <Show when={state.section !== "starred" && state.section !== "calls"}>
          <HeaderButton label="Hide chat list"><PanelIcon size={18} /></HeaderButton>
          <HeaderButton label="New chat"><NewChatIcon size={18} /></HeaderButton>
        </Show>
        {/* chats.rs:header_buttons: Calls has New call, none over its list. */}
        <Show when={state.section === "calls" && !newCall()}>
          <HeaderButton label="New call" onClick={() => commands.showNewCall(true)}><PhonePlusIcon size={18} /></HeaderButton>
        </Show>
      </header>
      <div class="shrink-0" style={{ padding: "4px var(--search-inset) 0" }}>
        <label
          class="search-pill flex items-center"
          style={{ height: "var(--search-h)", "border-radius": "calc(var(--search-h) / 2)", "padding-left": "11px" }}
        >
          <span style={{ color: "var(--icon)" }}><SearchIcon size={17} /></span>
          <input
            ref={searchEl}
            type="text"
            placeholder={t(state.section === "calls" ? "Search name or number" : "Search")}
            value={newCall() ? state.newCall!.search : state.section === "calls" ? state.callsSearch : state.search}
            onInput={(e) =>
              newCall()
                ? commands.searchNewCall(e.currentTarget.value)
                : state.section === "calls"
                  ? commands.searchCalls(e.currentTarget.value)
                  : commands.search(e.currentTarget.value)
            }
            class="selectable h-full min-w-0 flex-1 bg-transparent outline-none"
            style={{ "padding-left": "9px", "padding-right": "12px", "font-size": "var(--fs-search)", color: "var(--text-primary)" }}
          />
        </label>
      </div>
      <Show when={state.section === "calls" && !newCall()}>
        <div role="tablist" aria-label="Filters" class="chip-row flex shrink-0" style={{ padding: "14px var(--search-inset) 0", gap: "4px" }}>
          <Chip label={t("All")} count={0} on={state.callFilter === "all"} onClick={() => commands.setCallFilter("all")} />
          {/* A second click on Missed returns to every call. */}
          <Chip label={t("Missed")} count={0} on={state.callFilter === "missed"} onClick={() => commands.setCallFilter(state.callFilter === "missed" ? "all" : "missed")} />
        </div>
      </Show>
      <Show when={chips()}>
        <div role="tablist" aria-label="Filters" class="chip-row flex shrink-0" style={{ padding: "14px var(--search-inset) 0", gap: "4px" }}>
          <For each={state.filters}>
            {(chip) => (
              <Chip
                label={chip.label}
                count={chip.count}
                on={!folder() && state.filter === chip.id}
                // A second click on the chosen chip returns to every chat.
                onClick={() => commands.setFilter(!folder() && state.filter === chip.id ? "all" : chip.id)}
              />
            )}
          </For>
        </div>
      </Show>
      <div class="scroll min-h-0 flex-1" style={{ "padding-top": chips() ? "12px" : state.section === "starred" ? "14px" : state.section === "calls" ? "12px" : "8px" }}>
        <Show when={state.section === "starred"}>
          <StarredList />
        </Show>
        <Show when={state.section === "calls"}>
          <Show when={newCall()} fallback={<CallsList />}>
            <NewCallList />
          </Show>
        </Show>
        <Show when={state.section === "chats" && zngaRow()}>
          {(row) => <ChatRow row={row()} selected={!!state.znga} onOpen={() => commands.openChat(row().id)} />}
        </Show>
        <Show when={listing()}>
          <div role="listbox" aria-label="Chat list">
            <For each={state.chats}>
              {(row) => <ChatRow row={row} selected={!state.znga && state.open?.id === row.id} onOpen={() => commands.openChat(row.id)} />}
            </For>
          </div>
        </Show>
      </div>
    </section>
  );
}
