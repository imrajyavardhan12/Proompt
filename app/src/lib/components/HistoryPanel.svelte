<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface PromptRoutingMetadata {
    source: string;
    confidence: string;
    reason: string;
  }

  interface PromptHistoryRecord {
    id: string;
    original_prompt: string;
    enhanced_prompt: string;
    enhancement_type: "text" | "image";
    platform: string;
    provider: string;
    model: string;
    created_at_ms: number;
    favorite: boolean;
    routing?: PromptRoutingMetadata | null;
  }

  interface HistoryReuseDraft {
    id: string;
    prompt: string;
    platform: string;
    mode: "text" | "image";
  }

  let { onReuse = () => {} } = $props<{ onReuse?: (draft: HistoryReuseDraft) => void }>();

  let records = $state<PromptHistoryRecord[]>([]);
  let search = $state("");
  let filter = $state<"all" | "favorites">("all");
  let loading = $state(true);
  let error = $state("");
  let copiedId = $state<string | null>(null);

  let filteredRecords = $derived(
    records.filter((record) => {
      if (filter === "favorites" && !record.favorite) return false;
      const query = search.trim().toLowerCase();
      if (!query) return true;
      return [
        record.original_prompt,
        record.enhanced_prompt,
        record.platform,
        record.provider,
        record.model,
        record.routing?.source ?? "",
        record.routing?.confidence ?? "",
        record.routing?.reason ?? "",
      ].some((value) => value.toLowerCase().includes(query));
    })
  );

  $effect(() => {
    loadHistory();
  });

  async function loadHistory() {
    loading = true;
    error = "";
    try {
      records = await invoke<PromptHistoryRecord[]>("list_history", {
        limit: 200,
        favoritesOnly: false,
      });
    } catch (e: any) {
      error = e?.toString?.() ?? `${e}`;
      records = [];
    } finally {
      loading = false;
    }
  }

  async function toggleFavorite(record: PromptHistoryRecord) {
    try {
      const updated = await invoke<PromptHistoryRecord>("set_history_favorite", {
        id: record.id,
        favorite: !record.favorite,
      });
      records = records.map((item) => item.id === updated.id ? updated : item);
    } catch (e: any) {
      error = e?.toString?.() ?? `${e}`;
    }
  }

  async function deleteRecord(record: PromptHistoryRecord) {
    try {
      await invoke("delete_history_record", { id: record.id });
      records = records.filter((item) => item.id !== record.id);
    } catch (e: any) {
      error = e?.toString?.() ?? `${e}`;
    }
  }

  async function clearHistory() {
    if (!records.length || !confirm("Clear all prompt history?")) return;
    try {
      await invoke("clear_prompt_history");
      records = [];
    } catch (e: any) {
      error = e?.toString?.() ?? `${e}`;
    }
  }

  async function copyText(id: string, text: string) {
    try {
      await invoke("copy_to_clipboard", { text });
    } catch {
      await navigator.clipboard.writeText(text);
    }
    copiedId = id;
    setTimeout(() => {
      if (copiedId === id) copiedId = null;
    }, 1600);
  }

  function reuse(record: PromptHistoryRecord) {
    onReuse({
      id: record.id,
      prompt: record.original_prompt,
      platform: record.platform,
      mode: record.enhancement_type,
    });
  }

  function formatDate(ms: number) {
    if (!ms) return "Unknown time";
    return new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(new Date(ms));
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
      midjourney: "Midjourney",
      dalle: "DALL-E",
      stablediffusion: "Stable Diffusion",
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

  function truncate(text: string, max = 220) {
    return text.length > max ? `${text.slice(0, max)}…` : text;
  }
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>History</h1>
      <p class="subtitle">Reuse, copy, and favorite your previous prompt enhancements</p>
    </div>
    <button class="btn-ghost" onclick={loadHistory} disabled={loading}>Refresh</button>
  </div>

  <div class="toolbar">
    <div class="filter-chips">
      <button class="chip" class:active={filter === "all"} onclick={() => (filter = "all")}>All</button>
      <button class="chip" class:active={filter === "favorites"} onclick={() => (filter = "favorites")}>Favorites</button>
    </div>
    <div class="search-wrap">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input class="search" type="text" bind:value={search} placeholder="Search history..." />
    </div>
  </div>

  {#if error}
    <div class="alert">{error}</div>
  {/if}

  {#if loading}
    <div class="empty">Loading history...</div>
  {:else if filteredRecords.length === 0}
    <div class="empty">
      <div class="empty-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 3v5h5"/><path d="M3.05 13A9 9 0 1 0 5 5.3L3 8"/><path d="M12 7v5l3 2"/></svg>
      </div>
      <p>{records.length ? "No matching prompts" : "No prompt history yet"}</p>
      <span>{records.length ? "Try a different search." : "Enhance a prompt from the app, CLI, or quick hotkey to save it locally on this device."}</span>
    </div>
  {:else}
    <div class="list">
      {#each filteredRecords as record}
        <article class="history-card">
          <div class="card-top">
            <div class="meta">
              <span class="pill" class:image={record.enhancement_type === "image"}>{record.enhancement_type}</span>
              <span>{platformLabel(record.platform)}</span>
              <span>·</span>
              <span>{record.provider} / {record.model}</span>
              <span>·</span>
              <span>{formatDate(record.created_at_ms)}</span>
            </div>
            <button class="icon-btn" class:favorite={record.favorite} onclick={() => toggleFavorite(record)} aria-label="Toggle favorite">
              <svg width="15" height="15" viewBox="0 0 24 24" fill={record.favorite ? "currentColor" : "none"} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
            </button>
          </div>

          {#if record.routing}
            <div class="route-line">
              <span class="route-badge">Quick Enhance</span>
              <span>{routingSourceLabel(record.routing.source)} · {routingConfidenceLabel(record.routing.confidence)} · {record.routing.reason}</span>
            </div>
          {/if}

          <div class="prompt-block">
            <span class="block-label">Original</span>
            <p>{truncate(record.original_prompt)}</p>
          </div>
          <div class="prompt-block enhanced">
            <span class="block-label">Enhanced</span>
            <p>{truncate(record.enhanced_prompt)}</p>
          </div>

          <div class="actions">
            <button class="btn-secondary" onclick={() => reuse(record)}>Reuse original</button>
            <button class="btn-secondary" onclick={() => copyText(`${record.id}-enhanced`, record.enhanced_prompt)}>
              {copiedId === `${record.id}-enhanced` ? "Copied" : "Copy enhanced"}
            </button>
            <button class="btn-ghost danger" onclick={() => deleteRecord(record)}>Delete</button>
          </div>
        </article>
      {/each}
    </div>

    <div class="footer-row">
      <span>{filteredRecords.length} prompt{filteredRecords.length !== 1 ? "s" : ""}</span>
      <button class="btn-ghost danger" onclick={clearHistory}>Clear all</button>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  h1 {
    font-size: 22px;
    font-weight: 650;
    color: #fafafa;
    letter-spacing: -0.5px;
  }

  .subtitle {
    margin-top: 4px;
    font-size: 13px;
    color: #52525b;
    font-weight: 450;
  }

  .toolbar {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .filter-chips {
    display: flex;
    gap: 3px;
    background: #18181b;
    padding: 3px;
    border-radius: 8px;
  }

  .chip {
    padding: 5px 12px;
    border: none;
    background: transparent;
    color: #52525b;
    cursor: pointer;
    font-size: 12px;
    font-weight: 550;
    border-radius: 6px;
    transition: all 0.12s ease;
  }

  .chip:hover { color: #a1a1aa; }

  .chip.active {
    background: #27272a;
    color: #fafafa;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
  }

  .search-wrap {
    flex: 1;
    position: relative;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: #3f3f46;
    pointer-events: none;
  }

  .search {
    width: 100%;
    padding: 7px 10px 7px 30px;
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    color: #e4e4e7;
    border-radius: 8px;
    font-size: 12.5px;
    font-family: inherit;
  }

  .search::placeholder { color: #3f3f46; }
  .search:focus {
    outline: none;
    border-color: #27272a;
  }

  .alert {
    padding: 10px 12px;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.18);
    color: #fca5a5;
    border-radius: 10px;
    font-size: 12.5px;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 260px;
    text-align: center;
    color: #52525b;
    font-size: 13px;
  }

  .empty-icon {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    color: #71717a;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty p {
    color: #a1a1aa;
    font-weight: 600;
  }

  .empty span {
    max-width: 360px;
    line-height: 1.45;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .history-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    border-radius: 12px;
  }

  .card-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    color: #52525b;
    font-size: 11px;
  }

  .pill {
    padding: 2px 6px;
    border-radius: 5px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-size: 9px;
    font-weight: 700;
    background: rgba(16, 185, 129, 0.1);
    color: #34d399;
  }

  .pill.image {
    background: rgba(59, 130, 246, 0.1);
    color: #60a5fa;
  }

  .icon-btn {
    width: 28px;
    height: 28px;
    border-radius: 7px;
    border: 1px solid #27272a;
    background: #18181b;
    color: #52525b;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.12s ease;
  }

  .icon-btn:hover {
    color: #a1a1aa;
    border-color: #3f3f46;
  }

  .icon-btn.favorite {
    color: #fbbf24;
    border-color: rgba(251, 191, 36, 0.25);
    background: rgba(251, 191, 36, 0.08);
  }

  .route-line {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 10px;
    background: rgba(16, 185, 129, 0.06);
    border: 1px solid rgba(16, 185, 129, 0.12);
    border-radius: 9px;
    color: #86efac;
    font-size: 11px;
    line-height: 1.4;
  }

  .route-badge {
    padding: 2px 6px;
    border-radius: 5px;
    background: rgba(16, 185, 129, 0.12);
    color: #34d399;
    text-transform: uppercase;
    letter-spacing: 0.45px;
    font-size: 9px;
    font-weight: 700;
  }

  .prompt-block {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .block-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: #52525b;
    font-weight: 700;
  }

  .prompt-block p {
    color: #d4d4d8;
    font-size: 12.5px;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .prompt-block.enhanced p {
    color: #a1a1aa;
  }

  .actions,
  .footer-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .actions {
    justify-content: flex-start;
  }

  .footer-row {
    color: #3f3f46;
    font-size: 12px;
  }

  .btn-secondary,
  .btn-ghost {
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    transition: all 0.12s ease;
  }

  .btn-secondary {
    padding: 7px 10px;
    background: #18181b;
    color: #d4d4d8;
    border: 1px solid #27272a;
  }

  .btn-secondary:hover {
    background: #27272a;
    color: #fafafa;
  }

  .btn-ghost {
    padding: 7px 10px;
    background: transparent;
    color: #71717a;
  }

  .btn-ghost:hover {
    color: #d4d4d8;
    background: #18181b;
  }

  .btn-ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .danger:hover {
    color: #fca5a5;
    background: rgba(239, 68, 68, 0.08);
  }
</style>
