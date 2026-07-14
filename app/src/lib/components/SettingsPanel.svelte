<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type Category = "provider" | "quick" | "privacy" | "help";

  let { initialProvider = null, initialSection = "provider" } = $props<{
    initialProvider?: string | null;
    initialSection?: "provider" | "troubleshoot";
  }>();

  interface QuickEnhanceRouteInspection {
    promptPreview?: string | null;
    environment?: any;
    resolution?: { platform: string; source: string; confidence: string; reason: string } | null;
    error?: string | null;
  }

  interface AccessibilityStatus {
    platformSupported: boolean;
    accessibilityTrusted?: boolean | null;
    selectedTextEnabled: boolean;
    diagnosticsPath: string;
    lastCapture?: {
      timestampMs: number;
      outcome: string;
      inputSource?: string | null;
      resolution?: { platform: string; source: string; confidence: string; reason: string } | null;
      delivery?: string | null;
      deliveryNote?: string | null;
      error?: string | null;
    } | null;
  }

  interface QuickEnhanceSelfCheck {
    accessibility: AccessibilityStatus;
    clipboard: { readable: boolean; writable: boolean; restored: boolean; message: string };
    provider: {
      mode: string;
      provider: string;
      model: string;
      api_key_configured: boolean;
      api_key_status: string;
      api_key_error?: string | null;
    };
    route: QuickEnhanceRouteInspection;
  }

  const providers = [
    {
      id: "openai",
      label: "OpenAI",
      models: ["gpt-4o", "gpt-4o-mini", "o1", "o3-mini"],
      keyPlaceholder: "sk-proj-...",
      envVar: "OPENAI_API_KEY",
      cliCommand: "proompt config set openai.api_key YOUR_KEY",
      modelHint: "OpenAI model IDs usually start with gpt, chatgpt, o1, o3, or o4.",
    },
    {
      id: "anthropic",
      label: "Anthropic",
      models: ["claude-sonnet-4-20250514", "claude-3-5-haiku-20241022"],
      keyPlaceholder: "sk-ant-...",
      envVar: "ANTHROPIC_API_KEY",
      cliCommand: "proompt config set anthropic.api_key YOUR_KEY",
      modelHint: "Anthropic model IDs should start with claude.",
    },
    {
      id: "google",
      label: "Google",
      models: ["gemini-2.0-flash", "gemini-2.0-pro", "gemini-1.5-flash"],
      keyPlaceholder: "AI...",
      envVar: "GEMINI_API_KEY",
      cliCommand: "proompt config set google.api_key YOUR_KEY",
      modelHint: "Google model IDs should start with gemini.",
    },
    {
      id: "openrouter",
      label: "OpenRouter",
      models: [
        "openai/gpt-4o-mini",
        "anthropic/claude-3.5-sonnet",
        "google/gemini-2.0-flash-001",
        "meta-llama/llama-3.1-8b-instruct",
      ],
      keyPlaceholder: "sk-or-...",
      envVar: "OPENROUTER_API_KEY",
      cliCommand: "proompt config set openrouter.api_key YOUR_KEY",
      modelHint: "OpenRouter uses provider/model-id, for example openai/gpt-4o-mini.",
    },
  ];

  const categories: { id: Category; label: string }[] = [
    { id: "provider", label: "Provider" },
    { id: "quick", label: "Quick Enhance" },
    { id: "privacy", label: "Privacy" },
    { id: "help", label: "Help" },
  ];

  let category = $state<Category>("provider");
  let mode = $state("byok");
  let provider = $state("openai");
  let model = $state("gpt-4o");
  let apiKey = $state("");
  let defaultPlatform = $state("claude");
  let defaultImagePlatform = $state("midjourney");
  let quickEnhanceHotkey = $state("CmdOrCtrl+Shift+E");
  let autoDetectTarget = $state(true);
  let selectedTextEnabled = $state(true);
  let terminalPlatform = $state("");
  let saveHistoryEnabled = $state(true);
  let supermemoryEnabled = $state(false);
  let supermemoryKey = $state("");
  let status = $state<{ type: "success" | "error"; text: string } | null>(null);
  let saving = $state(false);
  let testingConnection = $state(false);
  let axStatus = $state<AccessibilityStatus | null>(null);
  let axLoading = $state(false);
  let selfCheck = $state<QuickEnhanceSelfCheck | null>(null);
  let selfChecking = $state(false);
  let routeInspection = $state<QuickEnhanceRouteInspection | null>(null);
  let routeInspecting = $state(false);

  let currentProvider = $derived(providers.find((item) => item.id === provider) ?? providers[0]);
  let modelError = $derived(validateModel(provider, model));
  let hotkeyDisplay = $derived(formatHotkey(quickEnhanceHotkey));
  let ready = $derived(mode === "byok" && !modelError);

  const accessibilityResetCommand = "tccutil reset Accessibility com.proompt.desktop";

  $effect(() => {
    category = initialSection === "troubleshoot" ? "help" : "provider";
    loadConfig();
    loadAccessibilityStatus();
  });

  async function loadConfig() {
    try {
      const config = await invoke<any>("get_config");
      mode = config.mode?.toLowerCase() || "byok";
      provider = config.byok?.provider || "openai";
      model = config.byok?.model || "gpt-4o";
      defaultPlatform = config.default_platform?.toLowerCase() || "claude";
      defaultImagePlatform = config.default_image_platform?.toLowerCase() || "midjourney";
      quickEnhanceHotkey = config.hotkeys?.quick_enhance || "CmdOrCtrl+Shift+E";
      autoDetectTarget = config.quick_enhance?.auto_detect_target ?? true;
      selectedTextEnabled = config.quick_enhance?.selected_text_enabled ?? true;
      terminalPlatform = config.quick_enhance?.terminal_platform?.toLowerCase() || "";
      saveHistoryEnabled = config.preferences?.save_history ?? true;
      supermemoryEnabled = config.supermemory?.enabled || false;
      if (initialProvider && providers.some((item) => item.id === initialProvider)) {
        selectProvider(initialProvider);
      }
    } catch (error: any) {
      showStatus("error", `Could not load settings: ${error}`);
    }
  }

  async function loadAccessibilityStatus() {
    axLoading = true;
    try {
      axStatus = await invoke<AccessibilityStatus>("get_accessibility_status");
    } catch {
      axStatus = null;
    } finally {
      axLoading = false;
    }
  }

  function selectProvider(providerId: string) {
    provider = providerId;
    const selected = providers.find((item) => item.id === providerId) ?? providers[0];
    model = selected.models[0];
  }

  function providerChanged(event: Event) {
    selectProvider((event.currentTarget as HTMLSelectElement).value);
  }

  function formatHotkey(hotkey: string) {
    const isMac = typeof navigator !== "undefined" && navigator.platform?.includes("Mac");
    return hotkey
      .replace("CmdOrCtrl", isMac ? "⌘" : "Ctrl")
      .replace("Shift", isMac ? "⇧" : "Shift")
      .replace(/\+/g, isMac ? "" : " + ");
  }

  function validateModel(providerId: string, modelId: string) {
    const value = modelId.trim();
    if (!value) return "Choose a model.";
    if (providerId === "openrouter" && !/^.+\/.+$/.test(value)) return "Use provider/model-id format.";
    if (providerId === "anthropic" && !value.toLowerCase().startsWith("claude")) return "Anthropic models start with claude.";
    if (providerId === "google" && !value.toLowerCase().startsWith("gemini")) return "Google models start with gemini.";
    if (providerId === "openai" && !/^(gpt|chatgpt|o1|o3|o4)/i.test(value)) return "Choose an OpenAI model ID.";
    return "";
  }

  function showStatus(type: "success" | "error", text: string) {
    status = { type, text };
    setTimeout(() => (status = null), 3500);
  }

  async function persistSettings() {
    await invoke("save_settings", {
      input: {
        mode,
        provider,
        model,
        defaultPlatform,
        defaultImagePlatform,
        autoDetectTarget,
        selectedTextEnabled,
        terminalPlatform: terminalPlatform || null,
        supermemoryEnabled,
        saveHistoryEnabled,
      },
    });
  }

  async function saveConfig(message = "Settings saved") {
    if (mode === "hosted") mode = "byok";
    if (modelError) {
      showStatus("error", modelError);
      return;
    }
    saving = true;
    try {
      await persistSettings();
      showStatus("success", message);
    } catch (error: any) {
      showStatus("error", `${error}`);
    } finally {
      saving = false;
    }
  }

  async function saveApiKey() {
    if (!apiKey.trim() || modelError) return;
    try {
      await invoke("set_api_key", { service: provider, key: apiKey });
      mode = "byok";
      await persistSettings();
      apiKey = "";
      showStatus("success", `${currentProvider.label} connected`);
    } catch (error: any) {
      showStatus("error", `${error}`);
    }
  }

  async function testConnection() {
    if (modelError) return;
    testingConnection = true;
    try {
      const result = await invoke<string>("test_api_connection", {
        provider,
        model,
        apiKey: apiKey.trim() || null,
      });
      showStatus("success", result);
    } catch (error: any) {
      showStatus("error", `${error}`);
    } finally {
      testingConnection = false;
    }
  }

  async function saveSupermemoryKey() {
    if (!supermemoryKey.trim()) return;
    try {
      await invoke("set_api_key", { service: "supermemory", key: supermemoryKey });
      supermemoryKey = "";
      showStatus("success", "SuperMemory key saved");
    } catch (error: any) {
      showStatus("error", `${error}`);
    }
  }

  async function openAccessibilitySettings() {
    try {
      await invoke("open_accessibility_settings");
    } catch (error: any) {
      showStatus("error", `${error}`);
    }
  }

  async function copyResetCommand() {
    try {
      await invoke("copy_to_clipboard", { text: accessibilityResetCommand });
      showStatus("success", "Reset command copied");
    } catch (error: any) {
      showStatus("error", `${error}`);
    }
  }

  async function runSelfCheck() {
    selfChecking = true;
    try {
      selfCheck = await invoke<QuickEnhanceSelfCheck>("run_quick_enhance_self_check");
      axStatus = selfCheck.accessibility;
      showStatus("success", "Check complete");
    } catch (error: any) {
      showStatus("error", `${error}`);
    } finally {
      selfChecking = false;
    }
  }

  async function inspectRoute() {
    routeInspecting = true;
    try {
      routeInspection = await invoke<QuickEnhanceRouteInspection>("inspect_quick_enhance_route");
    } catch (error: any) {
      routeInspection = { error: `${error}` };
    } finally {
      routeInspecting = false;
    }
  }

  function platformLabel(platform?: string | null) {
    const labels: Record<string, string> = {
      claude: "Claude",
      "claude-code": "Claude Code",
      openai: "GPT",
      gemini: "Gemini",
      cursor: "Cursor",
      codex: "Codex",
      "coding-agent": "Coding Agent",
      generic: "Generic",
    };
    return platform ? (labels[platform] ?? platform) : "Unavailable";
  }

  function relativeTime(timestampMs: number) {
    const seconds = Math.max(0, Math.floor((Date.now() - timestampMs) / 1000));
    if (seconds < 60) return "just now";
    if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)} h ago`;
    return new Date(timestampMs).toLocaleString();
  }
</script>

<div class="page">
  <header class="page-header">
    <span class="eyebrow">SETTINGS</span>
    <h1>Preferences</h1>
  </header>

  <div class="settings-shell">
    <nav class="category-nav" aria-label="Settings categories">
      {#each categories as item}
        <button class:active={category === item.id} onclick={() => (category = item.id)}>
          {item.label}<span>›</span>
        </button>
      {/each}

      <div class="readiness">
        <i class:warning={!ready}></i>
        <span>
          <strong>{ready ? "Proompt is ready" : "Setup needed"}</strong>
          <small>{currentProvider.label} · {model}</small>
        </span>
      </div>
    </nav>

    <main class="category-content">
      {#if category === "provider"}
        <div class="content-header">
          <h2>Provider</h2>
          <p>Choose the AI service Proompt uses.</p>
        </div>

        {#if mode === "hosted"}
          <div class="notice warning">
            <div><strong>Hosted mode is not available yet</strong><span>Saving will switch Proompt back to your own API key.</span></div>
          </div>
        {/if}

        <label class="field">
          <span>Provider</span>
          <select value={provider} onchange={providerChanged}>
            {#each providers as item}<option value={item.id}>{item.label}</option>{/each}
          </select>
        </label>

        <label class="field">
          <span>Model</span>
          <select bind:value={model}>
            {#if !currentProvider.models.includes(model)}<option value={model}>{model} (custom)</option>{/if}
            {#each currentProvider.models as item}<option value={item}>{item}</option>{/each}
          </select>
        </label>

        <details class="disclosure">
          <summary>Use a custom model ID</summary>
          <input class="text-input" type="text" bind:value={model} placeholder="Model ID" />
          <small>{currentProvider.modelHint}</small>
        </details>
        {#if modelError}<p class="field-error">{modelError}</p>{/if}

        <div class="key-box">
          <div><strong>API key</strong><small>Stored securely in your OS keychain</small></div>
          <div class="key-actions">
            <input type="password" bind:value={apiKey} placeholder={currentProvider.keyPlaceholder} />
            <button class="secondary" onclick={testConnection} disabled={testingConnection || !!modelError}>{testingConnection ? "Testing…" : "Test"}</button>
            <button class="secondary" onclick={saveApiKey} disabled={!apiKey.trim() || !!modelError}>Save key</button>
          </div>
        </div>

        <details class="disclosure muted">
          <summary>Command-line setup</summary>
          <div class="code-row"><span>CLI</span><code>{currentProvider.cliCommand}</code></div>
          <div class="code-row"><span>ENV</span><code>export {currentProvider.envVar}=...</code></div>
        </details>

        <button class="primary" onclick={() => saveConfig("Provider saved")} disabled={saving || !!modelError}>{saving ? "Saving…" : "Save provider"}</button>

      {:else if category === "quick"}
        <div class="content-header">
          <h2>Quick Enhance</h2>
          <p>Control what happens when you press the shortcut.</p>
        </div>

        <div class="shortcut-card">
          <kbd>{hotkeyDisplay}</kbd>
          <span><strong>Enhance from any app</strong><small>Select text, press the shortcut, and Proompt replaces it.</small></span>
        </div>

        <label class="toggle-row">
          <span><strong>Automatic target</strong><small>Detect ChatGPT, Claude, Cursor, and terminal apps.</small></span>
          <input type="checkbox" bind:checked={autoDetectTarget} />
        </label>

        <label class="toggle-row">
          <span><strong>Use selected text</strong><small>Falls back to clipboard content when nothing is selected.</small></span>
          <input type="checkbox" bind:checked={selectedTextEnabled} />
        </label>

        <label class="field">
          <span>Fallback target</span>
          <select bind:value={defaultPlatform}>
            <optgroup label="Chat assistants">
              <option value="claude">Claude</option><option value="openai">OpenAI</option><option value="gemini">Gemini</option><option value="generic">Generic</option>
            </optgroup>
            <optgroup label="Coding agents">
              <option value="claude-code">Claude Code</option><option value="cursor">Cursor</option><option value="codex">Codex</option><option value="coding-agent">Coding Agent</option>
            </optgroup>
          </select>
          <small>Used only when Proompt cannot confidently detect a target.</small>
        </label>

        <label class="field">
          <span>Default image generator</span>
          <select bind:value={defaultImagePlatform}>
            <option value="midjourney">Midjourney</option><option value="dalle">DALL-E</option><option value="stablediffusion">Stable Diffusion</option><option value="generic">Generic</option>
          </select>
        </label>

        <details class="disclosure">
          <summary>Terminal app target</summary>
          <label class="field nested">
            <span>When Quick Enhance runs from a terminal</span>
            <select bind:value={terminalPlatform}>
              <option value="">Use fallback target</option><option value="claude-code">Claude Code</option><option value="cursor">Cursor</option><option value="codex">Codex</option><option value="coding-agent">Coding Agent</option><option value="claude">Claude</option><option value="openai">GPT</option><option value="gemini">Gemini</option><option value="generic">Generic</option>
            </select>
          </label>
        </details>

        <button class="primary" onclick={() => saveConfig("Quick Enhance saved")} disabled={saving}>{saving ? "Saving…" : "Save Quick Enhance"}</button>

      {:else if category === "privacy"}
        <div class="content-header">
          <h2>Privacy</h2>
          <p>Choose what Proompt stores and which integrations it uses.</p>
        </div>

        <div class="privacy-note">
          <strong>Your prompts go directly to {currentProvider.label}</strong>
          <p>API keys stay in your OS keychain. Proompt does not proxy prompts through a hosted server.</p>
        </div>

        <label class="toggle-row">
          <span><strong>Save prompt history</strong><small>Store successful enhancements locally on this device.</small></span>
          <input type="checkbox" bind:checked={saveHistoryEnabled} />
        </label>

        <label class="toggle-row">
          <span><strong>SuperMemory</strong><small>Add relevant personal context to enhanced prompts.</small></span>
          <input type="checkbox" bind:checked={supermemoryEnabled} />
        </label>

        {#if supermemoryEnabled}
          <div class="key-box">
            <div><strong>SuperMemory key</strong><small>Stored in your OS keychain</small></div>
            <div class="key-actions compact">
              <input type="password" bind:value={supermemoryKey} placeholder="sm_..." />
              <button class="secondary" onclick={saveSupermemoryKey} disabled={!supermemoryKey.trim()}>Save key</button>
            </div>
          </div>
        {/if}

        <button class="primary" onclick={() => saveConfig("Privacy settings saved")} disabled={saving}>{saving ? "Saving…" : "Save privacy settings"}</button>

      {:else}
        <div class="content-header">
          <h2>Help</h2>
          <p>Run checks only when something is not working.</p>
        </div>

        <div class="permission-card">
          <div class="permission-state">
            <i class:ok={axStatus?.accessibilityTrusted === true} class:bad={axStatus?.accessibilityTrusted === false}></i>
            <span>
              <strong>{axLoading ? "Checking Accessibility…" : axStatus?.accessibilityTrusted ? "Accessibility allowed" : "Accessibility needs attention"}</strong>
              <small>Required to read and replace selected text on macOS.</small>
            </span>
          </div>
          <button class="secondary" onclick={openAccessibilitySettings}>Open settings</button>
        </div>

        <button class="help-row" onclick={runSelfCheck} disabled={selfChecking}>
          <span><strong>Quick Enhance check</strong><small>Checks Accessibility, clipboard, provider, and routing.</small></span>
          <b>{selfChecking ? "Checking…" : "Run"}</b>
        </button>
        <button class="help-row" onclick={loadAccessibilityStatus} disabled={axLoading}>
          <span><strong>Refresh permission status</strong><small>Check macOS Accessibility again.</small></span>
          <b>Refresh</b>
        </button>

        {#if selfCheck}
          <div class="check-results">
            <div><span>Accessibility</span><strong>{selfCheck.accessibility.accessibilityTrusted ? "Ready" : "Needs access"}</strong></div>
            <div><span>Clipboard</span><strong>{selfCheck.clipboard.writable && selfCheck.clipboard.restored ? "Ready" : "Check failed"}</strong></div>
            <div><span>Provider</span><strong>{selfCheck.provider.api_key_configured ? "Ready" : "Needs key"}</strong></div>
            <div><span>Target</span><strong>{platformLabel(selfCheck.route.resolution?.platform)}</strong></div>
          </div>
          {#if selfCheck.provider.api_key_error}<div class="notice warning">{selfCheck.provider.api_key_error}</div>{/if}
        {/if}

        {#if axStatus?.lastCapture && axStatus.lastCapture.outcome !== "not_started"}
          <div class="last-attempt">
            <span>Last Quick Enhance · {relativeTime(axStatus.lastCapture.timestampMs)}</span>
            <strong>{axStatus.lastCapture.outcome.replaceAll("_", " ")}</strong>
            <small>Target: {platformLabel(axStatus.lastCapture.resolution?.platform)} · Delivery: {axStatus.lastCapture.delivery?.replaceAll("_", " ") ?? "not completed"}</small>
            {#if axStatus.lastCapture.error}<em>{axStatus.lastCapture.error}</em>{/if}
          </div>
        {/if}

        <details class="advanced-help">
          <summary>Advanced diagnostics</summary>
          <div class="advanced-content">
            <p>If an unsigned rebuild loses Accessibility access, reset it and grant permission again.</p>
            <div class="command-row"><code>{accessibilityResetCommand}</code><button class="secondary" onclick={copyResetCommand}>Copy</button></div>
            {#if axStatus?.diagnosticsPath}<small>Local diagnostics: {axStatus.diagnosticsPath}</small>{/if}
            <button class="secondary inspect" onclick={inspectRoute} disabled={routeInspecting}>{routeInspecting ? "Inspecting…" : "Inspect clipboard route"}</button>
            {#if routeInspection}
              <div class="route-result">
                {#if routeInspection.error}<span class="field-error">{routeInspection.error}</span>{/if}
                {#if routeInspection.resolution}<strong>{platformLabel(routeInspection.resolution.platform)}</strong><small>{routeInspection.resolution.reason}</small>{/if}
                {#if routeInspection.promptPreview}<code>{routeInspection.promptPreview}</code>{/if}
              </div>
            {/if}
          </div>
        </details>
      {/if}
    </main>
  </div>

  {#if status}
    <div class="toast" class:success={status.type === "success"} class:error={status.type === "error"}>{status.text}</div>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 22px; color: #f6f6f3; font-size: 15px; line-height: 1.45; }
  .page-header { display: grid; gap: 3px; }
  .eyebrow { color: #858991; font-size: 12px; font-weight: 750; letter-spacing: 1.1px; }
  h1 { font-size: 32px; line-height: 1.1; letter-spacing: -1px; font-weight: 680; }
  h2 { font-size: 26px; line-height: 1.15; letter-spacing: -0.6px; font-weight: 680; }
  button, input, select { font: inherit; }
  button { cursor: pointer; }
  button:disabled { cursor: not-allowed; opacity: .45; }

  .settings-shell { min-height: 500px; display: grid; grid-template-columns: 220px minmax(0, 1fr); border: 1px solid #2d2f34; border-radius: 17px; overflow: hidden; background: #16171a; box-shadow: 0 18px 52px rgba(0,0,0,.18); }
  .category-nav { padding: 16px 12px; border-right: 1px solid #2d2f34; background: #121316; display: flex; flex-direction: column; gap: 5px; }
  .category-nav > button { display: flex; justify-content: space-between; align-items: center; padding: 12px 13px; border: 0; border-radius: 9px; background: transparent; color: #989ba2; text-align: left; font-size: 15px; }
  .category-nav > button:hover { background: #202126; color: #d7d8da; }
  .category-nav > button.active { background: #292b30; color: #f4f4f2; }
  .category-nav > button span { color: #63666d; font-size: 21px; line-height: 1; }

  .readiness { margin-top: auto; display: flex; align-items: center; gap: 11px; padding: 14px 9px 8px; min-width: 0; }
  .readiness i, .permission-state i { width: 9px; height: 9px; flex: 0 0 auto; border-radius: 50%; background: #8fb08b; box-shadow: 0 0 0 4px rgba(143,176,139,.1); }
  .readiness i.warning { background: #c4a46b; box-shadow: 0 0 0 4px rgba(196,164,107,.1); }
  .readiness span { min-width: 0; display: grid; gap: 2px; }
  .readiness strong { font-size: 13px; }
  .readiness small { overflow: hidden; color: #777b83; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }

  .category-content { min-width: 0; padding: 32px 36px 38px; }
  .content-header { margin-bottom: 26px; }
  .content-header p { margin-top: 6px; color: #a1a4aa; font-size: 15px; }

  .field { display: grid; gap: 7px; margin-top: 18px; color: #c9cbd0; font-size: 14px; font-weight: 650; }
  .field small { color: #7f8289; font-size: 12.5px; font-weight: 450; }
  select, .text-input, .key-actions input { width: 100%; padding: 12px 14px; border: 1px solid #393b41; border-radius: 10px; outline: 0; background: #101114; color: #f3f3f1; font-size: 15px; }
  select:focus, .text-input:focus, .key-actions input:focus { border-color: #a9aaac; box-shadow: 0 0 0 3px rgba(220,220,216,.08); }
  .field-error { margin-top: 7px; color: #d79494; font-size: 13px; }
  .nested { margin: 14px 0 2px; }

  .key-box { margin-top: 22px; padding: 17px; border: 1px solid #303239; border-radius: 12px; background: #121317; }
  .key-box > div:first-child { display: grid; gap: 2px; }
  .key-box strong { font-size: 15px; }
  .key-box small { color: #858991; font-size: 12.5px; }
  .key-actions { display: grid; grid-template-columns: minmax(0, 1fr) auto auto; gap: 8px; margin-top: 13px; }
  .key-actions.compact { grid-template-columns: minmax(0, 1fr) auto; }

  .primary, .secondary { border-radius: 9px; font-weight: 650; }
  .primary { margin-top: 26px; padding: 12px 19px; border: 0; background: #efeee9; color: #16171a; }
  .primary:hover:not(:disabled) { background: #fff; }
  .secondary { padding: 10px 13px; border: 1px solid #3a3d43; background: #222328; color: #d6d7d9; font-size: 13px; white-space: nowrap; }
  .secondary:hover:not(:disabled) { border-color: #53565d; background: #2b2d32; color: #fff; }

  .disclosure, .advanced-help { margin-top: 14px; }
  .disclosure summary, .advanced-help summary { width: fit-content; color: #999ca3; cursor: pointer; font-size: 13px; }
  .disclosure[open] summary { margin-bottom: 10px; }
  .disclosure small { display: block; margin-top: 6px; color: #777b83; font-size: 12px; }
  .disclosure.muted { margin-top: 18px; }
  .code-row { display: grid; grid-template-columns: 34px minmax(0, 1fr); gap: 8px; align-items: center; margin-top: 7px; }
  .code-row span { color: #777b83; font-size: 10px; font-weight: 750; }
  code { overflow: hidden; padding: 6px 8px; border: 1px solid #34363c; border-radius: 7px; background: #101114; color: #c4c6ca; font-family: "SF Mono", ui-monospace, monospace; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }

  .shortcut-card { display: flex; align-items: center; gap: 16px; padding: 17px; border: 1px solid #303239; border-radius: 12px; background: #111216; }
  .shortcut-card kbd { padding: 9px 11px; border: 1px solid #42454c; border-radius: 8px; background: #23252a; color: #f0f0ed; font-size: 14px; font-weight: 700; white-space: nowrap; }
  .shortcut-card span, .toggle-row span { display: grid; gap: 3px; }
  .shortcut-card strong, .toggle-row strong { font-size: 15px; }
  .shortcut-card small, .toggle-row small { color: #858991; font-size: 13px; font-weight: 450; }
  .toggle-row { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 19px 0; border-bottom: 1px solid #292b30; color: #ececea; }
  input[type="checkbox"] { width: 42px; height: 24px; flex: 0 0 auto; accent-color: #ecece8; }

  .privacy-note { padding: 17px; border: 1px solid #303239; border-radius: 12px; background: #111216; }
  .privacy-note p { margin-top: 4px; color: #8d9097; font-size: 13px; }

  .notice { margin-bottom: 18px; padding: 13px 15px; border: 1px solid #3b3d43; border-radius: 10px; color: #cfd0d2; font-size: 13px; }
  .notice div { display: grid; gap: 2px; }
  .notice span { color: #9699a0; }
  .notice.warning { border-color: rgba(196,164,107,.24); background: rgba(196,164,107,.08); color: #dcc69f; }

  .permission-card { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 17px; border: 1px solid #303239; border-radius: 12px; background: #111216; }
  .permission-state { display: flex; align-items: center; gap: 12px; }
  .permission-state i { background: #777b83; box-shadow: 0 0 0 4px rgba(119,123,131,.1); }
  .permission-state i.ok { background: #8fb08b; box-shadow: 0 0 0 4px rgba(143,176,139,.1); }
  .permission-state i.bad { background: #c37878; box-shadow: 0 0 0 4px rgba(195,120,120,.1); }
  .permission-state span { display: grid; gap: 3px; }
  .permission-state small { color: #858991; font-size: 12.5px; }

  .help-row { width: 100%; display: flex; align-items: center; justify-content: space-between; padding: 18px 2px; border: 0; border-bottom: 1px solid #292b30; background: transparent; color: #eeeeec; text-align: left; }
  .help-row span { display: grid; gap: 3px; }
  .help-row small { color: #858991; font-size: 13px; }
  .help-row b { color: #c6c8cd; font-size: 13px; }

  .check-results { display: grid; grid-template-columns: repeat(2, 1fr); gap: 9px; margin-top: 18px; }
  .check-results div { display: grid; gap: 3px; padding: 12px; border: 1px solid #303239; border-radius: 9px; background: #111216; }
  .check-results span { color: #777b83; font-size: 11px; text-transform: uppercase; letter-spacing: .5px; }
  .check-results strong { font-size: 14px; }
  .last-attempt { display: grid; gap: 4px; margin-top: 16px; padding: 15px; border: 1px solid #303239; border-radius: 11px; background: #111216; }
  .last-attempt span { color: #858991; font-size: 12px; }
  .last-attempt strong { text-transform: capitalize; }
  .last-attempt small { color: #a0a3a9; text-transform: capitalize; }
  .last-attempt em { color: #d79494; font-size: 12px; font-style: normal; }

  .advanced-help { margin-top: 20px; padding-top: 4px; }
  .advanced-content { display: grid; gap: 12px; margin-top: 12px; padding: 16px; border: 1px solid #303239; border-radius: 11px; background: #111216; }
  .advanced-content p, .advanced-content small { color: #858991; font-size: 12.5px; }
  .command-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 8px; }
  .inspect { width: fit-content; }
  .route-result { display: grid; gap: 5px; }

  .toast { position: fixed; z-index: 20; right: 24px; bottom: 24px; max-width: 380px; padding: 12px 16px; border-radius: 10px; box-shadow: 0 14px 38px rgba(0,0,0,.35); font-size: 14px; }
  .toast.success { border: 1px solid rgba(143,176,139,.28); background: #202820; color: #d6e4d3; }
  .toast.error { border: 1px solid rgba(195,120,120,.28); background: #2b2020; color: #e7b8b8; }

  @media (max-width: 700px) {
    .settings-shell { grid-template-columns: 1fr; }
    .category-nav { border-right: 0; border-bottom: 1px solid #2d2f34; }
    .readiness { display: none; }
    .category-content { padding: 26px 22px 30px; }
    .key-actions { grid-template-columns: 1fr 1fr; }
    .key-actions input { grid-column: 1 / -1; }
  }
</style>
