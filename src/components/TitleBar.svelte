<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  async function minimize() {
    await getCurrentWindow().minimize();
  }
  async function hide() {
    await getCurrentWindow().hide();
  }
</script>

<div class="titlebar">
  <span class="title">Sneaker</span>
  <button onclick={minimize} title="最小化">—</button>
  <button onclick={hide} title="隐藏">×</button>
</div>
