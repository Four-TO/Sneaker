<script lang="ts">
  import { settings, scheduleSave, showToast } from "../lib/store";
  import { api } from "../lib/api";

  let { view = $bindable() }: { view: "main" | "tasks" | "settings" } = $props();

  async function toggleTop() {
    const next = !$settings.alwaysOnTop;
    settings.update((s) => ({ ...s, alwaysOnTop: next }));
    await api.setAlwaysOnTop(next);
    scheduleSave();
    showToast(next ? "已置顶" : "取消置顶");
  }

  async function togglePassthrough() {
    const order: Array<"off" | "semi" | "full"> = $settings.showTitleBar ? ["off", "semi", "full"] : ["off", "full"];
    const i = order.indexOf($settings.passthrough);
    const next = order[(i + 1) % order.length];
    settings.update((s) => ({ ...s, passthrough: next }));
    await api.setPassthrough(next);
    scheduleSave();
    showToast(`穿透: ${next === "off" ? "关" : next === "semi" ? "半" : "全"}`);
  }

  function toggleSidebar() {
    settings.update((s) => ({ ...s, showSidebar: !s.showSidebar }));
    scheduleSave();
  }

  function toggleTransparentBg() {
    const next = !$settings.transparentBg;
    settings.update((s) => ({ ...s, transparentBg: next }));
    scheduleSave();
    showToast(next ? "背景全透明开" : "背景全透明关");
  }

  async function lockNow() {
    if ($settings.hasMasterPassword) await api.lockNow();
    else showToast("未设置主密码");
  }
</script>

<div class="bottombar">
  <button class:active={view === "main"} onclick={() => (view = "main")} title="笔记 (Ctrl+1)">📝</button>
  <button class:active={view === "tasks"} onclick={() => (view = "tasks")} title="任务 (Ctrl+2)">📋</button>
  {#if view === "main"}
    <button onclick={toggleSidebar} title={($settings.showSidebar ? "隐藏" : "显示") + "侧栏 (Ctrl+B)"}>
      {$settings.showSidebar ? "⮜" : "☰"}
    </button>
  {/if}
  <button class:active={$settings.transparentBg} onclick={toggleTransparentBg} title="背景全透明">
    {$settings.transparentBg ? "▣" : "▢"}
  </button>
  <button onclick={togglePassthrough} title="穿透模式">
    {$settings.passthrough === "off" ? "●" : $settings.passthrough === "semi" ? "◐" : "○"}
  </button>
  <button onclick={toggleTop} title="置顶">{$settings.alwaysOnTop ? "📌" : "📍"}</button>
  {#if $settings.hasMasterPassword}
    <button onclick={lockNow} title="立即锁定">🔒</button>
  {/if}
  <span class="spacer"></span>
  <span class="status">{Math.round($settings.opacity * 100)}%</span>
  <button class:active={view === "settings"} onclick={() => (view = view === "settings" ? "main" : "settings")} title="设置 (Ctrl+,)">
    ⚙
  </button>
</div>
