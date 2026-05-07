<script lang="ts">
  import { onMount, tick } from "svelte";
  import { api, type NoteMeta } from "../lib/api";
  import { showToast, settings, notesState } from "../lib/store";
  import { get } from "svelte/store";

  let notes: NoteMeta[] = $state([]);
  let activeId: string | null = $state(get(notesState).activeId);
  let content: string = $state(get(notesState).content);
  let query: string = $state("");
  let passwordPrompt = $state(false);
  let pendingPassword = $state("");
  let saveTimer: number | null = null;
  let editingId: string | null = $state(null);
  let editingText: string = $state("");

  $effect(() => {
    notesState.set({ activeId, content });
  });

  const filtered = $derived(
    notes.filter(n => !query || n.name.toLowerCase().includes(query.toLowerCase()) || content.toLowerCase().includes(query.toLowerCase()))
  );

  async function refresh(selectId?: string) {
    notes = await api.listNotes();
    if (selectId) {
      const n = notes.find(x => x.id === selectId);
      if (n) await select(n);
    }
  }

  async function select(n: NoteMeta) {
    activeId = n.id;
    content = "";
    passwordPrompt = false;
    if (n.encrypted) {
      passwordPrompt = true;
      return;
    }
    try {
      content = await api.readNote(n.id);
    } catch (e) {
      showToast(`读取失败: ${e}`);
    }
  }

  async function unlockNote() {
    if (!activeId) return;
    try {
      content = await api.readNote(activeId, pendingPassword);
      passwordPrompt = false;
      pendingPassword = "";
    } catch (e) {
      showToast("密码错误");
    }
  }

  async function newNote() {
    const name = `笔记-${new Date().toISOString().slice(0, 10)}-${Date.now().toString(36).slice(-4)}`;
    const n = await api.createNote(name);
    await refresh(n.id);
  }

  async function delNote() {
    if (!activeId) return;
    await api.deleteNote(activeId);
    activeId = null;
    content = "";
    await refresh();
  }

  async function startRename() {
    if (!activeId) return;
    const n = notes.find(x => x.id === activeId);
    if (!n) return;
    editingId = activeId;
    editingText = n.name;
    await tick();
    const el = document.getElementById("note-rename-input") as HTMLInputElement | null;
    el?.focus();
    el?.select();
  }

  async function commitRename() {
    if (!editingId) return;
    const id = editingId;
    const name = editingText.trim();
    editingId = null;
    editingText = "";
    if (!name) return;
    const orig = notes.find(x => x.id === id);
    if (orig && orig.name === name) return;
    try {
      const nn = await api.renameNote(id, name);
      activeId = nn.id;
      notes = await api.listNotes();
    } catch (e) {
      showToast(`重命名失败: ${e}`);
    }
  }

  function cancelRename() {
    editingId = null;
    editingText = "";
  }

  function onInput() {
    if (!activeId) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = window.setTimeout(async () => {
      if (!activeId) return;
      try {
        await api.saveNote(activeId, content);
      } catch (e) {
        showToast(`保存失败: ${e}`);
      }
    }, 600);
  }

  onMount(() => {
    (async () => {
      notes = await api.listNotes();
      // Restore content if we have an activeId from a previous mount
      if (activeId) {
        const n = notes.find(x => x.id === activeId);
        if (!n) {
          activeId = null;
          content = "";
        } else if (n.encrypted) {
          if (!content) passwordPrompt = true;
        } else if (!content) {
          try { content = await api.readNote(n.id); } catch (e) { showToast(`读取失败: ${e}`); }
        }
      }
    })();
    const h = (e: any) => refresh(e.detail);
    window.addEventListener("notes-refresh", h);
    return () => window.removeEventListener("notes-refresh", h);
  });
</script>

{#if $settings.showSidebar}
<div class="sidebar">
  <div class="sidebar-header">
    <button onclick={newNote} title="新建">+ 新建</button>
    <button onclick={startRename} title="重命名" disabled={!activeId}>✎</button>
    <button onclick={delNote} title="删除" disabled={!activeId}>✕</button>
  </div>
  <div class="search">
    <input placeholder="搜索..." bind:value={query} />
  </div>
  <div class="note-list">
    {#each filtered as n (n.id)}
      <div class="note-item" class:active={n.id === activeId} onclick={() => editingId !== n.id && select(n)} ondblclick={() => { activeId = n.id; startRename(); }} role="button" tabindex="0">
        {#if editingId === n.id}
          <input
            id="note-rename-input"
            class="rename-input"
            bind:value={editingText}
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => { if (e.key === 'Enter') commitRename(); else if (e.key === 'Escape') cancelRename(); }}
            onblur={commitRename}
          />
        {:else}
          <span class="name">{n.name}</span>
          {#if n.encrypted}<span class="lock">🔒</span>{/if}
        {/if}
      </div>
    {/each}
    {#if filtered.length === 0}
      <div style="padding:12px;color:var(--fg-dim);font-size:12px;text-align:center;">暂无笔记</div>
    {/if}
  </div>
</div>
{/if}
<div class="editor">
  {#if passwordPrompt}
    <div class="empty" style="flex-direction:column;gap:10px;">
      <div>该笔记已加密</div>
      <input type="password" bind:value={pendingPassword} placeholder="输入密码" style="padding:6px;border-radius:4px;border:1px solid var(--border);background:rgba(0,0,0,0.3);color:var(--fg);" onkeydown={(e) => e.key === 'Enter' && unlockNote()} />
      <button onclick={unlockNote} style="padding:6px 14px;background:var(--accent);color:#fff;border:none;border-radius:4px;cursor:pointer;">解锁</button>
    </div>
  {:else if activeId}
    <textarea bind:value={content} oninput={onInput} spellcheck="false" placeholder="在此书写..."></textarea>
  {:else}
    <div class="empty">选择或新建一条笔记，或拖拽 .txt / .md 文件到窗口</div>
  {/if}
</div>
