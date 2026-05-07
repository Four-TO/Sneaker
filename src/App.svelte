<script lang="ts">
  import { onMount } from "svelte";
  import { settings, locked, toast, loadSettings, showToast, scheduleSave } from "./lib/store";
  import { api } from "./lib/api";
  import TitleBar from "./components/TitleBar.svelte";
  import BottomBar from "./components/BottomBar.svelte";
  import Main from "./views/Main.svelte";
  import Tasks from "./views/Tasks.svelte";
  import Settings from "./views/Settings.svelte";
  import Lock from "./views/Lock.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let view: "main" | "tasks" | "settings" = $state("tasks");

  onMount(async () => {
    await loadSettings();
    document.documentElement.setAttribute("data-theme", $settings.theme);
    document.documentElement.style.setProperty("--bg-solid", $settings.bgColor);
    await api.applyWindow($settings);
    const failed = await api.applyHotkeys($settings);
    if (failed.length) {
      const list = failed.map(f => { const p = f.split("|"); return `${p[1]}(${p[2]})`; }).join("，");
      showToast(`热键冲突：${list}`, 4000);
    }

    const l = await api.isLocked();
    locked.set(l);

    // Ctrl + wheel → opacity
    window.addEventListener("wheel", async (e) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      const step = e.deltaY < 0 ? 0.03 : -0.03;
      const next = Math.max(0.2, Math.min(1, +($settings.opacity + step).toFixed(2)));
      settings.update((s) => ({ ...s, opacity: next }));
      await api.setOpacity(next);
      scheduleSave();
      showToast(`透明度 ${Math.round(next * 100)}%`);
    }, { passive: false });

    // Drag-drop open
    const webview = getCurrentWebview();
    await webview.onDragDropEvent(async (event) => {
      if (event.payload.type === "drop") {
        for (const p of event.payload.paths) {
          if (/\.(txt|md|markdown|log|json|yaml|yml|ini|cfg|rs|ts|js|py)$/i.test(p)) {
            try {
              const note = await api.importFile(p);
              showToast(`已打开 ${note.name}`);
              window.dispatchEvent(new CustomEvent("notes-refresh", { detail: note.id }));
            } catch (e) {
              showToast(`打开失败: ${e}`);
            }
          }
        }
      }
    });

    // Events from backend
    await listen<string>("view-change", (e) => { view = e.payload as any; });
    await listen("locked", () => locked.set(true));
    await listen("unlocked", () => locked.set(false));
    await listen<any>("settings-updated", async (e) => {
      settings.set(e.payload);
      document.documentElement.setAttribute("data-theme", e.payload.theme);
      document.documentElement.style.setProperty("--bg-solid", e.payload.bgColor);
    });

    // Modifier + left-drag to move window (non-passthrough only)
    window.addEventListener("mousedown", async (e) => {
      if (e.button !== 0) return;
      if ($settings.passthrough === "full") return;
      const mod = $settings.dragModifier;
      const pressed =
        (mod === "Alt" && e.altKey) ||
        (mod === "Ctrl" && e.ctrlKey) ||
        (mod === "Shift" && e.shiftKey) ||
        (mod === "Meta" && e.metaKey);
      if (!pressed) return;
      e.preventDefault();
      e.stopPropagation();
      try { await getCurrentWindow().startDragging(); } catch {}
    }, true);

    // Local keyboard shortcuts
    window.addEventListener("keydown", (e) => {
      if (!e.ctrlKey || e.altKey || e.metaKey) return;
      const k = e.key.toLowerCase();
      if (k === "b" && !e.shiftKey) {
        e.preventDefault();
        settings.update((s) => ({ ...s, showSidebar: !s.showSidebar }));
        scheduleSave();
      } else if (k === "1" && !e.shiftKey) {
        e.preventDefault(); view = "tasks";
      } else if (k === "2" && !e.shiftKey) {
        e.preventDefault(); view = "main";
      } else if (k === "," && !e.shiftKey) {
        e.preventDefault(); view = "settings";
      } else if (k === "n" && !e.shiftKey) {
        const tag = (e.target as HTMLElement)?.tagName;
        if (tag === "INPUT" || tag === "TEXTAREA") return;
        e.preventDefault();
        if (view === "tasks") {
          setTimeout(() => document.getElementById("new-task-input")?.focus(), 30);
        }
      }
    });

    await listen("focus-new-task", () => {
      view = "tasks";
      setTimeout(() => document.getElementById("new-task-input")?.focus(), 50);
    });

    // Hide context menu in production
    document.addEventListener("contextmenu", (e) => {
      const t = e.target as HTMLElement;
      if (t.tagName !== "TEXTAREA" && t.tagName !== "INPUT") e.preventDefault();
    });
  });

  $effect(() => {
    document.documentElement.setAttribute("data-theme", $settings.theme);
    document.documentElement.style.setProperty("--bg-solid", $settings.bgColor);
  });
</script>

{#if $locked}
  <Lock />
{/if}
<div class="app-root"
  class:transparent-bg={$settings.transparentBg}
  style="--content-opacity: {$settings.transparentBg ? $settings.opacity : 1}; opacity: {$settings.transparentBg ? 1 : $settings.opacity}">
  {#if $settings.showTitleBar}
    <TitleBar />
  {/if}
  <div class="body">
    {#if view === "main"}
      <Main />
    {:else if view === "tasks"}
      <Tasks />
    {:else}
      <Settings onBack={() => (view = "main")} />
    {/if}
  </div>
  {#if $settings.showBottomBar}
    <BottomBar bind:view />
  {/if}
</div>
{#if $toast}
  <div class="toast">{$toast}</div>
{/if}
