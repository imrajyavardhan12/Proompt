<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface AppConfig {
    default_platform?: string;
    default_image_platform?: string;
    hotkeys?: { quick_enhance?: string };
    quick_enhance?: {
      auto_detect_target?: boolean;
      selected_text_enabled?: boolean;
    };
  }

  interface EnhanceDraft {
    id: string;
    prompt: string;
    platform: string;
    mode: "text" | "image";
  }

  interface ProviderSetupStatus {
    mode: "byok" | "hosted";
    provider: string;
    model: string;
    api_key_configured: boolean;
    api_key_status?: "env_configured" | "deferred" | "hosted_unavailable" | "missing" | "error";
    api_key_error?: string | null;
    env_var: string;
    cli_command: string;
  }

  interface RecentPrompt {
    id: string;
    original_prompt: string;
    enhancement_type: "text" | "image";
    platform: string;
    created_at_ms: number;
  }

  type SettingsSection = "provider" | "troubleshoot";

  let {
    onOpenSettings = () => {},
    onOpenHistory = () => {},
    onOpenTemplates = () => {},
    draft = null,
  } = $props<{
    onOpenSettings?: (providerHint?: string, sectionHint?: SettingsSection) => void;
    onOpenHistory?: () => void;
    onOpenTemplates?: () => void;
    draft?: EnhanceDraft | null;
  }>();

  let prompt = $state("");
  let enhancedPrompt = $state("");
  let platform = $state("claude");
  let mode = $state<"text" | "image">("text");
  let isLoading = $state(false);
  let includeMemory = $state(false);
  let error = $state("");
  let copied = $state(false);
  let selectedStyles = $state<Set<string>>(new Set());
  let resultPlatform = $state("");
  let resultMode = $state<"text" | "image">("text");
  let defaultTextPlatform = $state("claude");
  let defaultImagePlatform = $state("midjourney");
  let quickEnhanceHotkey = $state("CmdOrCtrl+Shift+E");
  let autoDetectTarget = $state(true);
  let selectedTextEnabled = $state(true);
  let providerSetup = $state<ProviderSetupStatus | null>(null);
  let setupStatusLoading = $state(true);
  let appliedDraftId = $state<string | null>(null);
  let recentPrompts = $state<RecentPrompt[]>([]);
  let promptInput: HTMLTextAreaElement;

  const textPlatformGroups = [
    {
      label: "Chat assistants",
      platforms: [
        { id: "claude", label: "Claude" },
        { id: "openai", label: "GPT" },
        { id: "gemini", label: "Gemini" },
        { id: "generic", label: "Generic" },
      ],
    },
    {
      label: "Coding agents",
      platforms: [
        { id: "claude-code", label: "Claude Code" },
        { id: "cursor", label: "Cursor" },
        { id: "codex", label: "Codex" },
        { id: "coding-agent", label: "Coding Agent" },
      ],
    },
  ];

  const textPlatforms = textPlatformGroups.flatMap((group) => group.platforms);
  const imagePlatforms = [
    { id: "midjourney", label: "Midjourney" },
    { id: "dalle", label: "DALL-E" },
    { id: "stablediffusion", label: "Stable Diffusion" },
    { id: "generic", label: "Generic" },
  ];
  const styleOptions = [
    "Photorealistic", "Cinematic", "Anime", "Oil Painting",
    "3D Render", "Watercolor", "Sketch", "Pixel Art",
  ];

  let platforms = $derived(mode === "text" ? textPlatforms : imagePlatforms);
  let isResultStale = $derived(Boolean(enhancedPrompt) && (platform !== resultPlatform || mode !== resultMode));
  let providerNeedsSetup = $derived(Boolean(providerSetup && providerSetup.mode === "byok" && providerSetup.api_key_status === "missing"));
  let hostedModeUnavailable = $derived(Boolean(providerSetup && providerSetup.mode === "hosted"));
  let missingKeyError = $derived(isMissingApiKeyError(error));
  let hostedModeError = $derived(isHostedModeError(error));
  let setupIssueVisible = $derived(!setupStatusLoading && (providerNeedsSetup || hostedModeUnavailable || missingKeyError || hostedModeError));
  let activeProviderLabel = $derived(providerLabel(providerSetup?.provider || "openai"));
  let quickEnhanceHotkeyDisplay = $derived(formatHotkey(quickEnhanceHotkey));
  let submitShortcutDisplay = $derived(
    typeof navigator !== "undefined" && navigator.platform?.includes("Mac") ? "⌘↵" : "Ctrl ↵"
  );
  let quickEnhanceTargetLabel = $derived(getPlatformLabel(defaultTextPlatform, "text"));
  let readinessLabel = $derived(getReadinessLabel());
  let readinessProblem = $derived(
    !setupStatusLoading && (!providerSetup || providerNeedsSetup || hostedModeUnavailable || providerSetup.api_key_status === "error")
  );
  let targetControlLabel = $derived(getPlatformLabel(platform, mode));
  let charCount = $derived(prompt.length);

  function getPlatformLabel(id: string, enhancementMode: "text" | "image") {
    const options = enhancementMode === "text" ? textPlatforms : imagePlatforms;
    return options.find((item) => item.id === id)?.label ?? id;
  }

  function providerLabel(providerId: string) {
    const labels: Record<string, string> = {
      openai: "OpenAI",
      anthropic: "Anthropic",
      google: "Google",
      openrouter: "OpenRouter",
    };
    return labels[providerId] ?? providerId;
  }

  function formatHotkey(hotkey: string) {
    const isMac = typeof navigator !== "undefined" && navigator.platform?.includes("Mac");
    return hotkey
      .replace("CmdOrCtrl", isMac ? "⌘" : "Ctrl")
      .replace("Shift", isMac ? "⇧" : "Shift")
      .replace(/\+/g, isMac ? "" : " + ");
  }

  function isMissingApiKeyError(message: string) {
    const normalized = message.toLowerCase();
    return normalized.includes("api key not configured")
      || normalized.includes("failed to get api key")
      || normalized.includes("api key not found")
      || normalized.includes("empty api key configured");
  }

  function isHostedModeError(message: string) {
    return message.toLowerCase().includes("hosted mode");
  }

  function getReadinessLabel() {
    if (setupStatusLoading) return "Checking setup";
    if (hostedModeUnavailable) return "Hosted unavailable";
    if (!providerSetup || providerSetup.api_key_status === "error") return "Check setup";
    if (providerNeedsSetup) return "Provider key needed";
    return "Ready";
  }

  async function loadConfigDefaults() {
    try {
      const config = await invoke<AppConfig>("get_config");
      defaultTextPlatform = config.default_platform?.toLowerCase() || "claude";
      defaultImagePlatform = config.default_image_platform?.toLowerCase() || "midjourney";
      quickEnhanceHotkey = config.hotkeys?.quick_enhance || "CmdOrCtrl+Shift+E";
      autoDetectTarget = config.quick_enhance?.auto_detect_target ?? true;
      selectedTextEnabled = config.quick_enhance?.selected_text_enabled ?? true;
      if (!draft) platform = defaultTextPlatform;
    } catch {
      defaultTextPlatform = "claude";
      defaultImagePlatform = "midjourney";
      quickEnhanceHotkey = "CmdOrCtrl+Shift+E";
      autoDetectTarget = true;
      selectedTextEnabled = true;
      if (!draft) platform = "claude";
    }
  }

  async function loadProviderSetup() {
    setupStatusLoading = true;
    try {
      providerSetup = await invoke<ProviderSetupStatus>("get_provider_setup_status");
    } catch {
      providerSetup = null;
    } finally {
      setupStatusLoading = false;
    }
  }

  async function loadRecentPrompts() {
    try {
      recentPrompts = await invoke<RecentPrompt[]>("list_history", {
        limit: 3,
        favoritesOnly: false,
      });
    } catch {
      recentPrompts = [];
    }
  }

  $effect(() => {
    loadConfigDefaults();
    loadProviderSetup();
    loadRecentPrompts();
  });

  $effect(() => {
    if (draft && draft.id !== appliedDraftId) {
      applyPrompt(draft.prompt, draft.mode, draft.platform);
      appliedDraftId = draft.id;
    }
  });

  $effect(() => {
    const ids = platforms.map((item) => item.id);
    const preferred = mode === "text" ? defaultTextPlatform : defaultImagePlatform;
    if (!ids.includes(platform)) platform = ids.includes(preferred) ? preferred : ids[0];
  });

  function applyPrompt(nextPrompt: string, nextMode: "text" | "image", nextPlatform: string) {
    prompt = nextPrompt;
    mode = nextMode;
    platform = nextPlatform;
    enhancedPrompt = "";
    error = "";
    copied = false;
    selectedStyles = new Set();
  }

  function reuseRecent(record: RecentPrompt) {
    applyPrompt(record.original_prompt, record.enhancement_type, record.platform);
    requestAnimationFrame(() => {
      promptInput?.scrollIntoView({ behavior: "smooth", block: "center" });
      promptInput?.focus({ preventScroll: true });
    });
  }

  function toggleStyle(style: string) {
    const next = new Set(selectedStyles);
    if (next.has(style)) next.delete(style);
    else next.add(style);
    selectedStyles = next;
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      handleEnhance();
    }
  }

  async function handleEnhance() {
    if (!prompt.trim() || isLoading || setupStatusLoading || providerNeedsSetup || hostedModeUnavailable) return;
    isLoading = true;
    error = "";
    enhancedPrompt = "";
    const requestedPlatform = platform;
    const requestedMode = mode;
    try {
      enhancedPrompt = await invoke<string>("enhance_prompt", {
        prompt,
        platform: requestedPlatform,
        enhanceType: requestedMode,
        includeMemory,
        styleHints: requestedMode === "image" ? Array.from(selectedStyles) : null,
      });
      resultPlatform = requestedPlatform;
      resultMode = requestedMode;
      loadProviderSetup();
      loadRecentPrompts();
    } catch (caught: any) {
      const message = caught?.toString?.() ?? `${caught}`;
      error = message;
      if (isMissingApiKeyError(message) || isHostedModeError(message)) await loadProviderSetup();
    } finally {
      isLoading = false;
    }
  }

  async function copyToClipboard() {
    if (!enhancedPrompt) return;
    try {
      await invoke("copy_to_clipboard", { text: enhancedPrompt });
    } catch {
      await navigator.clipboard.writeText(enhancedPrompt);
    }
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function relativeTime(timestampMs: number) {
    if (!timestampMs) return "Unknown";
    const seconds = Math.max(0, Math.floor((Date.now() - timestampMs) / 1000));
    if (seconds < 60) return "Just now";
    if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)} h ago`;
    return new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" }).format(new Date(timestampMs));
  }

  function targetMark(record: RecentPrompt) {
    const label = getPlatformLabel(record.platform, record.enhancement_type);
    return label.split(/\s+/).map((part) => part[0]).join("").slice(0, 2).toUpperCase();
  }
</script>

<div class="workspace">
  <header class="intro">
    <p>Prompt workspace</p>
    <h1>What do you want the AI to do?</h1>
  </header>

  {#if setupIssueVisible}
    <section class="setup-alert" class:warning={hostedModeUnavailable || hostedModeError}>
      <div>
        <strong>{hostedModeUnavailable || hostedModeError ? "Hosted mode is not available yet" : "Add a provider key to start"}</strong>
        <span>
          {hostedModeUnavailable || hostedModeError
            ? "Switch to BYOK and connect a provider."
            : `Connect ${activeProviderLabel} for ${providerSetup?.model || "your selected model"}.`}
        </span>
      </div>
      <button onclick={() => onOpenSettings(hostedModeUnavailable || hostedModeError ? undefined : "openrouter", "provider")}>
        {hostedModeUnavailable || hostedModeError ? "Open settings" : "Set up OpenRouter"}
      </button>
    </section>
  {/if}

  <section class="composer">
    <div class="composer-top">
      <div class="mode-switch" aria-label="Enhancement type">
        <button class:active={mode === "text"} onclick={() => (mode = "text")}>Text</button>
        <button class:active={mode === "image"} onclick={() => (mode = "image")}>Image</button>
      </div>
      <button class="template-button" onclick={onOpenTemplates}>
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>
        </svg>
        Templates
      </button>
    </div>

    <textarea
      bind:this={promptInput}
      bind:value={prompt}
      onkeydown={handleKeydown}
      placeholder={mode === "text" ? "Describe the task, constraints, and desired outcome..." : "Describe the image you want to create..."}
      aria-label="Prompt to enhance"
    ></textarea>

    {#if mode === "image"}
      <div class="style-row">
        <span>Style</span>
        <div>
          {#each styleOptions as style}
            <button class:active={selectedStyles.has(style)} onclick={() => toggleStyle(style)}>{style}</button>
          {/each}
        </div>
      </div>
    {/if}

    <div class="composer-controls">
      <div class="control-group">
        <label class="target-select">
          <span>Target</span>
          <strong>{targetControlLabel}</strong>
          <select bind:value={platform} aria-label="Target platform">
            {#if mode === "text"}
              {#each textPlatformGroups as group}
                <optgroup label={group.label}>
                  {#each group.platforms as item}<option value={item.id}>{item.label}</option>{/each}
                </optgroup>
              {/each}
            {:else}
              {#each imagePlatforms as item}<option value={item.id}>{item.label}</option>{/each}
            {/if}
          </select>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
        </label>

        <label class="memory-toggle" class:on={includeMemory} title="Include SuperMemory context">
          <input type="checkbox" bind:checked={includeMemory} />
          Memory
        </label>
        <span class="char-count">{charCount}</span>
      </div>

      <button
        class="enhance-button"
        onclick={handleEnhance}
        disabled={isLoading || !prompt.trim() || setupStatusLoading || providerNeedsSetup || hostedModeUnavailable}
      >
        {#if isLoading}<span class="spinner"></span>Enhancing…{:else}Enhance <kbd>{submitShortcutDisplay}</kbd>{/if}
      </button>
    </div>
  </section>

  <p class="hotkey-hint">
    Or {selectedTextEnabled ? "select or copy text" : "copy text"} in any app and press
    <kbd>{quickEnhanceHotkeyDisplay}</kbd>. Proompt {selectedTextEnabled ? "replaces it when safe" : "copies the result"}.
  </p>

  <section class="readiness" class:problem={readinessProblem}>
    <span class="status-dot"></span>
    <strong>{readinessLabel}</strong>
    <span class="separator">·</span>
    <span>{activeProviderLabel}{providerSetup?.model ? ` / ${providerSetup.model}` : ""}</span>
    <span class="separator">·</span>
    <span>{autoDetectTarget ? "Auto-route" : `${quickEnhanceTargetLabel} fallback`}</span>
    <span class="separator">·</span>
    <span>Selected text {selectedTextEnabled ? "enabled" : "off"}</span>
    <button onclick={() => onOpenSettings(undefined, "troubleshoot")}>Troubleshoot</button>
  </section>

  {#if error && !missingKeyError && !hostedModeError}
    <div class="error-alert">
      <span>{error}</span>
      <button onclick={() => onOpenSettings(undefined, "troubleshoot")}>Troubleshoot</button>
    </div>
  {/if}

  {#if enhancedPrompt}
    <section class="result">
      <header>
        <div>
          <span class="result-dot"></span>
          Enhanced for {getPlatformLabel(resultPlatform, resultMode)}
          {#if isResultStale}<em>Selection changed · regenerate</em>{/if}
        </div>
        <button class:copied onclick={copyToClipboard}>
          {#if copied}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="m20 6-11 11-5-5"/></svg>
            Copied
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
            Copy
          {/if}
        </button>
      </header>
      <pre>{enhancedPrompt}</pre>
    </section>
  {/if}

  {#if recentPrompts.length > 0}
    <section class="recents">
      <header><strong>Recent work</strong><button onclick={onOpenHistory}>View all</button></header>
      <div class="recent-list">
        {#each recentPrompts as record}
          <button class="recent-item" onclick={() => reuseRecent(record)}>
            <span class="target-mark">{targetMark(record)}</span>
            <span class="recent-prompt">{record.original_prompt}</span>
            <small>{relativeTime(record.created_at_ms)}</small>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 17 17 7M7 7h10v10"/></svg>
          </button>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .workspace { display: flex; flex-direction: column; gap: 16px; }

  .intro { text-align: center; margin-bottom: 4px; }
  .intro p { color: #777a80; font-size: 10px; font-weight: 750; letter-spacing: 1.15px; text-transform: uppercase; }
  .intro h1 { margin-top: 8px; color: #f2f1ee; font-size: 29px; font-weight: 650; letter-spacing: -0.9px; }

  .setup-alert,
  .error-alert { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 11px 13px; border: 1px solid rgba(196, 164, 107, 0.24); border-radius: 10px; background: rgba(196, 164, 107, 0.08); }
  .setup-alert div { display: grid; gap: 3px; }
  .setup-alert strong { color: #e1d3b8; font-size: 12px; }
  .setup-alert span { color: #9b8e77; font-size: 10.5px; }
  .setup-alert button,
  .error-alert button { flex-shrink: 0; padding: 6px 9px; border: 1px solid #464137; border-radius: 7px; background: transparent; color: #d1bea0; cursor: pointer; font-size: 10.5px; }

  .composer { border: 1px solid #36383e; border-radius: 14px; overflow: hidden; background: rgba(23, 24, 29, 0.94); box-shadow: 0 22px 70px rgba(0, 0, 0, 0.28); transition: border-color 0.15s ease; }
  .composer:focus-within { border-color: #55575d; }
  .composer-top { height: 42px; padding: 0 11px 0 13px; border-bottom: 1px solid #292b30; display: flex; align-items: center; justify-content: space-between; }

  .mode-switch { padding: 2px; border-radius: 7px; display: flex; gap: 2px; background: #202126; }
  .mode-switch button { padding: 5px 9px; border: 0; border-radius: 5px; background: transparent; color: #6f7279; cursor: pointer; font-size: 10.5px; font-weight: 600; }
  .mode-switch button.active { background: #34363c; color: #ecebea; }

  .template-button { display: flex; align-items: center; gap: 6px; padding: 6px 7px; border: 0; background: transparent; color: #777a80; cursor: pointer; font-size: 10.5px; }
  .template-button:hover { color: #c6c4c0; }

  textarea { box-sizing: border-box; width: 100%; min-height: 168px; padding: 19px 20px; border: 0; resize: vertical; outline: 0; background: transparent; color: #f0efec; font-family: inherit; font-size: 14.5px; line-height: 1.55; }
  textarea::placeholder { color: #55585e; }

  .style-row { padding: 9px 12px; border-top: 1px solid #292b30; display: flex; align-items: flex-start; gap: 9px; }
  .style-row > span { padding-top: 5px; color: #666970; font-size: 9px; font-weight: 700; letter-spacing: 0.5px; text-transform: uppercase; }
  .style-row div { display: flex; flex-wrap: wrap; gap: 4px; }
  .style-row button { padding: 4px 7px; border: 1px solid #303238; border-radius: 5px; background: transparent; color: #73767d; cursor: pointer; font-size: 9.5px; }
  .style-row button.active { border-color: #5b5d62; background: #303238; color: #e4e2de; }

  .composer-controls { min-height: 54px; padding: 9px 10px; border-top: 1px solid #2c2e34; display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .control-group { min-width: 0; display: flex; align-items: center; gap: 6px; }

  .target-select { position: relative; max-width: 230px; padding: 5px 8px; border-radius: 7px; display: flex; align-items: center; gap: 7px; color: #6f7279; cursor: pointer; }
  .target-select:hover { background: #202126; }
  .target-select span { font-size: 9px; }
  .target-select strong { overflow: hidden; color: #bab8b4; font-size: 10.5px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
  .target-select svg { flex-shrink: 0; }
  .target-select select { position: absolute; inset: 0; width: 100%; opacity: 0; cursor: pointer; }

  .memory-toggle { padding: 6px 8px; border-radius: 7px; color: #686b72; cursor: pointer; font-size: 9.5px; }
  .memory-toggle:hover { background: #202126; }
  .memory-toggle.on { background: rgba(214, 211, 209, 0.08); color: #c6c4c0; }
  .memory-toggle input { position: absolute; opacity: 0; pointer-events: none; }
  .char-count { color: #51545a; font-size: 9px; font-variant-numeric: tabular-nums; }

  .enhance-button { min-width: 116px; padding: 9px 14px; border: 0; border-radius: 8px; display: flex; align-items: center; justify-content: center; gap: 7px; background: #e4e1dc; color: #171719; cursor: pointer; font-size: 11.5px; font-weight: 700; transition: 0.12s ease; }
  .enhance-button:hover:not(:disabled) { background: #f5f3ef; }
  .enhance-button:active:not(:disabled) { transform: scale(0.98); }
  .enhance-button:disabled { opacity: 0.35; cursor: not-allowed; }
  .enhance-button kbd { color: #77746f; font-size: 9px; font-weight: 600; }
  .spinner { width: 12px; height: 12px; border: 2px solid rgba(23, 23, 25, 0.25); border-top-color: #171719; border-radius: 50%; animation: spin 0.55s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .hotkey-hint { color: #686b72; text-align: center; font-size: 10px; }
  .hotkey-hint kbd { margin: 0 3px; padding: 2px 5px; border: 1px solid #35373c; border-radius: 5px; background: #1d1f23; color: #aaa8a4; font-size: 9px; }

  .readiness { padding: 9px 4px; border-top: 1px solid #25272c; border-bottom: 1px solid #25272c; display: flex; align-items: center; justify-content: center; flex-wrap: wrap; gap: 6px; color: #696c72; font-size: 9.5px; }
  .readiness strong { color: #98b8a3; font-weight: 600; }
  .readiness.problem strong { color: #c4a46b; }
  .status-dot { width: 5px; height: 5px; border-radius: 50%; background: #78a989; }
  .problem .status-dot { background: #c4a46b; }
  .separator { color: #3f4248; }
  .readiness button { margin-left: 4px; border: 0; background: transparent; color: #95979d; cursor: pointer; font-size: 9.5px; text-decoration: underline; text-underline-offset: 2px; }
  .readiness button:hover { color: #d0ceca; }

  .error-alert { border-color: rgba(184, 92, 92, 0.22); background: rgba(184, 92, 92, 0.09); color: #d08c8c; font-size: 11.5px; line-height: 1.4; }
  .error-alert button { border-color: rgba(184, 92, 92, 0.3); color: #d08c8c; }

  .result { border: 1px solid #2d2f34; border-radius: 12px; overflow: hidden; background: #17181c; animation: slide-up 0.2s ease; }
  @keyframes slide-up { from { opacity: 0; transform: translateY(5px); } }
  .result header { padding: 9px 12px; border-bottom: 1px solid #292b30; display: flex; align-items: center; justify-content: space-between; color: #8b8e94; font-size: 10.5px; }
  .result header div { display: flex; align-items: center; gap: 7px; }
  .result-dot { width: 6px; height: 6px; border-radius: 50%; background: #d6d3d1; }
  .result em { padding: 2px 6px; border-radius: 999px; background: rgba(196, 164, 107, 0.1); color: #c4a46b; font-size: 9px; font-style: normal; }
  .result header button { padding: 5px 8px; border: 1px solid #36383e; border-radius: 6px; display: flex; align-items: center; gap: 5px; background: transparent; color: #999b9f; cursor: pointer; font-size: 10px; }
  .result header button:hover,
  .result header button.copied { background: #24262b; color: #e0dedb; }
  .result pre { max-height: 360px; overflow-y: auto; padding: 15px; white-space: pre-wrap; word-wrap: break-word; color: #d8d6d2; font-family: "SF Mono", "Fira Code", ui-monospace, monospace; font-size: 11.5px; line-height: 1.65; }

  .recents { margin-top: 18px; }
  .recents > header { padding: 0 3px 8px; display: flex; align-items: center; justify-content: space-between; }
  .recents > header strong { color: #989aa0; font-size: 10px; font-weight: 600; }
  .recents > header button { border: 0; background: transparent; color: #6f7279; cursor: pointer; font-size: 9.5px; }
  .recents > header button:hover { color: #aaa; }
  .recent-list { border-bottom: 1px solid #27292e; }
  .recent-item { width: 100%; padding: 9px 7px; border: 0; border-top: 1px solid #27292e; display: grid; grid-template-columns: 28px minmax(0, 1fr) 70px 18px; align-items: center; gap: 9px; background: transparent; color: inherit; cursor: pointer; text-align: left; }
  .recent-item:hover { background: rgba(255, 255, 255, 0.018); }
  .target-mark { width: 26px; height: 26px; border-radius: 6px; display: grid; place-items: center; background: #24262b; color: #aaa8a4; font-size: 8px; font-weight: 750; }
  .recent-prompt { overflow: hidden; color: #b6b4b0; font-size: 10.5px; text-overflow: ellipsis; white-space: nowrap; }
  .recent-item small { color: #61646b; font-size: 9px; text-align: right; }
  .recent-item svg { color: #676a70; }

  @media (max-width: 580px) {
    .intro h1 { font-size: 24px; }
    .composer-controls { align-items: stretch; flex-direction: column; }
    .control-group { justify-content: space-between; }
    .enhance-button { width: 100%; }
    .readiness span:nth-of-type(n + 4) { display: none; }
    .recent-item { grid-template-columns: 28px minmax(0, 1fr) 18px; }
    .recent-item small { display: none; }
  }
</style>
