<script lang="ts">
  import { onMount } from "svelte";
  import { api, type State } from "../api";
  import { applyTheme, savedTheme } from "../theme";

  let st = $state<State | null>(null);
  let selected = $state("");
  let picked = $state<number | null | undefined>(undefined); // chip last clicked, for highlighting
  let until = $state("");

  const t = (k: string) => st?.strings[k] ?? k;
  const feature = $derived(st?.features.find((f) => f.id === selected) ?? st?.features[0]);
  const CHIPS: (number | null)[] = [1800, 3600, 7200, 14400, 28800, null];

  async function load() {
    st = await api.state();
    if (!selected && st.features[0]) selected = st.features[0].id;
  }

  const chipLabel = (s: number | null) =>
    s === null ? "∞" : s < 3600 ? `${s / 60}${t("ui.panel.min")}` : `${s / 3600}${t("ui.panel.hour")}`;

  function fmt(s: number) {
    const h = Math.floor(s / 3600);
    const m = String(Math.floor((s % 3600) / 60)).padStart(h ? 2 : 1, "0");
    const sec = String(s % 60).padStart(2, "0");
    return h ? `${h}:${m}:${sec}` : `${m}:${sec}`;
  }

  async function start(secs: number | null) {
    if (!feature) return;
    picked = secs;
    await api.startTimed(feature.id, secs);
    load();
  }

  function startUntil() {
    const [h, m] = until.split(":").map(Number);
    if (Number.isNaN(h)) return;
    const target = new Date();
    target.setHours(h, m, 0, 0);
    let secs = Math.round((target.getTime() - Date.now()) / 1000);
    if (secs <= 0) secs += 86400; // next occurrence
    start(secs);
  }

  onMount(() => {
    applyTheme(savedTheme());
    load();
    const id = setInterval(() => !document.hidden && load(), 1000);
    return () => clearInterval(id);
  });

  const chip = "rounded-lg px-2 py-2 text-sm font-medium transition-colors";
  const btn = "rounded-lg border border-neutral-300 px-3 py-2 text-sm hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800";
</script>

{#if st && feature}
  <main class="flex h-screen select-none flex-col gap-3 p-4">
    {#if st.features.length > 1}
      <nav class="flex gap-1 rounded-xl bg-neutral-100 p-1 dark:bg-neutral-800">
        {#each st.features as f (f.id)}
          <button
            class="flex-1 rounded-lg px-2 py-1.5 text-sm {feature.id === f.id ? 'bg-white font-semibold shadow dark:bg-neutral-700' : 'text-neutral-500'}"
            onclick={() => (selected = f.id)}
          >
            {f.name}
            {#if f.enabled}<span class="ml-1 inline-block size-1.5 rounded-full bg-green-500"></span>{/if}
          </button>
        {/each}
      </nav>
    {/if}

    <section class="rounded-xl border border-neutral-200 p-3 dark:border-neutral-700">
      <label class="flex items-center justify-between">
        <span class="font-semibold">{feature.name}</span>
        <input
          type="checkbox"
          class="peer sr-only"
          checked={feature.enabled}
          onchange={async (e) => {
            if (e.currentTarget.checked) await api.startTimed(feature.id, picked ?? null);
            else await api.setEnabled(feature.id, false);
            load();
          }}
        />
        <span class="relative h-6 w-11 rounded-full bg-neutral-300 transition-colors after:absolute after:left-0.5 after:top-0.5 after:size-5 after:rounded-full after:bg-white after:transition-transform peer-checked:bg-blue-600 peer-checked:after:translate-x-5 dark:bg-neutral-600"></span>
      </label>
      <p class="mt-1 text-sm text-neutral-500">{feature.status}</p>

      {#if feature.timer}
        <div class="mt-3 flex items-baseline justify-between">
          <span class="text-sm text-neutral-500">{t("ui.panel.remaining")}</span>
          <span class="text-2xl font-semibold tabular-nums">
            {#if !feature.enabled}–{:else if feature.remaining === null}∞{:else}{fmt(feature.remaining)}{/if}
          </span>
        </div>
        <div class="mt-3 grid grid-cols-3 gap-2">
          {#each CHIPS as c}
            <button
              class="{chip} {feature.enabled && picked === c ? 'bg-blue-600 text-white' : 'bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700'}"
              onclick={() => start(c)}>{chipLabel(c)}</button
            >
          {/each}
        </div>
        <div class="mt-2 flex items-center gap-2">
          <span class="text-sm text-neutral-500">{t("ui.panel.until")}</span>
          <input type="time" bind:value={until} class="flex-1 rounded-lg border border-neutral-300 bg-transparent px-2 py-1 dark:border-neutral-600" />
          <button class={btn} disabled={!until} onclick={startUntil}>{t("ui.panel.set")}</button>
        </div>
        <p class="mt-2 text-xs text-neutral-500">{t("ui.panel.hint")}</p>
      {/if}
    </section>

    <section class="flex items-center justify-between gap-2">
      <span class="text-sm text-neutral-500">{t("ui.panel.pause")}</span>
      {#if st.paused}
        <button class="{btn} text-amber-600" onclick={() => api.resumeAll().then(load)}>{t("ui.panel.resume")}</button>
      {:else}
        <div class="flex gap-1">
          <button class={btn} onclick={() => api.pauseAll(900).then(load)}>15{t("ui.panel.min")}</button>
          <button class={btn} onclick={() => api.pauseAll(3600).then(load)}>1{t("ui.panel.hour")}</button>
          <button class={btn} onclick={() => api.pauseAll(null).then(load)}>{t("tray.ptomorrow")}</button>
        </div>
      {/if}
    </section>

    <div class="mt-auto grid grid-cols-2 gap-2">
      <button class={btn} onclick={api.openSettings}>{t("ui.panel.settings")}</button>
      <button class={btn} onclick={api.quit}>{t("ui.panel.quit")}</button>
    </div>
  </main>
{/if}
