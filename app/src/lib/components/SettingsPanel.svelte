<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { initialProvider = null } = $props<{ initialProvider?: string | null }>();

  interface QuickEnhanceRouteInspection {
    promptPreview?: string | null;
    environment?: any;
    resolution?: {
      platform: string;
      source: string;
      confidence: string;
      reason: string;
    } | null;
    error?: string | null;
  }

  interface AccessibilityStatus {
    platformSupported: boolean;
    accessibilityTrusted?: boolean | null;
    selectedTextEnabled: boolean;
    diagnosticsPath: string;
    lastCapture?: {
      timestampMs: number;
      selectedTextEnabled: boolean;
      outcome: string;
      steps: string[];
    } | null;
  }

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
  let routeInspection = $state<QuickEnhanceRouteInspection | null>(null);
  let routeInspecting = $state(false);
  let axStatus = $state<AccessibilityStatus | null>(null);
  let axLoading = $state(false);
  let axStepsExpanded = $state(false);
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
  let quickEnhanceHotkeyDisplay = $derived(formatHotkey(quickEnhanceHotkey));
  const accessibilityResetCommand = "tccutil reset Accessibility com.proompt.desktop";

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
      if (initialProvider && providers.some((p) => p.id === initialProvider)) {
        selectProvider(initialProvider);
      }
    } catch (e: any) {
      showStatus("error", `Failed to load: ${e}`);
    }
  }

  $effect(() => { loadConfig(); loadAccessibilityStatus(); });

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

  async function openAccessibilitySettings() {
    try {
      await invoke("open_accessibility_settings");
    } catch (e: any) {
      showStatus("error", `${e}`);
    }
  }

  async function copyAccessibilityResetCommand() {
    try {
      await invoke("copy_to_clipboard", { text: accessibilityResetCommand });
      showStatus("success", "Accessibility reset command copied");
    } catch (e: any) {
      try {
        await navigator.clipboard.writeText(accessibilityResetCommand);
        showStatus("success", "Accessibility reset command copied");
      } catch {
        showStatus("error", `${e}`);
      }
    }
  }

  function captureOutcomeLabel(outcome: string) {
    const [kind, detail] = outcome.split(":", 2);
    const chars = detail?.trim() ? ` (${detail.trim()})` : "";
    const labels: Record<string, string> = {
      selected_text_via_accessibility: "Selected text captured via Accessibility",
      selected_text_via_clipboard: "Selected text captured via clipboard probe",
      clipboard_fallback: "Fell back to clipboard prompt",
      empty_input: "No selected text or clipboard text found",
      not_started: "No capture recorded yet",
    };
    return (labels[kind] ?? kind) + chars;
  }

  function captureOutcomeIsFallback(outcome: string) {
    return outcome.startsWith("clipboard_fallback") || outcome.startsWith("empty_input");
  }

  function relativeTime(timestampMs: number) {
    if (!timestampMs) return "unknown time";
    const seconds = Math.max(0, Math.floor((Date.now() - timestampMs) / 1000));
    if (seconds < 60) return "just now";
    if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)} h ago`;
    return new Date(timestampMs).toLocaleString();
  }

  function selectProvider(providerId: string) {
    provider = providerId;
    const selectedProvider = providers.find((p) => p.id === providerId) ?? providers[0];
    model = selectedProvider.models[0];
  }

  function selectMode(nextMode: "byok" | "hosted") {
    if (nextMode === "hosted") {
      showStatus("error", "Hosted mode is coming soon. Use BYOK for now.");
      return;
    }
    mode = nextMode;
  }

  function formatHotkey(hotkey: string) {
    const isMac = typeof navigator !== "undefined" && navigator.platform?.includes("Mac");
    return hotkey
      .replace("CmdOrCtrl", isMac ? "⌘" : "Ctrl")
      .replace("Shift", isMac ? "⇧" : "Shift")
      .replace(/\+/g, isMac ? "" : " + ");
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

  async function saveApiKey() {
    if (!apiKey.trim()) return;
    if (modelError) {
      showStatus("error", modelError);
      return;
    }
    try {
      await invoke("set_api_key", { service: provider, key: apiKey });
      mode = "byok";
      await persistSettings();
      showStatus("success", `${currentProvider.label} key saved and selected`);
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

  async function inspectRoute() {
    routeInspecting = true;
    try {
      routeInspection = await invoke<QuickEnhanceRouteInspection>("inspect_quick_enhance_route");
    } catch (e: any) {
      routeInspection = { error: e?.toString?.() ?? `${e}` };
    } finally {
      routeInspecting = false;
    }
  }

  function platformLabel(platform: string) {
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
    return labels[platform] ?? platform;
  }

  function routingSourceLabel(source: string) {
    const labels: Record<string, string> = {
      explicit_prefix: "Explicit prefix",
      active_app: "Active app",
      browser_context: "Browser context",
      terminal_default: "Terminal default",
      config_default: "Quick Enhance fallback",
    };
    return labels[source] ?? source;
  }

  function routingConfidenceLabel(confidence: string) {
    const labels: Record<string, string> = {
      explicit: "explicit",
      high: "high confidence",
      medium: "medium confidence",
      fallback: "fallback",
    };
    return labels[confidence] ?? confidence;
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
    if (mode === "hosted") {
      showStatus("error", "Hosted mode is coming soon. Choose BYOK to save settings.");
      return;
    }
    if (modelError) {
      showStatus("error", modelError);
      return;
    }
    saving = true;
    try {
      await persistSettings();
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

  <!-- Mode -->
  <section class="section">
    <div class="section-label">Mode</div>
    <div class="mode-grid">
      <button
        class="mode-card"
        class:active={mode === "byok"}
        onclick={() => selectMode("byok")}
      >
        <span class="mode-name">BYOK</span>
        <span class="mode-desc">Use your own API key. Private and free.</span>
      </button>
      <button
        class="mode-card"
        class:active={mode === "hosted"}
        onclick={() => selectMode("hosted")}
      >
        <span class="mode-name">Hosted Pro</span>
        <span class="mode-desc">Coming soon</span>
      </button>
    </div>
  </section>

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
      <button class="btn-secondary" onclick={saveApiKey} disabled={!apiKey.trim() || !!modelError}>Save</button>
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

  <!-- Quick Enhance Target -->
  <section class="section">
    <div class="section-label">Quick Enhance fallback target</div>
    <p class="hint" style="margin: 0 0 8px">Used by {quickEnhanceHotkeyDisplay} when no /prefix or active-app detection matches.</p>
    <div class="select-wrap">
      <select bind:value={defaultPlatform}>
        <optgroup label="Chat assistants">
          <option value="claude">Claude</option>
          <option value="openai">OpenAI</option>
          <option value="gemini">Gemini</option>
          <option value="generic">Generic</option>
        </optgroup>
        <optgroup label="Coding agents">
          <option value="claude-code">Claude Code</option>
          <option value="cursor">Cursor</option>
          <option value="codex">Codex</option>
          <option value="coding-agent">Coding Agent</option>
        </optgroup>
      </select>
    </div>

    <div class="section-row" style="margin-top: 12px">
      <div>
        <div class="section-label" style="margin-bottom: 2px">Auto-detect active app</div>
        <p class="hint" style="margin: 0">Route Quick Enhance to Cursor, ChatGPT, Claude, or browser context when confidently detected.</p>
      </div>
      <label class="toggle">
        <div class="toggle-track" class:on={autoDetectTarget}>
          <input type="checkbox" bind:checked={autoDetectTarget} />
          <div class="toggle-thumb"></div>
        </div>
      </label>
    </div>

    <div class="section-row" style="margin-top: 12px">
      <div>
        <div class="section-label" style="margin-bottom: 2px">Use selected text when available</div>
        <p class="hint" style="margin: 0">Quick Enhance copies the current selection, enhances it, then replaces it when the original app is still focused. Falls back to clipboard.</p>
      </div>
      <label class="toggle">
        <div class="toggle-track" class:on={selectedTextEnabled}>
          <input type="checkbox" bind:checked={selectedTextEnabled} />
          <div class="toggle-thumb"></div>
        </div>
      </label>
    </div>

    <div class="diagnostic-card">
      <div class="diagnostic-top">
        <div>
          <span class="diagnostic-title">Selected-text diagnostics</span>
          <p class="hint" style="margin: 2px 0 0">Accessibility permission state and the last capture attempt.</p>
        </div>
        <button class="btn-secondary" onclick={loadAccessibilityStatus} disabled={axLoading}>
          {axLoading ? "Checking..." : "Refresh"}
        </button>
      </div>

      {#if axStatus}
        {#if !axStatus.platformSupported}
          <p class="hint">Selected-text capture is only available on macOS.</p>
        {:else}
          <div class="ax-status-row">
            <span
              class="status-dot"
              class:ok={axStatus.accessibilityTrusted === true}
              class:bad={axStatus.accessibilityTrusted === false}
            ></span>
            <span class="ax-status-text">
              {axStatus.accessibilityTrusted
                ? "Accessibility permission granted"
                : "Accessibility permission not granted"}
            </span>
            {#if axStatus.accessibilityTrusted === false}
              <button class="btn-secondary" onclick={openAccessibilitySettings}>Open System Settings</button>
            {/if}
          </div>

          {#if axStatus.accessibilityTrusted === false}
            <div class="ax-help-card">
              <strong>Enable selected-text Quick Enhance</strong>
              <ol>
                <li>Open Privacy &amp; Security → Accessibility.</li>
                <li>Enable Proompt for the currently installed app.</li>
                <li>If Proompt already looks enabled but capture still fails, reset permission and grant it again.</li>
              </ol>
              <div class="command-row">
                <code>{accessibilityResetCommand}</code>
                <button class="btn-secondary" onclick={copyAccessibilityResetCommand}>Copy reset command</button>
              </div>
              <p class="hint" style="margin: 0">
                This is expected for unsigned macOS builds after replacing or rebuilding the app. It does not send selected text or prompt content anywhere.
              </p>
            </div>
          {:else if axStatus.accessibilityTrusted === true}
            <p class="hint">
              If a future unsigned update stops capturing selected text, reset Accessibility for Proompt and grant it again.
            </p>
          {/if}

          {#if axStatus.lastCapture && axStatus.lastCapture.outcome !== "not_started"}
            <div class="diagnostic-row">
              <span>Last capture · {relativeTime(axStatus.lastCapture.timestampMs)}</span>
              <strong
                class="ax-outcome"
                class:fallback={captureOutcomeIsFallback(axStatus.lastCapture.outcome)}
              >{captureOutcomeLabel(axStatus.lastCapture.outcome)}</strong>
            </div>
            {#if axStatus.lastCapture.steps.length > 0}
              <button class="ax-steps-toggle" onclick={() => (axStepsExpanded = !axStepsExpanded)}>
                {axStepsExpanded ? "Hide capture steps" : `Show capture steps (${axStatus.lastCapture.steps.length})`}
              </button>
              {#if axStepsExpanded}
                <ol class="ax-steps">
                  {#each axStatus.lastCapture.steps as step}
                    <li>{step}</li>
                  {/each}
                </ol>
              {/if}
            {/if}
          {:else}
            <p class="hint">No capture recorded yet. Trigger Quick Enhance once to populate this.</p>
          {/if}
        {/if}
      {:else if !axLoading}
        <p class="hint">Diagnostics unavailable.</p>
      {/if}
    </div>

    <div class="section-label" style="margin-top: 14px">Terminal default target</div>
    <p class="hint" style="margin: 0 0 8px">Used for Terminal, Ghostty, iTerm, Warp, and similar apps when no /prefix is present.</p>
    <div class="select-wrap">
      <select bind:value={terminalPlatform}>
        <option value="">Use fallback target</option>
        <option value="claude-code">Claude Code</option>
        <option value="cursor">Cursor</option>
        <option value="codex">Codex</option>
        <option value="coding-agent">Coding Agent</option>
        <option value="claude">Claude</option>
        <option value="openai">GPT</option>
        <option value="gemini">Gemini</option>
        <option value="generic">Generic</option>
      </select>
    </div>

    <div class="diagnostic-card">
      <div class="diagnostic-top">
        <div>
          <span class="diagnostic-title">Clipboard route preview</span>
          <p class="hint" style="margin: 2px 0 0">Preview saved-settings and /prefix routing. Actual hotkey active-app routes are recorded in History.</p>
        </div>
        <button class="btn-secondary" onclick={inspectRoute} disabled={routeInspecting}>
          {routeInspecting ? "Inspecting..." : "Inspect"}
        </button>
      </div>

      {#if routeInspection}
        {#if routeInspection.error}
          <div class="diagnostic-error">{routeInspection.error}</div>
        {/if}

        {#if routeInspection.resolution}
          <div class="diagnostic-result">
            <span>Resolved target</span>
            <strong>{platformLabel(routeInspection.resolution.platform)}</strong>
            <small>{routingSourceLabel(routeInspection.resolution.source)} · {routingConfidenceLabel(routeInspection.resolution.confidence)} · {routeInspection.resolution.reason}</small>
          </div>
        {/if}

        {#if routeInspection.promptPreview}
          <div class="diagnostic-row">
            <span>Clipboard</span>
            <code>{routeInspection.promptPreview}</code>
          </div>
        {/if}

        {#if routeInspection.environment}
          <div class="diagnostic-grid">
            <div>
              <span>Active app</span>
              <strong>{routeInspection.environment.active_app?.name ?? "Unknown"}</strong>
            </div>
            <div>
              <span>Window</span>
              <strong>{routeInspection.environment.window_title ?? "Unavailable"}</strong>
            </div>
            <div>
              <span>Browser URL</span>
              <strong>{routeInspection.environment.browser_context?.url ?? "Unavailable"}</strong>
            </div>
          </div>
        {/if}
      {/if}
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

  <!-- Hotkeys -->
  <section class="section">
    <div class="section-label">Hotkeys</div>
    <div class="hotkey-card">
      <div>
        <span class="hotkey-name">Quick Enhance</span>
        <p class="hint" style="margin: 2px 0 0">Reads clipboard, routes by /prefix or target above, enhances it, and copies the result back.</p>
      </div>
      <kbd>{quickEnhanceHotkeyDisplay}</kbd>
    </div>
  </section>

  <!-- Privacy -->
  <section class="section">
    <div class="section-row">
      <div>
        <div class="section-label" style="margin-bottom: 2px">Local history</div>
        <p class="hint" style="margin: 0">Save successful prompt enhancements locally on this device.</p>
      </div>
      <label class="toggle">
        <div class="toggle-track" class:on={saveHistoryEnabled}>
          <input type="checkbox" bind:checked={saveHistoryEnabled} />
          <div class="toggle-thumb"></div>
        </div>
      </label>
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
    color: #f5f5f5;
    letter-spacing: -0.5px;
  }

  .subtitle {
    font-size: 13px;
    color: #787878;
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
    color: #bebebe;
  }

  .section-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  /* ── Mode and provider cards ──────── */

  .mode-grid,
  .provider-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .mode-card,
  .provider-card {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 12px;
    background: #171717;
    border: 1px solid #2a2a2a;
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s ease;
  }

  .mode-card:hover,
  .provider-card:hover {
    border-color: #3a3a3a;
    background: #202020;
  }

  .mode-card.active,
  .provider-card.active {
    border-color: rgba(214, 211, 209, 0.40);
    background: rgba(214, 211, 209, 0.05);
  }

  .mode-name,
  .provider-name {
    font-size: 13px;
    font-weight: 600;
    color: #eeeeee;
  }

  .mode-card.active .mode-name,
  .provider-card.active .provider-name {
    color: #f5f5f4;
  }

  .mode-desc,
  .provider-desc {
    font-size: 11px;
    color: #787878;
  }

  /* ── Hotkeys ──────────────────────── */

  .hotkey-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px;
    background: #171717;
    border: 1px solid #2a2a2a;
    border-radius: 10px;
  }

  .hotkey-name {
    font-size: 13px;
    font-weight: 600;
    color: #eeeeee;
  }

  kbd {
    font-family: inherit;
    font-size: 11px;
    padding: 4px 8px;
    background: #202020;
    color: #bebebe;
    border-radius: 6px;
    border: 1px solid #3a3a3a;
    font-weight: 600;
    white-space: nowrap;
  }

  /* ── Form elements ────────────────── */

  .select-wrap select,
  .model-input,
  .key-row input {
    padding: 8px 12px;
    background: #171717;
    border: 1px solid #2a2a2a;
    color: #eeeeee;
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
    border-color: #d6d3d1;
    box-shadow: 0 0 0 3px rgba(214, 211, 209, 0.08);
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
    color: #5f5f5f;
    margin: 0;
  }

  .field-error {
    font-size: 11px;
    color: #d08c8c;
    margin: 0;
  }

  .setup-guide {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    background: #171717;
    border: 1px solid #2a2a2a;
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
    color: #787878;
  }

  code {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 3px 6px;
    background: #202020;
    border: 1px solid #3a3a3a;
    border-radius: 5px;
    color: #bebebe;
    font-family: "SF Mono", "Fira Code", ui-monospace, monospace;
    font-size: 10.5px;
  }

  /* ── Routing diagnostics ──────────── */

  .diagnostic-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 12px;
    padding: 12px;
    background: #171717;
    border: 1px solid #2a2a2a;
    border-radius: 10px;
  }

  .diagnostic-top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }

  .diagnostic-title {
    font-size: 13px;
    font-weight: 600;
    color: #eeeeee;
  }

  .diagnostic-error {
    padding: 8px 10px;
    background: rgba(184, 92, 92, 0.10);
    border: 1px solid rgba(184, 92, 92, 0.18);
    color: #d08c8c;
    border-radius: 8px;
    font-size: 11.5px;
  }

  .diagnostic-result {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 10px;
    background: rgba(214, 211, 209, 0.06);
    border: 1px solid rgba(214, 211, 209, 0.14);
    border-radius: 8px;
  }

  .diagnostic-result span,
  .diagnostic-row span,
  .diagnostic-grid span {
    font-size: 10px;
    color: #787878;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 700;
  }

  .diagnostic-result strong {
    color: #f5f5f4;
    font-size: 13px;
  }

  .diagnostic-result small {
    color: #d6d3d1;
    font-size: 11px;
    line-height: 1.4;
  }

  .diagnostic-row {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }

  .diagnostic-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }

  .diagnostic-grid div {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .diagnostic-grid strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #bebebe;
    font-size: 11px;
    font-weight: 500;
  }

  /* ── Accessibility diagnostics ────── */

  .ax-status-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 99px;
    background: #787878;
    flex-shrink: 0;
  }

  .status-dot.ok { background: #d6d3d1; }
  .status-dot.bad { background: #b85c5c; }

  .ax-status-text {
    font-size: 13px;
    font-weight: 500;
    color: #eeeeee;
    flex: 1;
    min-width: 0;
  }

  .ax-outcome {
    color: #f5f5f4;
    font-size: 13px;
    font-weight: 600;
  }

  .ax-outcome.fallback {
    color: #c4a46b;
  }

  .ax-help-card {
    display: flex;
    flex-direction: column;
    gap: 9px;
    padding: 10px;
    background: rgba(214, 211, 209, 0.05);
    border: 1px solid rgba(214, 211, 209, 0.12);
    border-radius: 8px;
  }

  .ax-help-card strong {
    color: #eeeeee;
    font-size: 12.5px;
  }

  .ax-help-card ol {
    margin: 0;
    padding-left: 18px;
    color: #bebebe;
    font-size: 11.5px;
    line-height: 1.5;
  }

  .command-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .command-row code {
    flex: 1;
  }

  .ax-steps-toggle {
    background: none;
    border: none;
    color: #9a9a9a;
    font-size: 11px;
    cursor: pointer;
    padding: 0;
    margin-top: 4px;
  }

  .ax-steps-toggle:hover {
    color: #bebebe;
  }

  .ax-steps {
    margin: 6px 0 0;
    padding-left: 20px;
    font-size: 11px;
    color: #9a9a9a;
    line-height: 1.6;
  }

  /* ── Buttons ──────────────────────── */

  .btn-primary {
    padding: 9px 20px;
    background: #d6d3d1;
    color: #171717;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    transition: all 0.12s ease;
  }

  .btn-primary:hover:not(:disabled) { background: #f5f5f4; }
  .btn-primary:disabled { opacity: 0.35; cursor: not-allowed; }
  .full-width { width: 100%; }

  .btn-secondary {
    padding: 8px 14px;
    background: #202020;
    color: #bebebe;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 550;
    white-space: nowrap;
    transition: all 0.12s ease;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #3a3a3a;
    color: #eeeeee;
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
    background: #3a3a3a;
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
    background: #787878;
    border-radius: 99px;
    transition: all 0.15s ease;
  }

  .toggle-track.on { background: #a8a29e; }
  .toggle-track.on .toggle-thumb { left: 18px; background: #ffffff; }

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
    background: rgba(214, 211, 209, 0.08);
    border: 1px solid rgba(214, 211, 209, 0.20);
    color: #f5f5f4;
  }

  .toast.error {
    background: rgba(184, 92, 92, 0.10);
    border: 1px solid rgba(184, 92, 92, 0.22);
    color: #d08c8c;
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
