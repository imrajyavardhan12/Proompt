<script lang="ts">
  import EnhancePanel from "./lib/components/EnhancePanel.svelte";
  import TemplatesPanel from "./lib/components/TemplatesPanel.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";

  let activeTab = $state<"enhance" | "templates" | "settings">("enhance");
  let settingsProviderHint = $state<string | null>(null);

  function openSettings(providerHint?: string) {
    settingsProviderHint = providerHint ?? null;
    activeTab = "settings";
  }
</script>

<div class="app-shell">
  <aside class="sidebar">
    <div class="sidebar-top">
      <div class="brand">
        <div class="brand-mark">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 3L20 7.5V16.5L12 21L4 16.5V7.5L12 3Z"/>
            <path d="M12 12L20 7.5"/>
            <path d="M12 12V21"/>
            <path d="M12 12L4 7.5"/>
          </svg>
        </div>
        <span class="brand-text">Proompt</span>
      </div>

      <nav class="nav">
        <button
          class="nav-item"
          class:active={activeTab === "enhance"}
          onclick={() => (activeTab = "enhance")}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
          </svg>
          <span>Enhance</span>
        </button>
        <button
          class="nav-item"
          class:active={activeTab === "templates"}
          onclick={() => (activeTab = "templates")}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7" rx="1"/>
            <rect x="14" y="3" width="7" height="7" rx="1"/>
            <rect x="3" y="14" width="7" height="7" rx="1"/>
            <rect x="14" y="14" width="7" height="7" rx="1"/>
          </svg>
          <span>Templates</span>
        </button>
        <button
          class="nav-item"
          class:active={activeTab === "settings"}
          onclick={() => openSettings()}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"/>
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
          </svg>
          <span>Settings</span>
        </button>
      </nav>
    </div>

    <div class="sidebar-bottom">
      <div class="version">v0.1.1</div>
    </div>
  </aside>

  <main class="main">
    <div class="main-inner">
      {#if activeTab === "enhance"}
        <EnhancePanel onOpenSettings={openSettings} />
      {:else if activeTab === "templates"}
        <TemplatesPanel />
      {:else if activeTab === "settings"}
        <SettingsPanel initialProvider={settingsProviderHint} />
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
    background: #09090b;
    color: #fafafa;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    overflow: hidden;
  }

  :global(::-webkit-scrollbar) {
    width: 5px;
  }
  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(::-webkit-scrollbar-thumb) {
    background: #27272a;
    border-radius: 99px;
  }
  :global(::-webkit-scrollbar-thumb:hover) {
    background: #3f3f46;
  }

  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
  }

  .sidebar {
    width: 200px;
    min-width: 200px;
    background: #09090b;
    border-right: 1px solid #1a1a1e;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 16px 10px;
    -webkit-app-region: drag;
    user-select: none;
  }

  .sidebar-top {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 2px 6px;
  }

  .brand-mark {
    width: 28px;
    height: 28px;
    background: linear-gradient(145deg, #10b981, #059669);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    flex-shrink: 0;
  }

  .brand-text {
    font-weight: 650;
    font-size: 15px;
    color: #fafafa;
    letter-spacing: -0.4px;
  }

  .nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border: none;
    background: transparent;
    color: #71717a;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    border-radius: 8px;
    transition: all 0.12s ease;
    text-align: left;
  }

  .nav-item:hover {
    color: #a1a1aa;
    background: #18181b;
  }

  .nav-item.active {
    color: #fafafa;
    background: #18181b;
  }

  .nav-item.active svg {
    color: #10b981;
  }

  .sidebar-bottom {
    padding: 0 6px;
  }

  .version {
    font-size: 11px;
    color: #3f3f46;
    font-weight: 500;
  }

  .main {
    flex: 1;
    overflow-y: auto;
    background: #09090b;
  }

  .main-inner {
    max-width: 760px;
    margin: 0 auto;
    padding: 32px 28px;
  }
</style>
