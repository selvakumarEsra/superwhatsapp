// Settings › Znga (REQ-ZCHAT-007): turning Znga on, its engine and key,
// which chats it may read, what it does on its own, what it remembers,
// open promises, and the Sentinel's log. Every change takes effect at once.
import { createSignal, For, type JSX, Show } from "solid-js";
import { commands, state, type ZngaSettings as Z } from "../../lib/bridge";
import { t } from "../../lib/i18n";

type Key = keyof Z;
const z = () => state.settings.znga;
const set = (key: Key, value: unknown) => void commands.zngaSet(key, value);

const field = {
  height: "26px",
  padding: "0 8px",
  "border-radius": "7px",
  background: "var(--search-bg)",
  color: "var(--text-primary)",
  "font-size": "var(--fs-row-text)",
  border: "none",
} as const;

function Heading(p: { children: JSX.Element }) {
  return (
    <h3 style={{ margin: "22px 0 4px", "font-size": "var(--fs-meta)", "font-weight": 600, "text-transform": "uppercase", "letter-spacing": "0.03em", color: "var(--text-secondary)" }}>
      {p.children}
    </h3>
  );
}

function Row(p: { title: string; description?: string; children?: JSX.Element }) {
  return (
    <div class="flex items-center justify-between" style={{ "min-height": "46px", padding: "8px 0", gap: "16px", "border-bottom": "var(--hairline) solid var(--divider)" }}>
      <div class="min-w-0">
        <div style={{ "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}>{t(p.title)}</div>
        <Show when={p.description}>
          <div style={{ "margin-top": "2px", "font-size": "var(--fs-meta)", "line-height": "14px", color: "var(--text-secondary)" }}>{t(p.description!)}</div>
        </Show>
      </div>
      <div class="flex shrink-0 items-center" style={{ gap: "6px" }}>{p.children}</div>
    </div>
  );
}

function Switch(p: { label: string; on: boolean; onChange: (on: boolean) => void }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={p.on}
      aria-label={p.label}
      onClick={() => p.onChange(!p.on)}
      class="relative inline-block rounded-full"
      style={{ width: "30px", height: "18px", background: p.on ? "var(--accent)" : "var(--switch-off)", transition: "background .15s" }}
    >
      <span class="absolute rounded-full" style={{ top: "2px", left: p.on ? "14px" : "2px", width: "14px", height: "14px", background: "#fff", transition: "left .15s" }} />
    </button>
  );
}

function Toggle(p: { title: string; description?: string; key: Key }) {
  return (
    <Row title={p.title} description={p.description}>
      <Switch label={p.title} on={z()[p.key] === true} onChange={(on) => set(p.key, on)} />
    </Row>
  );
}

function Text(p: { title: string; description?: string; key: Key; placeholder?: string; width?: string }) {
  return (
    <Row title={p.title} description={p.description}>
      <input
        aria-label={p.title}
        value={String(z()[p.key] ?? "")}
        placeholder={p.placeholder}
        onBlur={(e) => e.currentTarget.value !== z()[p.key] && set(p.key, e.currentTarget.value)}
        onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
        class="selectable outline-none"
        style={{ ...field, width: p.width ?? "220px" }}
      />
    </Row>
  );
}

function Time(p: { label: string; key: Key }) {
  return (
    <input
      type="time"
      aria-label={p.label}
      value={String(z()[p.key])}
      onChange={(e) => set(p.key, e.currentTarget.value)}
      style={{ ...field, width: "96px" }}
    />
  );
}

function Count(p: { title: string; description?: string; key: Key; min: number; max: number }) {
  return (
    <Row title={p.title} description={p.description}>
      <input
        type="number"
        aria-label={p.title}
        min={p.min}
        max={p.max}
        value={String(z()[p.key])}
        onChange={(e) => set(p.key, Math.round(+e.currentTarget.value))}
        style={{ ...field, width: "72px" }}
      />
    </Row>
  );
}

function Button(p: { label: string; danger?: boolean; onClick: () => void; disabled?: boolean }) {
  return (
    <button
      type="button"
      disabled={p.disabled}
      onClick={p.onClick}
      class="rounded-full"
      style={{ height: "26px", padding: "0 12px", "font-size": "var(--fs-row-text)", background: "var(--round-button-bg)", color: p.danger ? "var(--danger)" : "var(--text-primary)", opacity: p.disabled ? 0.4 : 1 }}
    >
      {t(p.label)}
    </button>
  );
}

/** A chat picker over the account's chats, for Twin and Excluded. */
function ChatPicker(p: { list: "twin" | "excluded"; chosen: [string, string][] }) {
  const [query, setQuery] = createSignal("");
  const matches = () => {
    const q = query().trim().toLowerCase();
    if (!q) return [];
    return state.chats.filter((c) => c.title.toLowerCase().includes(q) && !p.chosen.some(([id]) => id === c.id)).slice(0, 6);
  };
  return (
    <div style={{ padding: "8px 0", "border-bottom": "var(--hairline) solid var(--divider)" }}>
      <div class="flex flex-wrap" style={{ gap: "6px" }}>
        <For each={p.chosen}>
          {([id, title]) => (
            <span class="flex items-center rounded-full" style={{ height: "24px", padding: "0 4px 0 10px", gap: "4px", background: "var(--search-bg)", "font-size": "var(--fs-meta)", color: "var(--text-primary)" }}>
              {title}
              <button type="button" aria-label={`Remove ${title}`} onClick={() => void commands.zngaChatList(p.list, id, false)} style={{ width: "18px", color: "var(--text-secondary)" }}>×</button>
            </span>
          )}
        </For>
        <Show when={p.chosen.length === 0}>
          <span style={{ "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>{t("None yet.")}</span>
        </Show>
      </div>
      <input
        aria-label={p.list === "twin" ? "Add a Twin chat" : "Add an excluded chat"}
        placeholder={t("Add a chat…")}
        value={query()}
        onInput={(e) => setQuery(e.currentTarget.value)}
        class="selectable outline-none"
        style={{ ...field, width: "240px", "margin-top": "8px" }}
      />
      <For each={matches()}>
        {(chat) => (
          <button
            type="button"
            onClick={() => {
              void commands.zngaChatList(p.list, chat.id, true);
              setQuery("");
            }}
            class="chat-row block w-full text-left"
            style={{ padding: "5px 8px", "border-radius": "6px", "font-size": "var(--fs-row-text)", color: "var(--text-primary)" }}
          >
            {chat.title}
          </button>
        )}
      </For>
    </div>
  );
}

function when(at: number) {
  return new Date(at * 1000).toLocaleString(undefined, { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
}

export default function ZngaSettings() {
  const [key, setKey] = createSignal("");
  const [factChat, setFactChat] = createSignal("");
  const [factText, setFactText] = createSignal("");
  return (
    <>
      <Row title="Znga" description="Your AI companion. It reads only the chats it may, drafts but never sends, and keeps its chat on this computer.">
        <Switch label="Znga" on={z().enabled} onChange={(on) => set("enabled", on)} />
      </Row>
      <Show when={z().status}>
        {(status) => (
          <div style={{ padding: "8px 0", "font-size": "var(--fs-meta)", color: status()[1] ? "var(--danger)" : "var(--text-secondary)" }}>{status()[0]}</div>
        )}
      </Show>

      <Heading>{t("AI engine")}</Heading>
      <Row title="Engine">
        <select aria-label="Engine" value={z().engine} onChange={(e) => set("engine", e.currentTarget.value)} style={field}>
          <For each={z().engines}>{([id, label]) => <option value={id}>{label}</option>}</For>
        </select>
      </Row>
      <Text title="Model" key="model" placeholder={z().defaultModel} />
      <Text title={z().engine === "claude-code" ? "Program" : "Address"} key="endpoint" placeholder={z().defaultEndpoint} />
      <Show when={z().engine === "local"}>
        <div style={{ padding: "6px 0", "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>
          {t("Runs on this computer: install Ollama (ollama.com), then run `ollama pull llama3.2`. Nothing leaves your machine.")}
        </div>
      </Show>
      <Show when={z().needsKey}>
        <Row title="API key" description={z().keySource ? `Using a key ${z().keySource}.` : "No key yet."}>
          <input
            type="password"
            aria-label="API key"
            placeholder={t("Paste your API key")}
            value={key()}
            onInput={(e) => setKey(e.currentTarget.value)}
            class="selectable outline-none"
            style={{ ...field, width: "180px" }}
          />
          <Button label="Save" disabled={!key().trim()} onClick={() => { void commands.zngaKey(key()); setKey(""); }} />
          <Show when={z().keySource}>
            <Button label="Forget" danger onClick={() => void commands.zngaKey(null)} />
          </Show>
        </Row>
      </Show>
      <Show when={z().cloud}>
        <Toggle title="Let this engine read my messages" description="With this engine, recent messages from chats you have not excluded are sent to the provider. Locked chats are never sent." key="cloudConsent" />
      </Show>
      <Row title="Test connection">
        <Button label="Test" onClick={() => void commands.zngaTest()} />
      </Row>

      <Heading>{t("Chats Znga reads")}</Heading>
      <Row title="Twin chats" description="Your close ones: Znga prepares replies when they write and suggests check-ins." />
      <ChatPicker list="twin" chosen={z().twin} />
      <Row title="Excluded chats" description="Znga never reads these, like locked chats." />
      <ChatPicker list="excluded" chosen={z().excluded} />
      <Toggle title="Include group chats" key="includeGroups" />

      <Heading>{t("Speaking first")}</Heading>
      <Toggle title="Needs you" description="A message in the Znga chat when a new message needs you." key="alerts" />
      <Toggle title="Welcome back" description="A briefing when you return after being away." key="briefings" />
      <Show when={z().briefings}>
        <Count title="Messages before a briefing" key="briefingMinMessages" min={1} max={50} />
        <Count title="Minutes between briefings" key="briefingIntervalMinutes" min={5} max={240} />
      </Show>
      <Row title="Digests" description="A morning and an evening digest of your chats.">
        <Show when={z().digests}>
          <Time label="Morning digest" key="digestMorning" />
          <Time label="Evening digest" key="digestEvening" />
        </Show>
        <Switch label="Digests" on={z().digests} onChange={(on) => set("digests", on)} />
      </Row>
      <Toggle title="Reminders" description="Promises coming due, plans coming up, and close friends drifting away." key="reminders" />
      <Count title="Days before a check-in" key="checkinDays" min={1} max={90} />
      <Row title="Quiet hours" description="Only Needs you is posted; the rest waits until quiet hours end.">
        <Show when={z().quietHours}>
          <Time label="Quiet from" key="quietFrom" />
          <Time label="Quiet until" key="quietTo" />
        </Show>
        <Switch label="Quiet hours" on={z().quietHours} onChange={(on) => set("quietHours", on)} />
      </Row>
      <Toggle title="Copy briefings to Message yourself" description="At most six a day, to read on your phone." key="selfNotes" />
      <Toggle title="Answer in Message yourself" description="Questions you send yourself are answered in that chat. It never writes to anyone else." key="selfChat" />

      <Heading>{t("How you write")}</Heading>
      <Text title="Your name" key="yourName" placeholder="From your profile" />
      <Text title="Tone" key="tone" placeholder="warm and casual" />
      <Text title="How you write" description="Words Znga should know about your style." key="voiceNotes" placeholder="I say haha a lot" width="280px" />
      <Text title="Transcription address" description="For voice notes. Empty uses a local server on this computer." key="voiceEndpoint" placeholder="http://localhost:8000/v1" />
      <Text title="Transcription model" key="voiceModel" placeholder="whisper-1" />

      <Heading>{t("Memory")}</Heading>
      <Toggle title="Remember facts about people" description="From your Twin chats; used in drafts." key="memory" />
      <Toggle title="Track promises" key="promises" />
      <For each={z().facts}>
        {(fact) => (
          <Row title={fact.text} description={fact.chatTitle}>
            <Button label="Forget" danger onClick={() => void commands.zngaFact(fact.chat, fact.text, true)} />
          </Row>
        )}
      </For>
      <div class="flex items-center" style={{ gap: "6px", padding: "8px 0", "border-bottom": "var(--hairline) solid var(--divider)" }}>
        <select aria-label="Fact about" value={factChat()} onChange={(e) => setFactChat(e.currentTarget.value)} style={{ ...field, width: "160px" }}>
          <option value="">{t("About…")}</option>
          <For each={state.chats.slice(0, 200)}>{(c) => <option value={c.id}>{c.title}</option>}</For>
        </select>
        <input
          aria-label="New fact"
          placeholder={t("Add something Znga should remember")}
          value={factText()}
          onInput={(e) => setFactText(e.currentTarget.value)}
          class="selectable min-w-0 flex-1 outline-none"
          style={field}
        />
        <Button
          label="Add"
          disabled={!factChat() || !factText().trim()}
          onClick={() => {
            void commands.zngaFact(factChat(), factText(), false);
            setFactText("");
          }}
        />
      </div>

      <Heading>{t("Open promises")}</Heading>
      <Show when={z().openPromises.length === 0}>
        <div style={{ padding: "8px 0", "font-size": "var(--fs-meta)", color: "var(--text-secondary)" }}>{t("None.")}</div>
      </Show>
      <For each={z().openPromises}>
        {(p) => (
          <Row title={p.text} description={[p.chatTitle, p.due ? when(p.due) : ""].filter(Boolean).join(" · ")}>
            <Button label="Done" onClick={() => void commands.zngaPromise(p.chat, p.text, true)} />
            <Button label="Not a promise" onClick={() => void commands.zngaPromise(p.chat, p.text, false)} />
          </Row>
        )}
      </For>

      <Heading>{t("Sentinel log")}</Heading>
      <div style={{ "font-size": "var(--fs-meta)", color: "var(--text-secondary)", padding: "4px 0 6px" }}>
        {t("The last 100 things Znga did or was refused. Message text is never logged.")}
      </div>
      <For each={z().audit}>
        {(entry) => (
          <div class="flex items-baseline" style={{ gap: "10px", padding: "4px 0", "border-bottom": "var(--hairline) solid var(--divider)", "font-size": "var(--fs-meta)" }}>
            <span style={{ width: "110px", color: "var(--text-secondary)" }}>{when(entry.at)}</span>
            <span style={{ width: "16px", color: entry.allowed ? "var(--accent-text)" : "var(--danger)" }}>{entry.allowed ? "✓" : "✕"}</span>
            <span class="min-w-0 flex-1" style={{ color: "var(--text-primary)" }}>{entry.action}</span>
            <span style={{ color: "var(--text-secondary)" }}>{entry.origin} · {entry.reason}</span>
          </div>
        )}
      </For>
    </>
  );
}
