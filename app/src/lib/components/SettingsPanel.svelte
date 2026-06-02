<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let mode = $state("byok");
  let provider = $state("openai");
  let model = $state("gpt-4o");
  let apiKey = $state("");
  let defaultPlatform = $state("claude");
  let defaultImagePlatform = $state("midjourney");
  let supermemoryEnabled = $state(false);
  let supermemoryKey = $state("");
  let status = $state<{ type: "success" | "error"; text: string } | null>(null);
  let testingConnection = $state(false);
  let saving = $state(false);

  const providers = [
    {
      id: "openai",
      label: "OpenAI",
      desc: "GPT-4o, o1, o3",
      models: ["gpt-4o", "gpt-4o-mini", "o1", "o3-mini"],
      keyPlaceholder: "sk-proj-...",
      envVar: "OPENAI_API_KEY",
      cliCommand: "proompt config set openai.api_key YOUR_KEY",
      modelHint: "OpenAI models usually start with gpt, chatgpt, o1, o3, or o4.",
    },
    {
      id: "anthropic",
      label: "Anthropic",
      desc: "Claude Sonnet, Haiku",
      models: ["claude-sonnet-4-20250514", "claude-3-5-haiku-20241022"],
      keyPlaceholder: "sk-ant-...",
      envVar: "ANTHROPIC_API_KEY",
      cliCommand: "proompt config set anthropic.api_key YOUR_KEY",
      modelHint: "Anthropic models should start with claude.",
    },
    {
      id: "google",
      label: "Google",
      desc: "Gemini 2.0, 1.5",
      models: ["gemini-2.0-flash", "gemini-2.0-pro", "gemini-1.5-flash"],
      keyPlaceholder: "AI...",
      envVar: "GEMINI_API_KEY",
      cliCommand: "proompt config set google.api_key YOUR_KEY",
      modelHint: "Google models should start with gemini.",
    },
    {
      id: "openrouter",
      label: "OpenRouter",
      desc: "Claude, GPT, Gemini, OSS",
      models: ["openai/gpt-4o-mini", "anthropic/claude-3.5-sonnet", "google/gemini-2.0-flash-001", "meta-llama/llama-3.1-8b-instruct"],
      keyPlaceholder: "sk-or-...",
      envVar: "OPENROUTER_API_KEY",
      cliCommand: "proompt config set openrouter.api_key YOUR_KEY",
      modelHint: "OpenRouter model ids must look like provider/model-id, for example openai/gpt-4o-mini.",
    },
  ];

  let currentProvider = $derived(providers.find((p) => p.id === provider) ?? providers[0]);
  let modelError = $derived(validateModel(provider, model));

  async function loadConfig() {
    try {
      const config = await invoke<any>("get_config");
      mode = config.mode?.toLowerCase() || "byok";
      provider = config.byok?.provider || "openai";
      model = config.byok?.model || "gpt-4o";
      defaultPlatform = config.default_platform?.toLowerCase() || "claude";
      defaultImagePlatform = config.default_image_platform?.toLowerCase() || "midjourney";
      supermemoryEnabled = config.supermemory?.enabled || false;
    } catch (e: any) {
      showStatus("error", `Failed to load: ${e}`);
    }
  }

  $effect(() => { loadConfig(); });

  function selectProvider(providerId: string) {
    provider = providerId;
    const selectedProvider = providers.find((p) => p.id === providerId) ?? providers[0];
    model = selectedProvider.models[0];
  }

  function showStatus(type: "success" | "error", text: string) {
    status = { type, text };
    setTimeout(() => (status = null), 3500);
  }

  async function saveApiKey() {
    if (!apiKey.trim()) return;
    try {
      await invoke("set_api_key", { service: provider, key: apiKey });
      showStatus("success", `${currentProvider.label} key saved to keychain`);
      apiKey = "";
    } catch (e: any) { showStatus("error", `${e}`); }
  }

  async function saveSmKey() {
    if (!supermemoryKey.trim()) return;
    try {
      await invoke("set_api_key", { service: "supermemory", key: supermemoryKey });
      showStatus("success", "SuperMemory key saved");
      supermemoryKey = "";
    } catch (e: any) { showStatus("error", `${e}`); }
  }

  function validateModel(providerId: string, modelId: string) {
    const trimmed = modelId.trim();
    if (!trimmed) return "Model is required";
    if (providerId === "openrouter") {
      const [modelProvider, modelName] = trimmed.split("/", 2);
      if (!modelProvider || !modelName) return "OpenRouter model must use provider/model-id format";
    }
    if (providerId === "anthropic" && !trimmed.toLowerCase().startsWith("claude")) {
      return "Anthropic model must start with claude";
    }
    if (providerId === "google" && !trimmed.toLowerCase().startsWith("gemini")) {
      return "Google model must start with gemini";
    }
    if (providerId === "openai" && !/^(gpt|chatgpt|o1|o3|o4)/i.test(trimmed)) {
      return "OpenAI model must start with gpt, chatgpt, o1, o3, or o4";
    }
    return "";
  }

  async function testConnection() {
    if (modelError) {
      showStatus("error", modelError);
      return;
    }
    testingConnection = true;
    try {
      const result = await invoke<string>("test_api_connection", {
        provider,
        model,
        apiKey: apiKey.trim() || null,
      });
      showStatus("success", result);
    } catch (e: any) {
      showStatus("error", `${e}`);
    } finally { testingConnection = false; }
  }

  async function saveConfig() {
    if (modelError) {
      showStatus("error", modelError);
      return;
    }
    saving = true;
    try {
      await invoke("save_settings", { mode, provider, model, defaultPlatform, defaultImagePlatform, supermemoryEnabled });
      showStatus("success", "Settings saved");
    } catch (e: any) { showStatus("error", `${e}`); }
    finally { saving = false; }
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Settings</h1>
    <p class="subtitle">Configure providers, keys, and preferences</p>
  </div>

  <!-- Provider -->
  <section class="section">
    <div class="section-label">Provider</div>
    <div class="provider-grid">
      {#each providers as p}
        <button
          class="provider-card"
          class:active={provider === p.id}
          onclick={() => selectProvider(p.id)}
        >
          <span class="provider-name">{p.label}</span>
          <span class="provider-desc">{p.desc}</span>
        </button>
      {/each}
    </div>
  </section>

  <!-- Model -->
  <section class="section">
    <div class="section-label">Model</div>
    <div class="select-wrap">
      <select bind:value={model}>
        {#each currentProvider.models as m}
          <option value={m}>{m}</option>
        {/each}
      </select>
    </div>
    <input
      class="model-input"
      type="text"
      bind:value={model}
      placeholder={provider === "openrouter" ? "provider/model-id" : "Custom model id"}
    />
    <p class="hint">{currentProvider.modelHint}</p>
    {#if modelError}
      <p class="field-error">{modelError}</p>
    {/if}
  </section>

  <!-- API Key -->
  <section class="section">
    <div class="section-label">API Key</div>
    <div class="key-row">
      <input
        type="password"
        bind:value={apiKey}
        placeholder={currentProvider.keyPlaceholder}
      />
      <button class="btn-secondary" onclick={saveApiKey} disabled={!apiKey.trim()}>Save</button>
      <button class="btn-secondary" onclick={testConnection} disabled={testingConnection || !!modelError}>
        {testingConnection ? "..." : "Test"}
      </button>
    </div>
    <div class="setup-guide">
      <div class="setup-row">
        <span>CLI</span>
        <code>{currentProvider.cliCommand}</code>
      </div>
      <div class="setup-row">
        <span>Env</span>
        <code>export {currentProvider.envVar}=...</code>
      </div>
    </div>
    <p class="hint">Stored in your OS keychain. Paste a key above to test it before saving.</p>
  </section>

  <!-- Default Platform -->
  <section class="section">
    <div class="section-label">Default platform</div>
    <div class="select-wrap">
      <select bind:value={defaultPlatform}>
        <option value="claude">Claude</option>
        <option value="openai">OpenAI</option>
        <option value="gemini">Gemini</option>
        <option value="generic">Generic</option>
      </select>
    </div>
  </section>

  <!-- Default Image Platform -->
  <section class="section">
    <div class="section-label">Default image platform</div>
    <div class="select-wrap">
      <select bind:value={defaultImagePlatform}>
        <option value="midjourney">Midjourney</option>
        <option value="dalle">DALL-E</option>
        <option value="stablediffusion">Stable Diffusion</option>
        <option value="generic">Generic</option>
      </select>
    </div>
  </section>

  <!-- SuperMemory -->
  <section class="section">
    <div class="section-row">
      <div>
        <div class="section-label" style="margin-bottom: 2px">SuperMemory</div>
        <p class="hint" style="margin: 0">Context retrieval for personalized prompts</p>
      </div>
      <label class="toggle">
        <div class="toggle-track" class:on={supermemoryEnabled}>
          <input type="checkbox" bind:checked={supermemoryEnabled} />
          <div class="toggle-thumb"></div>
        </div>
      </label>
    </div>
    {#if supermemoryEnabled}
      <div class="key-row" style="margin-top: 10px">
        <input type="password" bind:value={supermemoryKey} placeholder="sm_..." />
        <button class="btn-secondary" onclick={saveSmKey} disabled={!supermemoryKey.trim()}>Save</button>
      </div>
    {/if}
  </section>

  <button class="btn-primary full-width" onclick={saveConfig} disabled={saving || !!modelError}>
    {saving ? "Saving..." : "Save settings"}
  </button>

  {#if status}
    <div class="toast" class:success={status.type === "success"} class:error={status.type === "error"}>
      {#if status.type === "success"}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
      {/if}
      {status.text}
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 520px;
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  h1 {
    font-size: 22px;
    font-weight: 650;
    color: #fafafa;
    letter-spacing: -0.5px;
  }

  .subtitle {
    font-size: 13px;
    color: #52525b;
    font-weight: 450;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-size: 12px;
    font-weight: 600;
    color: #a1a1aa;
  }

  .section-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  /* ── Provider cards ───────────────── */

  .provider-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .provider-card {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 12px;
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s ease;
  }

  .provider-card:hover {
    border-color: #27272a;
    background: #18181b;
  }

  .provider-card.active {
    border-color: rgba(16, 185, 129, 0.4);
    background: rgba(16, 185, 129, 0.05);
  }

  .provider-name {
    font-size: 13px;
    font-weight: 600;
    color: #e4e4e7;
  }

  .provider-card.active .provider-name {
    color: #34d399;
  }

  .provider-desc {
    font-size: 11px;
    color: #52525b;
  }

  /* ── Form elements ────────────────── */

  .select-wrap select,
  .model-input,
  .key-row input {
    padding: 8px 12px;
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    color: #e4e4e7;
    border-radius: 8px;
    font-size: 13px;
    font-family: inherit;
    transition: border-color 0.12s ease;
    width: 100%;
  }

  .select-wrap select:focus,
  .model-input:focus,
  .key-row input:focus {
    outline: none;
    border-color: #10b981;
    box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.08);
  }

  .model-input {
    margin-top: 6px;
  }

  .key-row {
    display: flex;
    gap: 6px;
  }

  .key-row input {
    flex: 1;
    width: auto;
  }

  .hint {
    font-size: 11px;
    color: #3f3f46;
    margin: 0;
  }

  .field-error {
    font-size: 11px;
    color: #f87171;
    margin: 0;
  }

  .setup-guide {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    border-radius: 8px;
  }

  .setup-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .setup-row span {
    width: 28px;
    flex-shrink: 0;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #52525b;
  }

  code {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 3px 6px;
    background: #18181b;
    border: 1px solid #27272a;
    border-radius: 5px;
    color: #a1a1aa;
    font-family: "SF Mono", "Fira Code", ui-monospace, monospace;
    font-size: 10.5px;
  }

  /* ── Buttons ──────────────────────── */

  .btn-primary {
    padding: 9px 20px;
    background: #10b981;
    color: #022c22;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    transition: all 0.12s ease;
  }

  .btn-primary:hover:not(:disabled) { background: #34d399; }
  .btn-primary:disabled { opacity: 0.35; cursor: not-allowed; }
  .full-width { width: 100%; }

  .btn-secondary {
    padding: 8px 14px;
    background: #18181b;
    color: #a1a1aa;
    border: 1px solid #27272a;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 550;
    white-space: nowrap;
    transition: all 0.12s ease;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #27272a;
    color: #e4e4e7;
  }

  .btn-secondary:disabled { opacity: 0.35; cursor: not-allowed; }

  /* ── Toggle ───────────────────────── */

  .toggle {
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .toggle-track {
    position: relative;
    width: 36px;
    height: 20px;
    background: #27272a;
    border-radius: 99px;
    transition: background 0.15s ease;
  }

  .toggle-track input {
    position: absolute;
    opacity: 0;
    width: 100%;
    height: 100%;
    cursor: pointer;
    z-index: 1;
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    background: #52525b;
    border-radius: 99px;
    transition: all 0.15s ease;
  }

  .toggle-track.on { background: #059669; }
  .toggle-track.on .toggle-thumb { left: 18px; background: #fff; }

  /* ── Toast ────────────────────────── */

  .toast {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: 10px;
    font-size: 12.5px;
    font-weight: 500;
    animation: slideUp 0.2s ease;
  }

  .toast.success {
    background: rgba(16, 185, 129, 0.08);
    border: 1px solid rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .toast.error {
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.2);
    color: #fca5a5;
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
