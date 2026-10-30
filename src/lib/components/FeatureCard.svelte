<script lang="ts">
  import { api, type Feature } from "../api";

  let { feature, t, onchange }: { feature: Feature; t: (k: string) => string; onchange: () => void } = $props();

  // Draft values are seeded once; labels/options keep following the props (language switch).
  // svelte-ignore state_referenced_locally
  let values = $state<Record<string, string | number | boolean>>(Object.fromEntries(feature.fields.map((f) => [f.key, f.value])));
  let status = $state("");

  async function save() {
    try {
      await api.setConfig(feature.id, values);
      status = t("ui.saved");
    } catch (e) {
      status = String(e);
    }
    setTimeout(() => (status = ""), 1800);
  }

  const input = "rounded border border-neutral-300 bg-transparent px-2 py-1 dark:border-neutral-600";
</script>

<section class="rounded-lg border border-neutral-200 p-4 dark:border-neutral-700">
  <label class="flex items-center gap-2 text-base font-semibold">
    <input
      type="checkbox"
      checked={feature.enabled}
      onchange={async (e) => {
        await api.setEnabled(feature.id, e.currentTarget.checked);
        onchange();
      }}
    />
    {feature.name}
  </label>

  {#if feature.fields.length}
    <div class="mt-3 space-y-2">
      {#each feature.fields as f (f.key)}
        <label class="flex items-center justify-between gap-4">
          <span>{f.label}</span>
          {#if f.kind === "select"}
            <select class={input} bind:value={values[f.key]}>
              {#each f.options ?? [] as o}
                <option value={o}>{t("opt." + o)}</option>
              {/each}
            </select>
          {:else if f.kind === "bool"}
            <input type="checkbox" bind:checked={values[f.key] as boolean} />
          {:else if f.kind === "time"}
            <input type="time" class={input} bind:value={values[f.key]} />
          {:else}
            <input type="number" min="1" class="{input} w-24" bind:value={values[f.key]} />
          {/if}
        </label>
      {/each}
    </div>
    <div class="mt-3 flex items-center gap-3">
      <button class="rounded bg-blue-600 px-3 py-1 text-white hover:bg-blue-700" onclick={save}>{t("ui.save")}</button>
      <span class="text-sm text-neutral-500">{status}</span>
    </div>
  {/if}
</section>
