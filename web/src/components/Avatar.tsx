import { createSignal, Show } from "solid-js";
import { DefaultAvatar } from "./Icons";

/** A round picture, with the silhouette until (or unless) it loads. */
export default function Avatar(p: { src: string | null; size: number; group?: boolean; seed?: string }) {
  const [failed, setFailed] = createSignal(false);
  return (
    <div class="shrink-0 overflow-hidden rounded-full" style={{ width: `${p.size}px`, height: `${p.size}px` }}>
      <Show when={p.src && !failed()} fallback={<DefaultAvatar size={p.size} group={p.group} seed={p.seed} />}>
        <img src={p.src!} width={p.size} height={p.size} class="h-full w-full object-cover" onError={() => setFailed(true)} alt="" />
      </Show>
    </div>
  );
}
