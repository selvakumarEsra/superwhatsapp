// Contact and group info beside the conversation (REQ-WEBUI-011), as the
// egui app draws it (src/ui/info.rs): a 380-point panel with a close cross
// and the title, then centred the large picture, the bold name, the number,
// presence and the disappearing-messages timer, and the actions as soft
// pills that wrap and centre (Message, Copy number, Mute, Pin, Archive,
// Block). Groups list their members and the Leave button below; a
// contact's groups in common follow. Clearing or deleting the chat lives
// in the chat menu, as in egui.
import { createSignal, For, type JSX, Show } from "solid-js";
import { commands, state, type Info, type ItemRow, type MediaCell } from "../../lib/bridge";
import { ask } from "../../lib/ui";
import { t } from "../../lib/i18n";
import Avatar from "../../components/Avatar";
import ZngaKnows from "./ZngaKnows";
import {
  ArchiveIcon, BanIcon, BellIcon, CheckIcon, ChevronLeftIcon, ChevronRightIcon, ExternalLinkIcon, FileTextIcon, ImageIcon, UserIcon, BellOffIcon, ChatsIcon, CloseIcon, CopyIcon, PinIcon, PinOffIcon, TimerIcon,
} from "../../components/Icons";

const timerLabel = (seconds: number) =>
  seconds <= 86_400 ? "24 hours" : seconds <= 604_800 ? "7 days" : "90 days";

// theme::soft_button: medium 13, 15-point icon, 12×7 padding, a full pill
// on the surface colour.
function Pill(p: { icon: JSX.Element; label: string; onClick: () => void; disabled?: boolean }) {
  return (
    <button
      type="button"
      disabled={p.disabled}
      onClick={p.onClick}
      class="soft-pill flex items-center rounded-full"
      style={{
        padding: "7px 12px",
        gap: "6px",
        "font-size": "13px",
        "line-height": "16px",
        "font-weight": 500,
        background: "var(--surface)",
        color: "var(--text-primary)",
        opacity: p.disabled ? 0.45 : 1,
      }}
    >
      <span class="flex">{p.icon}</span>
      {t(p.label)}
    </button>
  );
}

function ListRow(p: { avatar: string | null; group?: boolean; seed: string; name: string; tag?: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={p.onClick}
      class="menu-item flex w-full items-center text-left"
      style={{ height: "52px", padding: "0 6px", gap: "12px", "border-radius": "10px" }}
    >
      <Avatar src={p.avatar} size={40} group={p.group} seed={p.seed} />
      <span class="flex-1 truncate" style={{ "font-size": "14px", color: "var(--text-primary)" }}>{p.name}</span>
      <Show when={p.tag}>
        <span class="shrink-0" style={{ "font-size": "12px", color: "var(--accent-text)" }}>{p.tag}</span>
      </Show>
    </button>
  );
}

function Heading(p: { children: JSX.Element }) {
  return (
    <div style={{ margin: "18px 0 4px", "font-size": "12.5px", "font-weight": 500, color: "var(--text-secondary)" }}>{p.children}</div>
  );
}

/** The name editor egui opens in place of the name: First name and
 *  Surname, saved with Enter or the check, dropped with Escape. */
function NameEditor(p: { info: Info; onDone: () => void }) {
  const [first, setFirst] = createSignal(p.info.editorNames[0]);
  const [last, setLast] = createSignal(p.info.editorNames[1]);
  const save = () => {
    if (!first().trim() && !last().trim()) return;
    void commands.saveContact(p.info.id, first().trim(), last().trim());
    p.onDone();
  };
  const field = (value: () => string, set: (v: string) => void, hint: string, focus: boolean) => (
    <input
      value={value()}
      placeholder={t(hint)}
      aria-label={t(hint)}
      ref={(el) => focus && queueMicrotask(() => el.focus())}
      onInput={(e) => set(e.currentTarget.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter") save();
        if (e.key === "Escape") { e.stopPropagation(); p.onDone(); }
      }}
      class="selectable outline-none"
      style={{ width: "128px", padding: "5px 10px", "border-radius": "8px", background: "var(--surface)", border: "none", "font-size": "15px", "font-weight": 600, color: "var(--text-primary)" }}
    />
  );
  return (
    <div class="flex items-center" style={{ gap: "6px", "margin-top": "14px" }}>
      {field(first, setFirst, "First name", true)}
      {field(last, setLast, "Surname", false)}
      <button type="button" title={t("Save name (Enter)")} aria-label={t("Save name (Enter)")} onClick={save} class="icon-button flex rounded-full" style={{ padding: "4px", color: "var(--text-secondary)" }}>
        <CheckIcon size={18} />
      </button>
    </div>
  );
}

/** A picture or video cell: the preview cropped to fill, a play disc and
 *  the length on a video (info::media_cell). A cell goes to its message. */
function Cell(p: { chat: string; cell: MediaCell; size: string }) {
  const length = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  return (
    <button
      type="button"
      aria-label={t(p.cell.video ? "Video" : "Photo")}
      onClick={() => commands.goToMessage(p.chat, p.cell.id)}
      class="relative shrink-0 overflow-hidden"
      style={{ width: p.size, height: p.size, "border-radius": "6px", background: "var(--surface)", color: "var(--text-meta)" }}
    >
      <Show when={p.cell.src} fallback={<span class="absolute inset-0 flex items-center justify-center"><ImageIcon size={22} /></span>}>
        <img src={p.cell.src!} alt="" loading="lazy" class="absolute inset-0 h-full w-full object-cover" />
      </Show>
      <Show when={p.cell.video}>
        <span class="absolute left-1/2 top-1/2 flex items-center justify-center rounded-full" style={{ width: "30px", height: "30px", transform: "translate(-50%, -50%)", background: "rgba(0,0,0,0.45)" }}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="#fff" aria-hidden="true"><path d="M7 4v16l13-8z" /></svg>
        </span>
        <Show when={p.cell.seconds !== null}>
          <span class="absolute" style={{ right: "4px", bottom: "4px", padding: "0 4px", "border-radius": "3px", background: "rgba(0,0,0,0.55)", color: "#fff", "font-size": "11px", "line-height": "16px" }}>
            {length(p.cell.seconds!)}
          </span>
        </Show>
      </Show>
    </button>
  );
}

/** widgets::empty_state: 48 down, a 40-point icon, the title semibold 16,
 *  the line regular 13.5. */
function Empty(p: { icon: JSX.Element; title: string; body: string }) {
  return (
    <div class="flex flex-col items-center text-center" style={{ "padding-top": "48px", gap: "2px", color: "var(--text-meta)" }}>
      {p.icon}
      <span style={{ "margin-top": "6px", "font-size": "16px", "font-weight": 600, color: "var(--text-primary)" }}>{p.title}</span>
      <span style={{ "font-size": "13.5px", color: "var(--text-secondary)", padding: "0 14px" }}>{p.body}</span>
    </div>
  );
}

/** info::item_row: a 60-point row, a 40-point tile with the icon, the
 *  title medium 13.5 and the detail regular 12, a rule under the text. */
function Item(p: { icon: JSX.Element; row: ItemRow; onClick: () => void }) {
  return (
    <button type="button" onClick={p.onClick} class="menu-item relative flex w-full items-center text-left" style={{ height: "60px", padding: "0 14px" }}>
      <span class="flex shrink-0 items-center justify-center" style={{ width: "40px", height: "40px", "border-radius": "8px", background: "var(--surface)", color: "var(--text-secondary)" }}>
        {p.icon}
      </span>
      <span class="flex min-w-0 flex-1 flex-col" style={{ "margin-left": "12px", gap: "4px" }}>
        <span class="truncate" style={{ "font-size": "13.5px", "font-weight": 500, color: "var(--text-primary)" }}>{p.row.title}</span>
        <span class="truncate" style={{ "font-size": "12px", color: "var(--text-meta)" }}>{p.row.detail}</span>
      </span>
      <span class="absolute bottom-0" style={{ left: "66px", right: "14px", height: "1px", background: "var(--panel-border)" }} />
    </button>
  );
}

/** info::media_view: the Media, Docs and Links chips with their counts
 *  under a rule, then the open tab: a grid of four (three when narrow),
 *  or rows of documents or links. A document goes to its message; a link
 *  opens in the browser. */
function MediaView(p: { info: Info }) {
  const tabs = [["media", "Media"], ["docs", "Docs"], ["links", "Links"]] as const;
  const cell = "calc((380px - 28px - 12px) / 4)";
  return (
    <div class="flex min-h-0 flex-1 flex-col">
      <div class="flex shrink-0" style={{ padding: "16px 14px", gap: "6px", "border-bottom": "1px solid var(--panel-border)" }}>
        <For each={tabs}>
          {([page, label], index) => {
            const on = () => p.info.page === page;
            return (
              <button
                type="button"
                role="tab"
                aria-selected={on()}
                onClick={() => commands.infoPage(page)}
                class="chip flex items-center"
                classList={{ on: on() }}
                style={{
                  height: "var(--chip-h)", padding: "0 9px", gap: "5px", "border-radius": "calc(var(--chip-h) / 2)",
                  border: `1px solid ${on() ? "transparent" : "var(--chip-border)"}`,
                  background: on() ? "var(--chip-on-bg)" : "var(--chip-bg)",
                  color: on() ? "var(--chip-on-fg)" : "var(--text-secondary)",
                  "font-size": "var(--fs-chip)", "font-weight": 500,
                }}
              >
                {t(label)}
                <Show when={p.info.tabCounts[index()] > 0}>
                  <span style={{ "font-size": "var(--fs-meta)", "font-weight": 400 }}>{p.info.tabCounts[index()]}</span>
                </Show>
              </button>
            );
          }}
        </For>
      </div>
      <div class="scroll min-h-0 flex-1">
        <Show when={p.info.tabTruncated}>
          <p style={{ margin: "8px 14px 4px", "font-size": "12px", color: "var(--text-meta)" }}>{t("Only the newest items are listed.")}</p>
        </Show>
        <Show when={p.info.mediaFailed}>
          <div class="flex flex-wrap items-center" style={{ margin: "24px 14px", gap: "8px", "font-size": "12.5px", color: "var(--text-meta)" }}>
            {t("Could not list the media.")}
            <button type="button" onClick={() => commands.retryInfoMedia()} class="soft-pill" style={{ padding: "7px 12px", "border-radius": "999px", background: "var(--surface)", "font-size": "13px", "font-weight": 500, color: "var(--text-primary)" }}>
              {t("Try again")}
            </button>
          </div>
        </Show>
        <Show when={p.info.mediaPending && !p.info.mediaFailed}>
          <div class="flex justify-center" style={{ "padding-top": "32px", color: "var(--text-secondary)" }}>
            <span class="spinner" style={{ width: "22px", height: "22px" }} />
          </div>
        </Show>
        <Show when={!p.info.mediaPending && !p.info.mediaFailed}>
          <Show when={p.info.page === "media"}>
            <Show when={p.info.gallery.length > 0} fallback={<Empty icon={<ImageIcon size={40} />} title={t("No media")} body={t("Photos and videos shared in this chat will show here.")} />}>
              <div class="flex flex-wrap" style={{ padding: "6px 14px", gap: "4px" }}>
                <For each={p.info.gallery}>{(c) => <Cell chat={p.info.id} cell={c} size={cell} />}</For>
              </div>
            </Show>
          </Show>
          <Show when={p.info.page === "docs"}>
            <Show when={p.info.docs.length > 0} fallback={<Empty icon={<FileTextIcon size={40} />} title={t("No documents")} body={t("Files shared in this chat will show here.")} />}>
              <For each={p.info.docs}>{(row) => <Item icon={<FileTextIcon size={20} />} row={row} onClick={() => commands.goToMessage(p.info.id, row.target)} />}</For>
            </Show>
          </Show>
          <Show when={p.info.page === "links"}>
            <Show when={p.info.links.length > 0} fallback={<Empty icon={<ExternalLinkIcon size={40} />} title={t("No links")} body={t("Links shared in this chat will show here.")} />}>
              <For each={p.info.links}>{(row) => <Item icon={<ExternalLinkIcon size={20} />} row={row} onClick={() => commands.openUrl(row.target)} />}</For>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
}

/** info::media_row: the title with the count and a chevron, then the
 *  three newest pictures in 4-point-apart squares of up to 110. A video
 *  shows a play disc and its length. A picture opens its chat. */
function MediaStrip(p: { info: Info }) {
  const length = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  return (
    <div style={{ "margin-top": "8px" }}>
      <button
        type="button"
        onClick={() => commands.infoPage("media")}
        class="menu-item flex w-full items-center justify-between text-left"
        style={{ height: "50px", margin: "0 -6px", padding: "0 6px", width: "calc(100% + 12px)", "border-radius": "6px" }}
      >
        <span style={{ "font-size": "13.5px", color: "var(--text-secondary)" }}>{t("Media, links and docs")}</span>
        <span class="flex items-center" style={{ gap: "2px", "font-size": "13px", color: "var(--text-meta)" }}>
          {p.info.mediaCount ?? ""}
          <ChevronRightIcon size={18} />
        </span>
      </button>
      <Show when={p.info.media.length > 0}>
        <div class="flex" style={{ gap: "4px" }}>
          <For each={p.info.media}>
            {(cell) => (
              <button
                type="button"
                aria-label={t(cell.video ? "Video" : "Photo")}
                onClick={() => commands.goToMessage(p.info.id, cell.id)}
                class="relative overflow-hidden"
                style={{ width: "min(110px, calc((100% - 8px) / 3))", "aspect-ratio": "1", "border-radius": "6px", background: "var(--surface)" }}
              >
                <Show when={cell.src}>
                  <img src={cell.src!} alt="" class="absolute inset-0 h-full w-full object-cover" />
                </Show>
                <Show when={cell.video}>
                  <span class="absolute left-1/2 top-1/2 flex items-center justify-center rounded-full" style={{ width: "30px", height: "30px", transform: "translate(-50%, -50%)", background: "rgba(0,0,0,0.45)" }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="#fff" aria-hidden="true"><path d="M7 4v16l13-8z" /></svg>
                  </span>
                  <Show when={cell.seconds !== null}>
                    <span class="absolute" style={{ right: "4px", bottom: "4px", padding: "0 4px", "border-radius": "3px", background: "rgba(0,0,0,0.55)", color: "#fff", "font-size": "11px", "line-height": "16px" }}>
                      {length(cell.seconds!)}
                    </span>
                  </Show>
                </Show>
              </button>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

function Body(p: { info: Info }) {
  const i = () => p.info;
  const [editing, setEditing] = createSignal(false);
  const contact = () => !i().group && !i().channel;
  const copy = () => {
    if (i().phone) void navigator.clipboard?.writeText(`+${i().phone}`);
  };
  return (
    <div class="scroll min-h-0 flex-1" style={{ padding: "12px 14px 24px" }}>
      <div class="flex flex-col items-center text-center">
        <div style={{ "margin-top": "6px" }}>
          <Avatar src={i().avatar} size={194} group={i().group} seed={i().id} />
        </div>
        <Show
          when={editing() && i().contactEditable}
          fallback={
            <h2 class="selectable" style={{ margin: "14px 0 0", "font-size": "19px", "line-height": "24px", "font-weight": 700, color: "var(--text-primary)" }}>
              {i().title}
            </h2>
          }
        >
          <NameEditor info={i()} onDone={() => setEditing(false)} />
        </Show>
        <Show when={i().group}>
          <p style={{ margin: "4px 0 0", "font-size": "13px", color: "var(--text-secondary)" }}>
            {t("Group")} · {i().memberCount} {t(i().memberCount === 1 ? "member" : "members")}
          </p>
        </Show>
        <Show when={contact() && i().phoneDisplay}>
          <p class="selectable" style={{ margin: "4px 0 0", "font-size": "14px", color: "var(--text-secondary)" }}>{i().phoneDisplay}</p>
        </Show>
        <Show when={i().status}>
          <p style={{ margin: "5px 0 0", "font-size": "12.5px", color: "var(--text-meta)" }}>{i().status}</p>
        </Show>
        <Show when={!i().channel && !i().left}>
          <p class="flex items-center" style={{ margin: "10px 0 0", gap: "5px", "font-size": "12.5px", color: "var(--text-secondary)" }}>
            <TimerIcon size={13} />
            {i().timer ? `${t("Disappearing messages")}: ${t(timerLabel(i().timer))}` : t("Disappearing messages")}
          </p>
        </Show>
        <Show when={i().description}>
          <p class="selectable whitespace-pre-wrap break-words" style={{ margin: "12px 0 0", "font-size": "13.5px", "line-height": "18px", color: "var(--text-primary)" }}>
            {i().description}
          </p>
        </Show>
        <div class="flex flex-wrap justify-center" style={{ gap: "8px", "margin-top": "16px" }}>
          <Show when={contact()}>
            <Pill icon={<ChatsIcon size={15} />} label="Message" onClick={() => { void commands.openChat(i().id); void commands.closeInfo(); }} />
          </Show>
          <Show when={i().contactEditable}>
            <Pill icon={<UserIcon size={15} />} label={i().known ? "Rename" : "Add to contacts"} onClick={() => setEditing(true)} />
          </Show>
          <Show when={i().phone}>
            <Pill icon={<CopyIcon size={15} />} label="Copy number" onClick={copy} />
          </Show>
          <Pill
            icon={i().muted ? <BellIcon size={15} /> : <BellOffIcon size={15} />}
            label={i().muted ? "Unmute" : "Mute"}
            onClick={() => commands.setMuted(i().id, i().muted ? null : 0)}
          />
          <Pill
            icon={i().pinned ? <PinOffIcon size={15} /> : <PinIcon size={15} />}
            label={i().pinned ? "Unpin" : "Pin"}
            onClick={() => commands.setPinned(i().id, !i().pinned)}
          />
          <Pill
            icon={<ArchiveIcon size={15} />}
            label={i().archived ? "Unarchive" : "Archive"}
            onClick={() => { void commands.setArchived(i().id, !i().archived); void commands.closeInfo(); }}
          />
        </div>
        {/* blocking::button: on a row of its own under the others. */}
        <div class="flex justify-center" style={{ "margin-top": "6px" }}>
            <Show when={contact() && i().blockable}>
              <Pill
                icon={<BanIcon size={15} />}
                label={i().blocked ? "Unblock contact" : "Block contact"}
                disabled={!i().blockEditable}
                onClick={() =>
                  ask({
                    title: t(i().blocked ? "Unblock contact?" : "Block contact?"),
                    body: i().blocked
                      ? t("They will be able to call you and send you messages.")
                      : t("Blocked contacts cannot call you or send you messages."),
                    action: t(i().blocked ? "Unblock" : "Block"),
                    run: () => commands.setBlocked(i().id, !i().blocked),
                  })
                }
              />
            </Show>
        </div>
      </div>
      <Show when={!i().channel}>
        <MediaStrip info={i()} />
      </Show>
      <Show when={i().znga}>{(profile) => <ZngaKnows chat={i().id} profile={profile()} />}</Show>
      <Show when={i().group && i().members.length > 0}>
        <Heading>{t("Members")} ({i().members.length})</Heading>
        <For each={i().members}>
          {(member) => (
            <ListRow avatar={member.avatar} seed={member.id} name={member.name} tag={member.admin ? t("Group admin") : undefined} onClick={() => commands.openInfo(member.id)} />
          )}
        </For>
      </Show>
      <Show when={i().commonGroups.length > 0}>
        <Heading>
          {i().commonGroups.length} {t(i().commonGroups.length === 1 ? "group in common" : "groups in common")}
        </Heading>
        <For each={i().commonGroups}>{([id, title]) => <ListRow avatar={null} group seed={id} name={title} onClick={() => commands.openChat(id)} />}</For>
      </Show>
      <Show when={i().group && i().canLeave}>
        <div class="flex justify-center" style={{ "margin-top": "16px" }}>
          <button
            type="button"
            class="soft-pill rounded-full"
            style={{ padding: "7px 14px", "font-size": "13px", "font-weight": 500, background: "var(--surface)", color: "var(--danger)" }}
            onClick={() =>
              ask({
                title: `${t("Exit")} "${i().title}"?`,
                body: t("You will stop getting messages from this group. Only the group's admins are told."),
                action: t("Exit group"),
                run: () => commands.leaveGroup(i().id),
              })
            }
          >
            {t(i().channel ? "Leave channel" : "Leave group")}
          </button>
        </div>
      </Show>
    </div>
  );
}

export default function InfoPanel() {
  return (
    <Show when={state.info}>
      {(info) => (
        <aside
          class="flex h-full shrink-0 flex-col"
          aria-label={info().group ? "Group info" : "Contact info"}
          style={{ width: "var(--info-w)", background: "var(--panel-bg)", "border-left": "var(--hairline) solid var(--panel-border)" }}
          onKeyDown={(e) => e.key === "Escape" && commands.closeInfo()}
        >
          <header
            data-tauri-drag-region
            class="flex shrink-0 items-center"
            style={{ height: "var(--list-header-h)", padding: "0 14px", gap: "12px", "border-bottom": "var(--hairline) solid var(--conversation-header-border)" }}
          >
            <Show
              when={info().page !== "overview"}
              fallback={
                <button
                  type="button"
                  aria-label={t("Close")}
                  onClick={() => commands.closeInfo()}
                  class="icon-button flex items-center justify-center rounded-full"
                  style={{ width: "30px", height: "30px", color: "var(--icon)" }}
                >
                  <CloseIcon size={20} />
                </button>
              }
            >
              <button
                type="button"
                aria-label={t("Back")}
                onClick={() => commands.infoPage("overview")}
                class="icon-button flex items-center justify-center rounded-full"
                style={{ width: "30px", height: "30px", color: "var(--icon)" }}
              >
                <ChevronLeftIcon size={20} />
              </button>
            </Show>
            <h1 data-tauri-drag-region style={{ margin: 0, "font-size": "16px", "font-weight": 600, color: "var(--text-primary)" }}>
              {t(info().page !== "overview" ? "Media, links and docs" : info().group ? "Group info" : info().channel ? "Channel info" : "Contact info")}
            </h1>
          </header>
          <Show when={info().page !== "overview"} fallback={<Body info={info()} />}>
            <MediaView info={info()} />
          </Show>
        </aside>
      )}
    </Show>
  );
}
