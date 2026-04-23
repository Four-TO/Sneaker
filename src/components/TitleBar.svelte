<script lang="ts">
  import { settings, scheduleSave, showToast } from "../lib/store";
  import { api } from "../lib/api";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let { view = $bindable() }: { view: "main" | "settings" } = $props();

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

  async function minimize() {
    await getCurrentWindow().minimize();
  }
  async function hide() {
    await getCurrentWindow().hide();
  }
</script>

<div class="titlebar">
  <span class="title">Sneaker</span>
  <button onclick={() => (view = view === "main" ? "settings" : "main")} title="切换">
    {view === "main" ? "⚙" : "◀"}
  </button>
  <button onclick={togglePassthrough} title="穿透模式">
    {$settings.passthrough === "off" ? "●" : $settings.passthrough === "semi" ? "◐" : "○"}
  </button>
  <button onclick={toggleTop} title="置顶">{$settings.alwaysOnTop ? "📌" : "📍"}</button>
  <button onclick={minimize} title="最小化">—</button>
  <button onclick={hide} title="隐藏">×</button>
</div>
