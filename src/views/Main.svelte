<script lang="ts">
  import { onMount } from "svelte";
  import { api, type NoteMeta } from "../lib/api";
  import { showToast } from "../lib/store";

  let notes: NoteMeta[] = $state([]);
  let activeId: string | null = $state(null);
  let content: string = $state("");
  let query: string = $state("");
  let passwordPrompt = $state(false);
  let pendingPassword = $state("");
  let saveTimer: number | null = null;

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

  async function renameNote() {
    if (!activeId) return;
    const n = notes.find(x => x.id === activeId);
    if (!n) return;
    const name = prompt("新名称", n.name);
    if (!name) return;
    const nn = await api.renameNote(activeId, name);
    await refresh(nn.id);
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
    refresh();
    const h = (e: any) => refresh(e.detail);
    window.addEventListener("notes-refresh", h);
    return () => window.removeEventListener("notes-refresh", h);
  });
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <button onclick={newNote} title="新建">+ 新建</button>
    <button onclick={renameNote} title="重命名" disabled={!activeId}>✎</button>
    <button onclick={delNote} title="删除" disabled={!activeId}>✕</button>
  </div>
  <div class="search">
    <input placeholder="搜索..." bind:value={query} />
  </div>
  <div class="note-list">
    {#each filtered as n (n.id)}
      <div class="note-item" class:active={n.id === activeId} onclick={() => select(n)} role="button" tabindex="0">
        <span class="name">{n.name}</span>
        {#if n.encrypted}<span class="lock">🔒</span>{/if}
      </div>
    {/each}
    {#if filtered.length === 0}
      <div style="padding:12px;color:var(--fg-dim);font-size:12px;text-align:center;">暂无笔记</div>
    {/if}
  </div>
</div>
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
