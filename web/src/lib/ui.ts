// Interface-only state the core does not keep: the open confirmation.
import { createSignal } from "solid-js";
import type { Ask } from "../features/chats/chatMenu";

export const [asking, ask] = createSignal<Ask | null>(null);
