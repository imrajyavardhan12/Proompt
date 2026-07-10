<script lang="ts">
  import EnhancePanel from "./lib/components/EnhancePanel.svelte";
  import HistoryPanel from "./lib/components/HistoryPanel.svelte";
  import TemplatesPanel from "./lib/components/TemplatesPanel.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";

  interface EnhanceDraft {
    id: string;
    prompt: string;
    platform: string;
    mode: "text" | "image";
  }

  type View = "enhance" | "history" | "templates" | "settings";
  type SettingsSection = "provider" | "troubleshoot";

  let activeView = $state<View>("enhance");
  let settingsProviderHint = $state<string | null>(null);
  let settingsSectionHint = $state<SettingsSection>("provider");
  let enhanceDraft = $state<EnhanceDraft | null>(null);

  function openEnhance() {
    activeView = "enhance";
  }

  function openSettings(providerHint?: string, sectionHint: SettingsSection = "provider") {
    settingsProviderHint = providerHint ?? null;
    settingsSectionHint = sectionHint;
    activeView = "settings";
  }

  function reuseHistoryDraft(draft: EnhanceDraft) {
    enhanceDraft = draft;
    activeView = "enhance";
  }
</script>

<div class="app-shell">
  <header class="topbar">
    <button class="brand" onclick={openEnhance} aria-label="Open prompt workspace">
      <span class="brand-mark">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 3L20 7.5V16.5L12 21L4 16.5V7.5L12 3Z"/>
          <path d="M12 12L20 7.5"/>
          <path d="M12 12V21"/>
          <path d="M12 12L4 7.5"/>
        </svg>
      </span>
      <span>Proompt</span>
    </button>

    <div class="workspace-label">
      <span class="workspace-dot"></span>
      Quick Enhance
    </div>

    <nav class="top-actions" aria-label="Primary navigation">
      {#if activeView !== "enhance"}
        <button class="nav-button" onclick={openEnhance}>Enhance</button>
      {/if}
      <button
        class="nav-button"
        class:active={activeView === "history"}
        onclick={() => (activeView = "history")}
      >History</button>
      <button
        class="settings-button"
        class:active={activeView === "settings"}
        onclick={() => openSettings()}
        aria-label="Settings"
        title="Settings"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/>
          <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
        </svg>
      </button>
    </nav>
  </header>

  <main class="main" class:focus={activeView === "enhance"}>
    <div class="main-inner">
      {#if activeView === "enhance"}
        <EnhancePanel
          onOpenSettings={openSettings}
          onOpenHistory={() => (activeView = "history")}
          onOpenTemplates={() => (activeView = "templates")}
          draft={enhanceDraft}
        />
      {:else if activeView === "history"}
        <HistoryPanel onReuse={reuseHistoryDraft} />
      {:else if activeView === "templates"}
        <TemplatesPanel />
      {:else}
        <SettingsPanel initialProvider={settingsProviderHint} initialSection={settingsSectionHint} />
      {/if}
    </div>
  </main>
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family:
      "SF Pro Text",
      -apple-system,
      BlinkMacSystemFont,
      "Inter",
      "Segoe UI",
      sans-serif;
    background: #0e0f11;
    color: #f5f5f5;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    overflow: hidden;
  }

  :global(button),
  :global(input),
  :global(select),
  :global(textarea) {
    font: inherit;
  }

  :global(::-webkit-scrollbar) { width: 5px; }
  :global(::-webkit-scrollbar-track) { background: transparent; }
  :global(::-webkit-scrollbar-thumb) { background: #3a3a3a; border-radius: 99px; }
  :global(::-webkit-scrollbar-thumb:hover) { background: #5f5f5f; }

  .app-shell {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background: #0e0f11;
  }

  .topbar {
    z-index: 2;
    height: 62px;
    min-height: 62px;
    padding: 0 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.045);
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    -webkit-app-region: drag;
    user-select: none;
  }

  .brand,
  .nav-button,
  .settings-button {
    -webkit-app-region: no-drag;
  }

  .brand {
    width: fit-content;
    display: flex;
    align-items: center;
    gap: 9px;
    border: 0;
    background: transparent;
    color: #f0efec;
    cursor: pointer;
    font-size: 14px;
    font-weight: 650;
    letter-spacing: -0.3px;
  }

  .brand-mark {
    width: 29px;
    height: 29px;
    border: 1px solid #393b40;
    border-radius: 8px;
    display: grid;
    place-items: center;
    background: #1d1f23;
    color: #e7e5e4;
  }

  .workspace-label {
    display: flex;
    align-items: center;
    gap: 7px;
    color: #777a80;
    font-size: 10.5px;
    font-weight: 550;
  }

  .workspace-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #62656b;
    box-shadow: 0 0 0 3px rgba(98, 101, 107, 0.08);
  }

  .top-actions {
    justify-self: end;
    display: flex;
    align-items: center;
    gap: 4px;
    -webkit-app-region: no-drag;
  }

  .nav-button,
  .settings-button {
    border: 1px solid transparent;
    border-radius: 7px;
    background: transparent;
    color: #85888e;
    cursor: pointer;
    transition: 0.12s ease;
  }

  .nav-button {
    padding: 7px 9px;
    font-size: 11.5px;
    font-weight: 550;
  }

  .settings-button {
    width: 36px;
    height: 34px;
    display: grid;
    place-items: center;
  }

  .nav-button:hover,
  .settings-button:hover,
  .nav-button.active,
  .settings-button.active {
    border-color: #2d2f34;
    background: #1b1d21;
    color: #ecebea;
  }

  .main {
    flex: 1;
    overflow-y: auto;
    background: #111214;
  }

  .main.focus {
    background: radial-gradient(circle at 50% 20%, #1a1c20 0, #111214 42%, #0e0f11 100%);
  }

  .main-inner {
    width: min(860px, calc(100vw - 48px));
    margin: 0 auto;
    padding: 34px 0 48px;
  }

  .main.focus .main-inner {
    width: min(700px, calc(100vw - 48px));
    padding-top: 48px;
  }

  @media (max-width: 620px) {
    .topbar { padding: 0 14px; }
    .workspace-label { display: none; }
    .topbar { grid-template-columns: 1fr auto; }
    .main-inner,
    .main.focus .main-inner { width: min(100% - 28px, 700px); }
  }
</style>
