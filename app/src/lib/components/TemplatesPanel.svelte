<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface TemplateField {
    name: string;
    label: string;
    placeholder: string;
    required: boolean;
  }

  interface Template {
    id: string;
    name: string;
    category: string;
    platform: string;
    description: string;
    trending: boolean;
    fields: TemplateField[];
    template: string;
  }

  let templates = $state<Template[]>([]);
  let selectedTemplate = $state<Template | null>(null);
  let fieldValues = $state<Record<string, string>>({});
  let generatedPrompt = $state("");
  let filter = $state<"all" | "Image" | "Text">("all");
  let search = $state("");
  let copied = $state(false);

  async function loadTemplates() {
    try {
      templates = await invoke<Template[]>("list_templates");
    } catch {
      templates = [];
    }
  }

  $effect(() => { loadTemplates(); });

  let filteredTemplates = $derived(
    templates.filter((t) => {
      if (filter !== "all" && t.category !== filter) return false;
      if (search.trim()) {
        const q = search.toLowerCase();
        return t.name.toLowerCase().includes(q) || t.description.toLowerCase().includes(q) || t.id.toLowerCase().includes(q);
      }
      return true;
    })
  );

  function selectTemplate(t: Template) {
    selectedTemplate = t;
    fieldValues = {};
    generatedPrompt = "";
  }

  async function generate() {
    if (!selectedTemplate) return;
    try {
      generatedPrompt = await invoke<string>("apply_template", {
        templateId: selectedTemplate.id,
        fields: fieldValues,
      });
    } catch (e: any) {
      generatedPrompt = `Error: ${e}`;
    }
  }

  async function copyToClipboard() {
    if (!generatedPrompt) return;
    try {
      await invoke("copy_to_clipboard", { text: generatedPrompt });
    } catch {
      await navigator.clipboard.writeText(generatedPrompt);
    }
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function deselect() {
    selectedTemplate = null;
    generatedPrompt = "";
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Templates</h1>
    <p class="subtitle">Pre-built prompt recipes for common tasks</p>
  </div>

  <div class="toolbar">
    <div class="filter-chips">
      <button class="chip" class:active={filter === "all"} onclick={() => (filter = "all")}>All</button>
      <button class="chip" class:active={filter === "Text"} onclick={() => (filter = "Text")}>Text</button>
      <button class="chip" class:active={filter === "Image"} onclick={() => (filter = "Image")}>Image</button>
    </div>
    <div class="search-wrap">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input class="search" type="text" bind:value={search} placeholder="Search..." />
    </div>
  </div>

  {#if filteredTemplates.length === 0}
    <div class="empty">
      <p>{search.trim() ? "No matches found" : "No templates loaded"}</p>
    </div>
  {:else}
    <div class="grid">
      {#each filteredTemplates as t}
        <button
          class="card"
          class:active={selectedTemplate?.id === t.id}
          onclick={() => selectTemplate(t)}
        >
          <div class="card-badges">
            <span class="badge" class:img={t.category === "Image"}>
              {t.category === "Image" ? "IMG" : "TXT"}
            </span>
            {#if t.trending}
              <span class="badge trending">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
              </span>
            {/if}
          </div>
          <span class="card-title">{t.name}</span>
          <span class="card-desc">{t.description}</span>
          <span class="card-meta">{t.platform}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if selectedTemplate}
    <div class="detail">
      <div class="detail-top">
        <div>
          <h3>{selectedTemplate.name}</h3>
          <p class="detail-desc">{selectedTemplate.description}</p>
        </div>
        <button class="btn-ghost-sm" onclick={deselect} aria-label="Close">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>

      {#each selectedTemplate.fields as field}
        <div class="field">
          <label for={field.name}>
            {field.label}
            {#if field.required}<span class="req">*</span>{/if}
          </label>
          <input
            id={field.name}
            type="text"
            bind:value={fieldValues[field.name]}
            placeholder={field.placeholder}
          />
        </div>
      {/each}

      <div class="detail-actions">
        <button class="btn-primary" onclick={generate}>Generate</button>
        {#if generatedPrompt}
          <button class="btn-ghost" class:copied onclick={copyToClipboard}>
            {copied ? "Copied" : "Copy"}
          </button>
        {/if}
      </div>

      {#if generatedPrompt}
        <pre class="output">{generatedPrompt}</pre>
      {/if}
    </div>
  {/if}

  <div class="footer-count">{filteredTemplates.length} template{filteredTemplates.length !== 1 ? 's' : ''}</div>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
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

  /* ── Toolbar ──────────────────────── */

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

  /* ── Empty ────────────────────────── */

  .empty {
    text-align: center;
    padding: 48px 0;
    color: #3f3f46;
    font-size: 13px;
  }

  /* ── Grid ─────────────────────────── */

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 8px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 13px 14px;
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
    color: #e4e4e7;
    transition: all 0.12s ease;
  }

  .card:hover {
    border-color: #27272a;
    background: #18181b;
  }

  .card.active {
    border-color: rgba(16, 185, 129, 0.4);
    background: rgba(16, 185, 129, 0.04);
  }

  .card-badges {
    display: flex;
    gap: 5px;
    align-items: center;
  }

  .badge {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.5px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(16, 185, 129, 0.1);
    color: #34d399;
  }

  .badge.img {
    background: rgba(59, 130, 246, 0.1);
    color: #60a5fa;
  }

  .badge.trending {
    background: rgba(251, 191, 36, 0.1);
    color: #fbbf24;
    display: flex;
    align-items: center;
    padding: 2px 5px;
  }

  .card-title {
    font-size: 13px;
    font-weight: 600;
    color: #e4e4e7;
  }

  .card-desc {
    font-size: 11.5px;
    color: #52525b;
    line-height: 1.4;
  }

  .card-meta {
    font-size: 10px;
    color: #3f3f46;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-top: auto;
    padding-top: 4px;
  }

  /* ── Detail pane ──────────────────── */

  .detail {
    background: #0f0f12;
    border: 1px solid #1a1a1e;
    border-radius: 12px;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    animation: slideUp 0.2s ease;
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .detail-top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }

  .detail-top h3 {
    font-size: 15px;
    font-weight: 650;
    color: #fafafa;
    letter-spacing: -0.2px;
  }

  .detail-desc {
    font-size: 12.5px;
    color: #52525b;
    margin-top: 2px;
  }

  .btn-ghost-sm {
    padding: 4px;
    background: transparent;
    border: 1px solid #1a1a1e;
    color: #52525b;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.12s ease;
    flex-shrink: 0;
  }

  .btn-ghost-sm:hover {
    background: #18181b;
    color: #a1a1aa;
    border-color: #27272a;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field label {
    font-size: 12px;
    color: #71717a;
    font-weight: 550;
  }

  .req { color: #ef4444; }

  .field input {
    padding: 8px 12px;
    background: #18181b;
    border: 1px solid #27272a;
    color: #e4e4e7;
    border-radius: 8px;
    font-size: 13px;
    font-family: inherit;
  }

  .field input::placeholder { color: #3f3f46; }

  .field input:focus {
    outline: none;
    border-color: #10b981;
    box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.08);
  }

  .detail-actions {
    display: flex;
    gap: 6px;
    padding-top: 4px;
  }

  .btn-primary {
    padding: 8px 18px;
    background: #10b981;
    color: #022c22;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 600;
    transition: all 0.12s ease;
  }

  .btn-primary:hover { background: #34d399; }

  .btn-ghost {
    padding: 8px 14px;
    background: transparent;
    color: #71717a;
    border: 1px solid #27272a;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12.5px;
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
    background: rgba(16, 185, 129, 0.06);
  }

  .output {
    padding: 14px;
    margin: 0;
    background: #18181b;
    border: 1px solid #27272a;
    border-radius: 8px;
    white-space: pre-wrap;
    word-wrap: break-word;
    font-family: "SF Mono", "Fira Code", ui-monospace, monospace;
    font-size: 12px;
    line-height: 1.6;
    color: #d4d4d8;
    max-height: 240px;
    overflow-y: auto;
  }

  .footer-count {
    text-align: center;
    font-size: 11px;
    color: #3f3f46;
  }
</style>
