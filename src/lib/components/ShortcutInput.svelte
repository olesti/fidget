<script lang="ts">
  import { api } from "../api";

  let { id, value, label, t }: { id: string; value: string; label: string; t: (k: string) => string } = $props();

  // svelte-ignore state_referenced_locally
  let current = $state(value);
  let error = $state("");

  /** KeyboardEvent -> Tauri accelerator, e.g. "CmdOrControl+Alt+J". */
  function accel(e: KeyboardEvent): string | null {
    if (["Control", "Meta", "Alt", "Shift"].includes(e.key)) return null;
    const key = e.code.replace(/^(Key|Digit)/, "").replace(/^Arrow/, "");
    const mods = [(e.metaKey || e.ctrlKey) && "CmdOrControl", e.altKey && "Alt", e.shiftKey && "Shift"].filter(Boolean);
    return mods.length ? [...mods, key].join("+") : null; // a global shortcut needs a modifier
  }

  async function set(a: string) {
    try {
      await api.setShortcut(id, a);
      current = a;
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    // Tab / Escape without a modifier must leave the field (no keyboard trap).
    if ((e.key === "Tab" || e.key === "Escape") && !(e.metaKey || e.ctrlKey || e.altKey)) return;
    e.preventDefault();
    if (e.key === "Backspace") return set("");
    const a = accel(e);
    if (a) set(a);
  }
</script>

<div class="flex flex-col items-end">
  <input
    readonly
    aria-label={label}
    class="w-48 cursor-pointer rounded border border-neutral-300 bg-transparent px-2 py-1 text-center dark:border-neutral-600"
    value={current ? current.replace("CmdOrControl", "⌘/Ctrl") : t("ui.shortcut_none")}
    {onkeydown}
  />
  {#if error}<span role="alert" class="text-xs text-red-600 dark:text-red-400">{error}</span>{/if}
</div>
