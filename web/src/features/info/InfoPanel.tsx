// Contact and group info beside the conversation (REQ-WEBUI-011): the
// picture, name, number or member count, description, the chat's
// switches, members with their admin tag, groups in common, and the
// destructive actions behind a confirmation.
import { For, type JSX, Show } from "solid-js";
import { commands, state, type Info } from "../../lib/bridge";
import { ask } from "../../lib/ui";
import Avatar from "../../components/Avatar";
import { ArchiveIcon, BackIcon, HeartIcon, MuteIcon, PinIcon } from "../../components/Icons";

const timerLabel = (seconds: number) =>
  seconds === 0 ? "Off" : seconds <= 86_400 ? "24 hours" : seconds <= 604_800 ? "7 days" : "90 days";

function Section(p: { title?: string; children: JSX.Element }) {
  return (
    <section style={{ padding: "12px 16px", "border-bottom": "6px solid var(--sidebar-bg)" }}>
      <Show when={p.title}>
        <h3 style={{ margin: "0 0 8px", "font-size": "var(--fs-meta)", "font-weight": 600, color: "var(--text-secondary)" }}>{p.title}</h3>
      </Show>
      {p.children}
    </section>
  );
}

function Switch(p: { icon: JSX.Element; label: string; on: boolean; onToggle: () => void }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={p.on}
      onClick={p.onToggle}
      class="menu-item flex w-full items-center text-left"
      style={{ height: "34px", padding: "0 6px", gap: "12px", "border-radius": "8px", color: "var(--text-primary)", "font-size": "var(--fs-row-text)" }}
    >
      <span style={{ color: "var(--icon)", display: "inline-flex" }}>{p.icon}</span>
      <span class="flex-1">{p.label}</span>
      <span
        aria-hidden="true"
        class="relative inline-block rounded-full"
        style={{ width: "30px", height: "18px", background: p.on ? "var(--accent)" : "var(--switch-off)", transition: "background .15s" }}
      >
        <span
          class="absolute rounded-full"
          style={{ top: "2px", left: p.on ? "14px" : "2px", width: "14px", height: "14px", background: "#fff", transition: "left .15s" }}
        />
      </span>
    </button>
  );
}

function Danger(p: { label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={p.onClick}
      class="menu-item flex w-full items-center text-left"
      style={{ height: "34px", padding: "0 6px", "border-radius": "8px", color: "var(--danger)", "font-size": "var(--fs-row-text)" }}
    >
      {p.label}
    </button>
  );
}

function Body(p: { info: Info }) {
  const i = () => p.info;
  const sub = () => (i().group ? `Group · ${i().memberCount} members` : i().channel ? "Channel" : (i().phone ?? ""));
  return (
    <div class="scroll min-h-0 flex-1">
      <div class="flex flex-col items-center" style={{ padding: "24px 16px 16px", "border-bottom": "6px solid var(--sidebar-bg)" }}>
        <Avatar src={i().avatar} size={120} group={i().group} seed={i().id} />
        <h2 class="selectable text-center" style={{ margin: "14px 0 0", "font-size": "17px", "font-weight": 600, color: "var(--text-primary)" }}>
          {i().title}
        </h2>
        <Show when={sub()}>
          <p class="selectable" style={{ margin: "4px 0 0", "font-size": "var(--fs-row-text)", color: "var(--text-secondary)" }}>{sub()}</p>
        </Show>
      </div>
      <Show when={i().description}>
        <Section title="Description">
          <p class="selectable whitespace-pre-wrap break-words" style={{ margin: 0, "font-size": "var(--fs-msg)", "line-height": "var(--lh-msg)", color: "var(--text-primary)" }}>
            {i().description}
          </p>
        </Section>
      </Show>
      <Section>
        <Switch icon={<MuteIcon size={16} />} label="Mute notifications" on={i().muted} onToggle={() => commands.setMuted(i().id, i().muted ? null : 0)} />
        <Switch icon={<PinIcon size={16} />} label="Pin chat" on={i().pinned} onToggle={() => commands.setPinned(i().id, !i().pinned)} />
        <Switch icon={<HeartIcon size={16} />} label="Favourite" on={i().favorite} onToggle={() => commands.setFavorite(i().id, !i().favorite)} />
        <Switch icon={<ArchiveIcon size={16} />} label="Archived" on={i().archived} onToggle={() => commands.setArchived(i().id, !i().archived)} />
        <div class="flex items-center justify-between" style={{ height: "34px", padding: "0 6px", "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>
          Disappearing messages
          <span style={{ color: "var(--text-secondary)" }}>{timerLabel(i().timer)}</span>
        </div>
      </Section>
      <Show when={i().group && i().members.length > 0}>
        <Section title={`${i().memberCount} members`}>
          <For each={i().members}>
            {(member) => (
              <button
                type="button"
                onClick={() => commands.openInfo(member.id)}
                class="menu-item flex w-full items-center text-left"
                style={{ height: "46px", padding: "0 6px", gap: "10px", "border-radius": "8px" }}
              >
                <Avatar src={member.avatar} size={32} seed={member.id} />
                <span class="flex-1 truncate" style={{ "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>{member.name}</span>
                <Show when={member.admin}>
                  <span
                    class="shrink-0 rounded-[5px]"
                    style={{ padding: "1px 6px", "font-size": "var(--fs-msg-meta)", color: "var(--accent-text)", background: "var(--round-button-bg)" }}
                  >
                    Group admin
                  </span>
                </Show>
              </button>
            )}
          </For>
        </Section>
      </Show>
      <Show when={i().commonGroups.length > 0}>
        <Section title={`${i().commonGroups.length} ${i().commonGroups.length === 1 ? "group" : "groups"} in common`}>
          <For each={i().commonGroups}>
            {([id, title]) => (
              <button
                type="button"
                onClick={() => commands.openChat(id)}
                class="menu-item flex w-full items-center text-left"
                style={{ height: "46px", padding: "0 6px", gap: "10px", "border-radius": "8px" }}
              >
                <Avatar src={null} size={32} group seed={id} />
                <span class="truncate" style={{ "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>{title}</span>
              </button>
            )}
          </For>
        </Section>
      </Show>
      <Section>
        <Danger
          label="Clear chat"
          onClick={() =>
            ask({
              title: "Clear this chat?",
              body: "Messages will be removed from this computer and your phone. The chat stays in the list.",
              action: "Clear chat",
              run: () => commands.clearChat(i().id),
            })
          }
        />
        <Show
          when={i().group && i().canLeave}
          fallback={
            <Show when={!i().group}>
              <Danger
                label="Delete chat"
                onClick={() =>
                  ask({
                    title: `Delete chat with ${i().title}?`,
                    body: "The chat and its messages will be deleted from this computer and your phone.",
                    action: "Delete chat",
                    run: () => commands.deleteChat(i().id),
                  })
                }
              />
            </Show>
          }
        >
          <Danger
            label="Exit group"
            onClick={() =>
              ask({
                title: `Exit "${i().title}"?`,
                body: "You will stop getting messages from this group. Only the group's admins are told.",
                action: "Exit group",
                run: () => commands.leaveGroup(i().id),
              })
            }
          />
        </Show>
      </Section>
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
            style={{ height: "var(--conv-header-h)", padding: "0 12px", gap: "10px", "border-bottom": "var(--hairline) solid var(--conversation-header-border)" }}
          >
            <button type="button" aria-label="Close" onClick={() => commands.closeInfo()} class="flex items-center" style={{ color: "var(--icon)" }}>
              <BackIcon size={18} />
            </button>
            <h1 data-tauri-drag-region style={{ margin: 0, "font-size": "var(--fs-header-title)", "font-weight": 600, color: "var(--text-primary)" }}>
              {info().group ? "Group info" : info().channel ? "Channel info" : "Contact info"}
            </h1>
          </header>
          <Body info={info()} />
        </aside>
      )}
    </Show>
  );
}
