<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Permission, type State } from "./lib/api";
  import Panel from "./lib/components/Panel.svelte";
  import { applyTheme as setTheme, savedTheme } from "./lib/theme";
  import FeatureCard from "./lib/components/FeatureCard.svelte";
  import ShortcutInput from "./lib/components/ShortcutInput.svelte";

  const TABS = ["general", "features", "shortcuts", "permissions", "about"] as const;
  let st = $state<State | null>(null);
  let perms = $state<Permission[]>([]);
  let tab = $state<(typeof TABS)[number]>("features");
  let theme = $state(savedTheme());
  const isPanel = (window as unknown as { __view?: string }).__view === "panel";

  // Update check: "" idle, otherwise a status line.
  let update = $state({ status: "", version: "" as string | null, busy: false });

  async function checkUpdate() {
    update = { status: t("ui.update.checking"), version: null, busy: true };
    try {
      const v = await api.checkUpdate();
      update = { status: v ? t("ui.update.available").replace("{}", v) : t("ui.update.uptodate"), version: v, busy: false };
    } catch (e) {
      update = { status: String(e), version: null, busy: false };
    }
  }

  async function installUpdate() {
    update.busy = true;
    update.status = t("ui.update.installing");
    try {
      await api.installUpdate(); // restarts the app on success
    } catch (e) {
      update = { status: String(e), version: null, busy: false };
    }
  }

  /** The tray can ask for a tab (window.__tab); "update" opens About and checks right away. */
  function openRequestedTab() {
    const req = (window as unknown as { __tab?: string }).__tab;
    if (req === "update") {
      tab = "about";
      checkUpdate();
    } else if (req && (TABS as readonly string[]).includes(req)) {
      tab = req as (typeof TABS)[number];
    }
  }

  const t = (k: string) => st?.strings[k] ?? k;

  async function load() {
    [st, perms] = await Promise.all([api.state(), api.permissions()]);
  }

  const applyTheme = () => setTheme(theme);
  $effect(() => {
    localStorage.setItem("theme", theme);
    applyTheme();
  });

  onMount(() => {
    load().then(openRequestedTab);
    window.addEventListener("fidget-tab", openRequestedTab);
    // The tray / a global shortcut may have changed state while this window was in the background.
    window.addEventListener("focus", load);
    matchMedia("(prefers-color-scheme: dark)").addEventListener("change", applyTheme);
    return () => window.removeEventListener("focus", load);
  });

  const row = "flex items-center justify-between gap-4 py-2";
  const select = "rounded border border-neutral-300 bg-transparent px-2 py-1 dark:border-neutral-600";
</script>

{#if isPanel}
  <Panel />
{:else if st}
  <main class="mx-auto max-w-xl p-5">
    <nav class="mb-4 flex gap-1 border-b border-neutral-200 dark:border-neutral-700">
      {#each TABS as name}
        <button
          class="px-3 py-2 {tab === name ? 'border-b-2 border-blue-600 font-semibold' : 'text-neutral-500'}"
          onclick={() => (tab = name)}>{t("ui.tab." + name)}</button
        >
      {/each}
    </nav>

    {#if st.paused}<p class="mb-3 text-amber-600">{t("ui.paused")}</p>{/if}

    {#if tab === "general"}
      <label class={row}>
        {t("ui.language")}
        <select
          class={select}
          value={st.lang}
          onchange={async (e) => {
            await api.setLang(e.currentTarget.value);
            load();
          }}
        >
          <option value="en">English</option>
          <option value="tr">Türkçe</option>
        </select>
      </label>
      <label class={row}>
        {t("ui.theme")}
        <select class={select} bind:value={theme}>
          {#each ["system", "light", "dark"] as o}<option value={o}>{t("ui.theme." + o)}</option>{/each}
        </select>
      </label>
      <label class={row}>
        {t("ui.autostart")}
        <input
          type="checkbox"
          checked={st.autostart}
          onchange={async (e) => {
            await api.setAutostart(e.currentTarget.checked);
            load();
          }}
        />
      </label>
    {:else if tab === "features"}
      <div class="space-y-4">
        {#each st.features as f (f.id)}
          <FeatureCard feature={f} {t} onchange={load} />
        {/each}
      </div>
    {:else if tab === "shortcuts"}
      <p class="mb-2 text-sm text-neutral-500">{t("ui.shortcut_hint")}</p>
      {#each st.features as f (f.id)}
        <div class={row}>
          {f.name}
          <ShortcutInput id={f.id} value={f.shortcut} {t} />
        </div>
      {/each}
    {:else if tab === "permissions"}
      {#each perms as p (p.id)}
        <div class={row}>
          {t("ui.perm." + p.id)}
          {#if p.granted}
            <span class="text-green-600">{t("ui.granted")}</span>
          {:else}
            <button class="rounded bg-blue-600 px-3 py-1 text-white" onclick={() => api.requestPermission(p.id)}>{t("ui.grant")}</button>
          {/if}
        </div>
      {:else}
        <p class="text-neutral-500">{t("ui.no_perms")}</p>
      {/each}
    {:else}
      <h1 class="text-xl font-semibold">Fidget</h1>
      <p class="mt-1 text-neutral-500">{t("ui.version")} {st.version} · {t("ui.license")}</p>
      <div class="mt-4 flex flex-wrap items-center gap-3">
        <button class="rounded bg-blue-600 px-3 py-1 text-white disabled:opacity-50" disabled={update.busy} onclick={checkUpdate}>{t("ui.update.check")}</button>
        {#if update.version}
          <button class="rounded bg-green-600 px-3 py-1 text-white disabled:opacity-50" disabled={update.busy} onclick={installUpdate}>{t("ui.update.install")}</button>
        {/if}
        <span class="text-sm text-neutral-500">{update.status}</span>
      </div>
      <button class="mt-4 rounded border border-neutral-300 px-3 py-1 dark:border-neutral-600" onclick={api.openLogs}>{t("ui.open_logs")}</button>
    {/if}
  </main>
{/if}
