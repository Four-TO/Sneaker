<script lang="ts">
  import { api } from "../lib/api";
  import { locked } from "../lib/store";

  let pw = $state("");
  let err = $state("");

  async function submit() {
    try {
      const ok = await api.unlock(pw);
      if (ok) { locked.set(false); pw = ""; err = ""; }
      else err = "密码错误";
    } catch (e) {
      err = String(e);
    }
  }
</script>

<div class="lock-overlay">
  <div class="title-big">🔒 Sneaker</div>
  <input type="password" bind:value={pw} onkeydown={(e) => e.key === 'Enter' && submit()} autofocus placeholder="主密码" />
  <button onclick={submit}>解锁</button>
  <div class="err">{err}</div>
</div>
