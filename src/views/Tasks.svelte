<script lang="ts">
  import { onMount } from "svelte";
  import { api, type Task } from "../lib/api";
  import { showToast } from "../lib/store";

  let tasks: Task[] = $state([]);
  let input: string = $state("");
  let inputEl: HTMLInputElement | undefined = $state();
  let showEarlier = $state(false);
  let editingId: string | null = $state(null);
  let editingText: string = $state("");

  const pinned = $derived(tasks.filter(t => t.pinned && t.status !== "done").sort((a,b) => a.order - b.order));
  const working = $derived(tasks.filter(t => !t.pinned && t.status === "working").sort((a,b) => a.order - b.order));
  const todos = $derived(tasks.filter(t => !t.pinned && t.status === "todo").sort((a,b) => a.order - b.order));
  const doneToday = $derived(tasks.filter(t => t.status === "done" && isToday(t.completedAt)).sort((a,b) => (b.completedAt ?? 0) - (a.completedAt ?? 0)));
  const doneEarlier = $derived(tasks.filter(t => t.status === "done" && !isToday(t.completedAt)).sort((a,b) => (b.completedAt ?? 0) - (a.completedAt ?? 0)));

  function isToday(ts?: number): boolean {
    if (!ts) return false;
    const d = new Date(ts * 1000);
    const n = new Date();
    return d.getFullYear() === n.getFullYear() && d.getMonth() === n.getMonth() && d.getDate() === n.getDate();
  }

  function fmtTime(ts?: number): string {
    if (!ts) return "";
    const d = new Date(ts * 1000);
    return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  }

  function fmtDate(ts?: number): string {
    if (!ts) return "";
    const d = new Date(ts * 1000);
    const n = new Date();
    const mm = String(d.getMonth() + 1).padStart(2, "0");
    const dd = String(d.getDate()).padStart(2, "0");
    if (d.getFullYear() !== n.getFullYear()) return `${d.getFullYear()}-${mm}-${dd}`;
    return `${mm}-${dd}`;
  }

  async function refresh() {
    try { tasks = await api.listTasks(); } catch (e) { showToast(`载入失败: ${e}`); }
  }

  async function submit(e: KeyboardEvent) {
    if (e.key !== "Enter" || !input.trim()) return;
    try {
      await api.createTask(input);
      input = "";
      await refresh();
    } catch (err) { showToast(`${err}`); }
  }

  async function toggle(t: Task) {
    try { await api.toggleTask(t.id); await refresh(); } catch (e) { showToast(`${e}`); }
  }

  async function togglePin(t: Task) {
    await api.updateTask(t.id, { pinned: !t.pinned });
    await refresh();
  }

  async function setStatus(t: Task, status: Task["status"]) {
    await api.updateTask(t.id, { status });
    await refresh();
  }

  async function del(t: Task) {
    await api.deleteTask(t.id);
    await refresh();
  }

  async function clearDoneToday() {
    for (const t of doneToday) await api.deleteTask(t.id);
    await refresh();
  }

  function startEdit(t: Task) {
    editingId = t.id;
    editingText = t.title;
  }
  async function commitEdit() {
    if (editingId && editingText.trim()) {
      await api.updateTask(editingId, { title: editingText.trim() });
      await refresh();
    }
    editingId = null;
    editingText = "";
  }

  export function focusInput() {
    setTimeout(() => inputEl?.focus(), 30);
  }

  onMount(() => {
    refresh();
  });
</script>

<div class="tasks">
  <div class="task-input-wrap">
    <input
      id="new-task-input"
      bind:this={inputEl}
      bind:value={input}
      onkeydown={submit}
      placeholder="➕ 任务标题... Enter 新建 · ! 前缀=紧急 · &gt; 前缀=Working"
    />
  </div>

  {#if pinned.length}
    <div class="task-group urgent">
      <div class="group-head">🔥 紧急置顶 · {pinned.length}</div>
      {#each pinned as t (t.id)}
        {@render row(t, true)}
      {/each}
    </div>
  {/if}

  {#if working.length}
    <div class="task-group">
      <div class="group-head">▶ Working · {working.length}</div>
      {#each working as t (t.id)}
        {@render row(t, false)}
      {/each}
    </div>
  {/if}

  <div class="task-group">
    <div class="group-head">📋 Todo · {todos.length}</div>
    {#each todos as t (t.id)}
      {@render row(t, false)}
    {/each}
    {#if todos.length === 0 && pinned.length === 0 && working.length === 0 && doneToday.length === 0}
      <div class="empty">暂无任务，在上方输入即可新建 (Ctrl+N)</div>
    {/if}
  </div>

  {#if doneToday.length}
    <div class="task-group done">
      <div class="group-head">
        ✓ 今日完成 · {doneToday.length}
        <button class="mini" onclick={clearDoneToday} title="清空今日完成">🗑</button>
      </div>
      {#each doneToday as t (t.id)}
        {@render row(t, false)}
      {/each}
    </div>
  {/if}

  {#if doneEarlier.length}
    <div class="task-group done earlier">
      <div class="group-head clickable" role="button" tabindex="0" onclick={() => (showEarlier = !showEarlier)}>
        {showEarlier ? "⌃" : "⌄"} 更早完成 · {doneEarlier.length}
      </div>
      {#if showEarlier}
        {#each doneEarlier as t (t.id)}
          {@render row(t, false)}
        {/each}
      {/if}
    </div>
  {/if}
</div>

{#snippet row(t: Task, urgent: boolean)}
  <div class="task-row" class:done={t.status === "done"} class:urgent>
    <button
      class="check"
      class:checked={t.status === "done"}
      onclick={() => toggle(t)}
      title={t.status === "done" ? "取消完成" : "标记完成"}
    >{t.status === "done" ? "☑" : "☐"}</button>

    {#if editingId === t.id}
      <input
        class="title-edit"
        bind:value={editingText}
        onkeydown={(e) => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') { editingId = null; } }}
        onblur={commitEdit}
        autofocus
      />
    {:else}
      <span class="title" ondblclick={() => startEdit(t)}>{t.title}</span>
    {/if}

    <span class="time" title="创建于 {fmtDate(t.createdAt)} {fmtTime(t.createdAt)}">📅 {fmtDate(t.createdAt)}</span>
    {#if t.status === "done"}
      <span class="time" title="完成于 {fmtDate(t.completedAt)} {fmtTime(t.completedAt)}">✓ {fmtDate(t.completedAt)} {fmtTime(t.completedAt)}</span>
    {/if}

    <span class="actions">
      {#if t.status !== "done"}
        {#if t.status === "working"}
          <button class="mini" onclick={() => setStatus(t, "todo")} title="暂停 (回 Todo)">⏸</button>
        {:else}
          <button class="mini" onclick={() => setStatus(t, "working")} title="开始 (Working)">▶</button>
        {/if}
        <button class="mini" onclick={() => togglePin(t)} title={t.pinned ? "取消置顶" : "置顶紧急"}>📌</button>
      {/if}
      <button class="mini danger" onclick={() => del(t)} title="删除">✕</button>
    </span>
  </div>
{/snippet}
