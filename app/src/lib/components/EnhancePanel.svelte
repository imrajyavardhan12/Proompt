<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let prompt = $state("");
  let enhancedPrompt = $state("");
  let platform = $state("openai");
  let mode = $state<"text" | "image">("text");
  let isLoading = $state(false);
  let includeMemory = $state(false);
  let error = $state("");
  let copied = $state(false);
  let selectedStyles = $state<Set<string>>(new Set());

  const textPlatforms = [
    { id: "claude", label: "Claude" },
    { id: "openai", label: "GPT" },
    { id: "gemini", label: "Gemini" },
    { id: "generic", label: "Generic" },
  ];

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

  $effect(() => {
    const ids = platforms.map((p) => p.id);
    if (!ids.includes(platform)) {
      platform = ids[0];
    }
  });

  function toggleStyle(style: string) {
    const next = new Set(selectedStyles);
    if (next.has(style)) next.delete(style);
    else next.add(style);
    selectedStyles = next;
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleEnhance();
    }
  }

  async function handleEnhance() {
    if (!prompt.trim() || isLoading) return;
    isLoading = true;
    error = "";
    enhancedPrompt = "";
    try {
      const result = await invoke<string>("enhance_prompt", {
        prompt: prompt,
        platform: platform,
        enhanceType: mode,
        includeMemory: includeMemory,
      });
      enhancedPrompt = result;
    } catch (e: any) {
      error = e.toString();
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

  let charCount = $derived(prompt.length);
</script>

<div class="page">
  <div class="page-header">
    <h1>Enhance</h1>
    <p class="subtitle">Transform rough prompts into optimized ones</p>
  </div>

  <div class="card">
    <div class="card-section">
      <div class="mode-switch">
        <button class:active={mode === "text"} onclick={() => (mode = "text")}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          Text
        </button>
        <button class:active={mode === "image"} onclick={() => (mode = "image")}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg>
          Image
        </button>
      </div>
    </div>

    <div class="divider"></div>

    <div class="card-section">
      <div class="label-row">
        <span class="label">Target platform</span>
      </div>
      <div class="chips">
        {#each platforms as p}
          <button
            class="chip"
            class:active={platform === p.id}
            onclick={() => (platform = p.id)}
          >
            {p.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="divider"></div>

    <div class="card-section">
      <div class="textarea-wrap">
        <textarea
          bind:value={prompt}
          onkeydown={handleKeydown}
          placeholder={mode === "text"
            ? "Type or paste your rough prompt here..."
            : "Describe your image idea..."}
          rows="5"
        ></textarea>
        <div class="textarea-footer">
          <span class="char-count">{charCount}</span>
          <span class="kbd-hint">
            <kbd>{navigator.platform?.includes("Mac") ? "\u2318" : "Ctrl"}</kbd>
            <kbd>Enter</kbd>
          </span>
        </div>
      </div>
    </div>

    {#if mode === "image"}
      <div class="divider"></div>
      <div class="card-section">
        <span class="label">Style hints</span>
        <div class="chips">
          {#each styleOptions as style}
            <button
              class="chip"
              class:active={selectedStyles.has(style)}
              onclick={() => toggleStyle(style)}
            >
              {style}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <div class="divider"></div>

    <div class="card-section card-footer">
      <label class="toggle">
        <div class="toggle-track" class:on={includeMemory}>
          <input type="checkbox" bind:checked={includeMemory} />
          <div class="toggle-thumb"></div>
        </div>
        <span class="toggle-label">SuperMemory</span>
      </label>

      <button
        class="btn-primary"
        onclick={handleEnhance}
        disabled={isLoading || !prompt.trim()}
      >
        {#if isLoading}
          <span class="spinner"></span>
          Enhancing...
        {:else}
          Enhance prompt
        {/if}
      </button>
    </div>
  </div>

  {#if error}
    <div class="alert alert-error">{error}</div>
  {/if}

  {#if enhancedPrompt}
    <div class="result" >
      <div class="result-bar">
        <div class="result-bar-left">
          <div class="result-dot"></div>
          <span>Enhanced for {platforms.find(p => p.id === platform)?.label ?? platform}</span>
        </div>
        <button class="btn-ghost" class:copied onclick={copyToClipboard}>
          {#if copied}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            Copied
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
            Copy
          {/if}
        </button>
      </div>
      <pre class="result-body">{enhancedPrompt}</pre>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 20px;
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

  /* ── Card ─────────────────────────── */

  .card {
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    border-radius: 12px;
    overflow: hidden;
  }

  .card-section {
    padding: 14px 18px;
  }

  .card-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .divider {
    height: 1px;
    background: #1a1a1e;
  }

  /* ── Mode switch ──────────────────── */

  .mode-switch {
    display: inline-flex;
    background: #18181b;
    border-radius: 8px;
    padding: 3px;
    gap: 2px;
  }

  .mode-switch button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border: none;
    background: transparent;
    color: #52525b;
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 550;
    border-radius: 6px;
    transition: all 0.12s ease;
  }

  .mode-switch button.active {
    background: #27272a;
    color: #fafafa;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
  }

  .mode-switch button.active svg {
    color: #10b981;
  }

  /* ── Labels & chips ───────────────── */

  .label-row {
    margin-bottom: 10px;
  }

  .label {
    font-size: 11.5px;
    font-weight: 600;
    color: #52525b;
    text-transform: uppercase;
    letter-spacing: 0.6px;
  }

  .chips {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .chip {
    padding: 5px 12px;
    border: 1px solid #27272a;
    background: transparent;
    color: #71717a;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.12s ease;
  }

  .chip:hover {
    color: #a1a1aa;
    border-color: #3f3f46;
  }

  .chip.active {
    background: rgba(16, 185, 129, 0.1);
    border-color: rgba(16, 185, 129, 0.35);
    color: #34d399;
  }

  /* ── Textarea ─────────────────────── */

  .textarea-wrap {
    position: relative;
  }

  textarea {
    width: 100%;
    padding: 12px 14px;
    padding-bottom: 36px;
    border: 1px solid #27272a;
    background: #18181b;
    color: #e4e4e7;
    border-radius: 10px;
    resize: vertical;
    font-family: inherit;
    font-size: 13.5px;
    line-height: 1.55;
    min-height: 110px;
    transition: border-color 0.12s ease;
  }

  textarea::placeholder {
    color: #3f3f46;
  }

  textarea:focus {
    outline: none;
    border-color: #10b981;
    box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.08);
  }

  .textarea-footer {
    position: absolute;
    bottom: 8px;
    left: 14px;
    right: 14px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .char-count {
    font-size: 10.5px;
    color: #3f3f46;
    font-variant-numeric: tabular-nums;
  }

  .kbd-hint {
    display: flex;
    gap: 3px;
  }

  kbd {
    font-family: inherit;
    font-size: 10px;
    padding: 1px 5px;
    background: #27272a;
    color: #52525b;
    border-radius: 4px;
    border: 1px solid #3f3f46;
    font-weight: 500;
  }

  /* ── Toggle ───────────────────────── */

  .toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .toggle-track {
    position: relative;
    width: 32px;
    height: 18px;
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
    width: 14px;
    height: 14px;
    background: #52525b;
    border-radius: 99px;
    transition: all 0.15s ease;
  }

  .toggle-track.on {
    background: #059669;
  }

  .toggle-track.on .toggle-thumb {
    left: 16px;
    background: #fff;
  }

  .toggle-label {
    font-size: 12.5px;
    color: #71717a;
    font-weight: 500;
  }

  /* ── Buttons ──────────────────────── */

  .btn-primary {
    padding: 8px 20px;
    background: #10b981;
    color: #022c22;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    transition: all 0.12s ease;
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .btn-primary:hover:not(:disabled) {
    background: #34d399;
  }

  .btn-primary:active:not(:disabled) {
    transform: scale(0.98);
  }

  .btn-primary:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .btn-ghost {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    background: transparent;
    color: #71717a;
    border: 1px solid #27272a;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.12s ease;
  }

  .btn-ghost:hover {
    color: #a1a1aa;
    border-color: #3f3f46;
    background: #18181b;
  }

  .btn-ghost.copied {
    color: #34d399;
    border-color: rgba(16, 185, 129, 0.3);
    background: rgba(16, 185, 129, 0.08);
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(2, 44, 34, 0.3);
    border-top-color: #022c22;
    border-radius: 50%;
    animation: spin 0.55s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* ── Error ────────────────────────── */

  .alert-error {
    padding: 10px 14px;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.2);
    color: #fca5a5;
    border-radius: 10px;
    font-size: 12.5px;
    line-height: 1.45;
  }

  /* ── Result ───────────────────────── */

  .result {
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    border-radius: 12px;
    overflow: hidden;
    animation: slideUp 0.25s ease;
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .result-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid #1a1a1e;
  }

  .result-bar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #71717a;
    font-weight: 500;
  }

  .result-dot {
    width: 7px;
    height: 7px;
    background: #10b981;
    border-radius: 50%;
  }

  .result-body {
    padding: 16px;
    margin: 0;
    white-space: pre-wrap;
    word-wrap: break-word;
    font-family: "SF Mono", "Fira Code", "JetBrains Mono", ui-monospace, monospace;
    font-size: 12.5px;
    line-height: 1.7;
    color: #d4d4d8;
    max-height: 380px;
    overflow-y: auto;
    background: transparent;
  }
</style>
