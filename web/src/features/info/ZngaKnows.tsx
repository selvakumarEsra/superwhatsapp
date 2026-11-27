// "Znga knows" in the contact and group info panel (REQ-ZPROF-007): the
// profile Znga keeps as the person's digital twin. Each value shows where
// it came from (learned, or confirmed by you); sensitive ones are marked
// and stay out of drafts until confirmed (REQ-ZPROF-003). A group adds its
// active members, their moods and the trend (REQ-ZPROF-004).
import { createEffect, createSignal, For, on, Show } from "solid-js";
import { commands, type ZField, type ZProfile } from "../../lib/bridge";
import { t } from "../../lib/i18n";
import { CheckIcon, CloseIcon } from "../../components/Icons";

const trendMark = (trend: string) => (trend === "up" ? " ↑" : trend === "down" ? " ↓" : "");
const moodColor = (mood: string | null) =>
  mood === "positive" ? "var(--accent-text)" : mood === "negative" ? "var(--danger)" : "var(--text-secondary)";

function TextButton(p: { label: string; onClick: () => void; disabled?: boolean }) {
  return (
    <button
      type="button"
      disabled={p.disabled}
      onClick={p.onClick}
      class="soft-pill rounded-full"
      style={{ padding: "4px 10px", "font-size": "12px", "font-weight": 500, background: "var(--surface)", color: "var(--text-primary)", opacity: p.disabled ? 0.45 : 1 }}
    >
      {t(p.label)}
    </button>
  );
}

function Field(p: { chat: string; field: ZField }) {
  const [adding, setAdding] = createSignal(false);
  const [text, setText] = createSignal("");
  const save = () => {
    const value = text().trim();
    if (value) void commands.zngaProfile(p.chat, "set", p.field.key, value);
    setText("");
    setAdding(false);
  };
  return (
    <div style={{ padding: "7px 0", "border-bottom": "var(--hairline) solid var(--divider)" }}>
      <div class="flex items-center" style={{ gap: "8px" }}>
        <span class="flex-1" style={{ "font-size": "12.5px", color: "var(--text-secondary)" }}>{t(p.field.label)}</span>
        <Show when={!p.field.local && !adding()}>
          <button
            type="button"
            aria-label={`${p.field.single && p.field.values.length ? "Change" : "Add"} ${p.field.label}`}
            onClick={() => {
              setText(p.field.single ? p.field.values[0]?.text ?? "" : "");
              setAdding(true);
            }}
            style={{ "font-size": "12px", color: "var(--accent-text)" }}
          >
            {t(p.field.single && p.field.values.length ? "Change" : "Add")}
          </button>
        </Show>
      </div>
      <For each={p.field.values}>
        {(value) => (
          <div class="flex items-start" style={{ gap: "6px", "margin-top": "4px" }}>
            <span class="selectable min-w-0 flex-1" style={{ "font-size": "14px", "line-height": "18px", color: "var(--text-primary)" }}>
              {value.text}
              <span style={{ "margin-left": "6px", "font-size": "11.5px", color: value.sensitive && !value.confirmed ? "var(--danger)" : "var(--text-meta)" }}>
                {value.sensitive && !value.confirmed ? t("sensitive, not used") : value.confirmed ? t("confirmed") : t("learned")}
              </span>
            </span>
            <Show when={!value.confirmed && !p.field.local}>
              <button type="button" title={t("Confirm")} aria-label={`Confirm ${value.text}`} onClick={() => void commands.zngaProfile(p.chat, "confirm", p.field.key, value.text)} style={{ color: "var(--accent-text)" }}>
                <CheckIcon size={16} />
              </button>
            </Show>
            <button type="button" title={t("Delete")} aria-label={`Delete ${value.text}`} onClick={() => void commands.zngaProfile(p.chat, "delete", p.field.key, value.text)} style={{ color: "var(--text-secondary)" }}>
              <CloseIcon size={16} />
            </button>
          </div>
        )}
      </For>
      <Show when={adding()}>
        <input
          aria-label={p.field.label}
          value={text()}
          ref={(el) => queueMicrotask(() => el.focus())}
          onInput={(e) => setText(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") save();
            if (e.key === "Escape") setAdding(false);
          }}
          onBlur={save}
          class="selectable w-full outline-none"
          style={{ height: "28px", padding: "0 8px", "margin-top": "6px", "border-radius": "6px", border: "1px solid var(--divider)", background: "var(--surface)", color: "var(--text-primary)", "font-size": "14px" }}
        />
      </Show>
    </div>
  );
}

export default function ZngaKnows(p: { chat: string; profile: ZProfile }) {
  // A group's members are recounted on this computer when its panel opens.
  createEffect(on(() => p.chat, (chat) => p.profile.group && void commands.zngaProfile(chat, "count", "", "")));
  const [more, setMore] = createSignal(false);
  const known = () => p.profile.fields.filter((f) => f.values.length > 0);
  const unknown = () => p.profile.fields.filter((f) => f.values.length === 0 && !f.local);
  const updated = () =>
    p.profile.learnedAt
      ? new Date(p.profile.learnedAt * 1000).toLocaleDateString(undefined, { day: "numeric", month: "short" })
      : null;
  return (
    <section aria-label={t("Znga knows")} style={{ "margin-top": "18px" }}>
      <div class="flex items-center" style={{ gap: "8px", "margin-bottom": "4px" }}>
        <span class="flex-1" style={{ "font-size": "12.5px", "font-weight": 500, color: "var(--text-secondary)" }}>
          {t("Znga knows")}
          <Show when={updated()}>{(day) => <span style={{ "font-weight": 400 }}> · {t("updated")} {day()}</span>}</Show>
        </span>
        <Show when={p.profile.learning} fallback={
          <>
            <TextButton label="Refresh" onClick={() => void commands.zngaProfile(p.chat, "refresh", "", "")} />
            <TextButton label="Learn from history" onClick={() => void commands.zngaProfile(p.chat, "learn", "", "")} />
          </>
        }>
          <span style={{ "font-size": "12px", color: "var(--text-secondary)" }}>{t("Learning…")}</span>
        </Show>
      </div>
      <Show when={known().length === 0 && p.profile.facts.length === 0 && p.profile.active.length === 0}>
        <div style={{ padding: "6px 0", "font-size": "13px", color: "var(--text-secondary)" }}>
          {t("Nothing yet. Znga learns as messages arrive, or press Learn from history.")}
        </div>
      </Show>
      <For each={known()}>{(field) => <Field chat={p.chat} field={field} />}</For>
      <Show when={p.profile.group && p.profile.active.length > 0}>
        <div style={{ padding: "7px 0", "border-bottom": "var(--hairline) solid var(--divider)" }}>
          <div style={{ "font-size": "12.5px", color: "var(--text-secondary)" }}>
            {t("Active members, 30 days")}
            <Show when={p.profile.moodTrend}>
              <span> · {t("mood")}{trendMark(p.profile.moodTrend)}</span>
            </Show>
          </div>
          <For each={p.profile.active.slice(0, 10)}>
            {(member) => (
              <button
                type="button"
                onClick={() => void commands.openInfo(member.id)}
                class="menu-item flex w-full items-center text-left"
                style={{ gap: "8px", padding: "3px 4px", "margin-top": "2px", "border-radius": "6px", "font-size": "14px" }}
              >
                <span class="min-w-0 flex-1 truncate" style={{ color: member.former ? "var(--text-secondary)" : "var(--text-primary)" }}>
                  {member.name}
                  <Show when={member.former}><span style={{ "font-size": "11.5px" }}> · {t("former member")}</span></Show>
                </span>
                <Show when={member.mood}>
                  <span style={{ "font-size": "12px", color: moodColor(member.mood) }}>{t(member.mood!)}{trendMark(member.trend)}</span>
                </Show>
                <span style={{ width: "34px", "text-align": "right", "font-size": "12px", color: "var(--text-meta)" }}>{member.count}</span>
              </button>
            )}
          </For>
        </div>
      </Show>
      <Show when={p.profile.facts.length > 0}>
        <div style={{ padding: "7px 0", "border-bottom": "var(--hairline) solid var(--divider)" }}>
          <div style={{ "font-size": "12.5px", color: "var(--text-secondary)" }}>{t("Facts")}</div>
          <For each={p.profile.facts}>
            {(fact) => (
              <div class="flex items-start" style={{ gap: "6px", "margin-top": "4px" }}>
                <span class="selectable min-w-0 flex-1" style={{ "font-size": "14px", "line-height": "18px", color: "var(--text-primary)" }}>{fact}</span>
                <button type="button" title={t("Forget")} aria-label={`Forget ${fact}`} onClick={() => void commands.zngaFact(p.chat, fact, true)} style={{ color: "var(--text-secondary)" }}>
                  <CloseIcon size={16} />
                </button>
              </div>
            )}
          </For>
        </div>
      </Show>
      <Show when={unknown().length > 0}>
        <button type="button" onClick={() => setMore(!more())} style={{ padding: "8px 0 2px", "font-size": "12.5px", color: "var(--accent-text)" }}>
          {more() ? t("Hide empty fields") : t("Add something Znga should know")}
        </button>
        <Show when={more()}>
          <For each={unknown()}>{(field) => <Field chat={p.chat} field={field} />}</For>
        </Show>
      </Show>
    </section>
  );
}
