<script lang="ts">
  import { onMount } from "svelte";
  import { settings, scheduleSave, showToast, defaultHotkeys } from "../lib/store";
  import { api } from "../lib/api";

  let { onBack }: { onBack: () => void } = $props();

  let masterOld = $state("");
  let masterNew = $state("");
  let masterNew2 = $state("");
  let capturing = $state<string | null>(null);
  let hkFailed = $state<Record<string, string>>({});

  function updateFailedFromList(failed: string[]) {
    const m: Record<string, string> = {};
    for (const f of failed) {
      const [field, , reason] = f.split("|");
      if (field) m[field] = reason || "失败";
    }
    hkFailed = m;
  }

  onMount(async () => {
    try { updateFailedFromList(await api.applyHotkeys($settings)); } catch {}
  });

  async function applyOpacity(v: number) {
    settings.update((s) => ({ ...s, opacity: v }));
    await api.setOpacity(v);
    scheduleSave();
  }
  async function applyTheme(v: "dark" | "light") {
    const defaultBg = v === "dark" ? "#14161e" : "#f8f9fc";
    settings.update((s) => {
      const isDefault = s.bgColor === "#14161e" || s.bgColor === "#f8f9fc" || !s.bgColor;
      return { ...s, theme: v, bgColor: isDefault ? defaultBg : s.bgColor };
    });
    scheduleSave();
  }
  async function applyBg(v: string) {
    settings.update((s) => ({ ...s, bgColor: v }));
    scheduleSave();
  }
  async function applyBlur(v: string) {
    settings.update((s) => ({ ...s, blur: v as any }));
    await api.applyTheme($settings);
    scheduleSave();
  }
  async function applyTop(v: boolean) {
    settings.update((s) => ({ ...s, alwaysOnTop: v }));
    await api.setAlwaysOnTop(v);
    scheduleSave();
  }
  async function applyTitleBar(v: boolean) {
    settings.update((s) => ({ ...s, showTitleBar: v, passthrough: !v && s.passthrough === "semi" ? "off" : s.passthrough }));
    await api.setTitleBar(v);
    await api.setPassthrough($settings.passthrough);
    scheduleSave();
  }
  async function applyTray(v: boolean) {
    settings.update((s) => ({ ...s, showTrayIcon: v }));
    await api.setTrayVisible(v);
    scheduleSave();
    if (!v) showToast("托盘已隐藏，注意用热键唤出");
  }
  async function applyTaskbar(v: boolean) {
    settings.update((s) => ({ ...s, skipTaskbar: v }));
    await api.setSkipTaskbar(v);
    scheduleSave();
  }
  async function applyPassthrough(v: string) {
    settings.update((s) => ({ ...s, passthrough: v as any }));
    await api.setPassthrough(v);
    scheduleSave();
  }
  async function applyAutostart(v: boolean) {
    settings.update((s) => ({ ...s, autostart: v }));
    await api.setAutostart(v);
    scheduleSave();
  }
  async function applyAutoLock(v: number) {
    settings.update((s) => ({ ...s, autoLockMinutes: v }));
    scheduleSave();
  }

  async function captureKey(field: keyof typeof $settings.hotkeys) {
    capturing = field;
    const oldVal = $settings.hotkeys[field];
    await api.pauseHotkeys();
    showToast("按下组合键… (Esc 取消)");
    const finish = async (combo: string | null) => {
      window.removeEventListener("keydown", handler, true);
      capturing = null;
      if (combo) {
        settings.update((s) => ({ ...s, hotkeys: { ...s.hotkeys, [field]: combo } }));
      }
      scheduleSave();
      try {
        let failed = await api.applyHotkeys($settings);
        const mine = failed.find(f => f.startsWith(field + "|"));
        if (mine) {
          const reason = mine.split("|")[2] ?? "未知原因";
          settings.update((s) => ({ ...s, hotkeys: { ...s.hotkeys, [field]: oldVal } }));
          scheduleSave();
          failed = await api.applyHotkeys($settings);
          showToast(`${combo} 注册失败：${reason}，已回滚`);
        } else if (combo) {
          showToast(`已设置 ${combo}`);
        }
        updateFailedFromList(failed);
      } catch (e) {
        showToast(`热键注册失败: ${e}`);
      }
    };
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") { finish(null); return; }
      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
      const parts: string[] = [];
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      if (e.metaKey) parts.push("Meta");
      let k = e.key;
      if (k.length === 1) k = k.toUpperCase();
      if (k === " ") k = "Space";
      parts.push(k);
      finish(parts.join("+"));
    };
    window.addEventListener("keydown", handler, true);
  }

  async function restoreHotkeys() {
    settings.update((s) => ({ ...s, hotkeys: { ...defaultHotkeys } }));
    scheduleSave();
    const failed = await api.applyHotkeys($settings);
    updateFailedFromList(failed);
    if (failed.length === 0) showToast("已恢复默认快捷键");
    else showToast(`默认已应用，${failed.length} 项被占用：${failed.map(f => f.split("|")[1]).join(", ")}`);
  }

  async function toggleTransparentBg(v: boolean) {
    settings.update((s) => ({ ...s, transparentBg: v }));
    scheduleSave();
  }

  async function saveMasterPassword() {
    if (masterNew !== masterNew2) { showToast("两次密码不一致"); return; }
    if (!masterNew) { showToast("密码为空"); return; }
    try {
      await api.setMasterPassword(masterOld, masterNew);
      settings.update((s) => ({ ...s, hasMasterPassword: true }));
      masterOld = masterNew = masterNew2 = "";
      showToast("主密码已设置");
    } catch (e) {
      showToast(`失败: ${e}`);
    }
  }

  async function clearMasterPassword() {
    if (!masterOld) { showToast("请输入当前密码"); return; }
    try {
      await api.setMasterPassword(masterOld, "");
      settings.update((s) => ({ ...s, hasMasterPassword: false }));
      masterOld = "";
      showToast("已清除主密码");
    } catch (e) {
      showToast(`失败: ${e}`);
    }
  }

  async function lockNow() {
    await api.lockNow();
  }
</script>

<div class="settings">
  <div style="display:flex;align-items:center;gap:8px;margin-bottom:8px;">
    <button class="ghost" onclick={onBack} style="background:transparent;border:1px solid var(--border);color:var(--fg);padding:4px 10px;border-radius:4px;cursor:pointer;">◀ 返回</button>
    <div style="flex:1;"></div>
  </div>

  <h2>外观</h2>
  <div class="row"><label>主题</label>
    <select value={$settings.theme} onchange={(e) => applyTheme((e.currentTarget as HTMLSelectElement).value as any)}>
      <option value="dark">深色</option><option value="light">浅色</option>
    </select>
  </div>
  <div class="row"><label>透明度 {Math.round($settings.opacity * 100)}%</label>
    <input type="range" min="0.2" max="1" step="0.01" value={$settings.opacity}
      oninput={(e) => applyOpacity(+(e.currentTarget as HTMLInputElement).value)} />
  </div>
  <div class="row"><label>背景色</label>
    <input type="text" value={$settings.bgColor} oninput={(e) => applyBg((e.currentTarget as HTMLInputElement).value)} placeholder="#14161e" />
  </div>
  <div class="row"><label>背景全透明 <span class="hint">(仅文字应用透明度)</span></label>
    <input type="checkbox" checked={$settings.transparentBg} onchange={(e) => toggleTransparentBg((e.currentTarget as HTMLInputElement).checked)} />
  </div>
  <div class="row"><label>模糊效果</label>
    <select value={$settings.blur} onchange={(e) => applyBlur((e.currentTarget as HTMLSelectElement).value)}>
      <option value="none">关闭</option><option value="acrylic">Acrylic</option><option value="mica">Mica</option>
    </select>
  </div>

  <h2>窗口行为</h2>
  <div class="row"><label>显示标题栏</label>
    <input type="checkbox" checked={$settings.showTitleBar} onchange={(e) => applyTitleBar((e.currentTarget as HTMLInputElement).checked)} />
  </div>
  <div class="row"><label>始终置顶</label>
    <input type="checkbox" checked={$settings.alwaysOnTop} onchange={(e) => applyTop((e.currentTarget as HTMLInputElement).checked)} />
  </div>
  <div class="row"><label>穿透模式</label>
    <select value={$settings.passthrough} onchange={(e) => applyPassthrough((e.currentTarget as HTMLSelectElement).value)}>
      <option value="off">关闭</option>
      <option value="semi" disabled={!$settings.showTitleBar}>半穿透 {$settings.showTitleBar ? "" : "(需标题栏)"}</option>
      <option value="full">全穿透</option>
    </select>
  </div>
  <div class="row"><label>隐藏任务栏图标</label>
    <input type="checkbox" checked={$settings.skipTaskbar} onchange={(e) => applyTaskbar((e.currentTarget as HTMLInputElement).checked)} />
  </div>
  <div class="row"><label>显示托盘图标</label>
    <input type="checkbox" checked={$settings.showTrayIcon} onchange={(e) => applyTray((e.currentTarget as HTMLInputElement).checked)} />
  </div>
  <div class="row"><label>拖动修饰键 <span class="hint">(按住此键+左键可在任意处拖动窗口)</span></label>
    <select value={$settings.dragModifier} onchange={(e) => { const v = (e.currentTarget as HTMLSelectElement).value as any; settings.update(s => ({ ...s, dragModifier: v })); scheduleSave(); }}>
      <option value="Alt">Alt</option>
      <option value="Ctrl">Ctrl</option>
      <option value="Shift">Shift</option>
      <option value="Meta">Win</option>
    </select>
  </div>

  <h2>全局快捷键 {capturing ? `(按下组合键为 ${capturing} 赋值，Esc 取消)` : ""}</h2>
  <div class="row"><label>恢复默认</label>
    <button class="ghost" onclick={restoreHotkeys}>重置全部</button>
  </div>
  <div class="row"><label>显隐主窗</label>
    <button class="ghost" class:hk-fail={hkFailed.toggleShow} onclick={() => captureKey("toggleShow")}>{$settings.hotkeys.toggleShow}</button>
    {#if hkFailed.toggleShow}<span class="hk-err">❌ {hkFailed.toggleShow}</span>{/if}
  </div>
  <div class="row"><label>切置顶</label>
    <button class="ghost" class:hk-fail={hkFailed.toggleTop} onclick={() => captureKey("toggleTop")}>{$settings.hotkeys.toggleTop}</button>
    {#if hkFailed.toggleTop}<span class="hk-err">❌ {hkFailed.toggleTop}</span>{/if}
  </div>
  <div class="row"><label>切穿透</label>
    <button class="ghost" class:hk-fail={hkFailed.togglePassthrough} onclick={() => captureKey("togglePassthrough")}>{$settings.hotkeys.togglePassthrough}</button>
    {#if hkFailed.togglePassthrough}<span class="hk-err">❌ {hkFailed.togglePassthrough}</span>{/if}
  </div>
  <div class="row"><label>老板键（隐藏并锁定）</label>
    <button class="ghost" class:hk-fail={hkFailed.bossKey} onclick={() => captureKey("bossKey")}>{$settings.hotkeys.bossKey}</button>
    {#if hkFailed.bossKey}<span class="hk-err">❌ {hkFailed.bossKey}</span>{/if}
  </div>
  <div class="row"><label>快速捕获</label>
    <button class="ghost" class:hk-fail={hkFailed.quickCapture} onclick={() => captureKey("quickCapture")}>{$settings.hotkeys.quickCapture}</button>
    {#if hkFailed.quickCapture}<span class="hk-err">❌ {hkFailed.quickCapture}</span>{/if}
  </div>

  <h2>安全</h2>
  <div class="row"><label>当前状态</label>
    <span class="hint">{$settings.hasMasterPassword ? "已启用主密码" : "未设置主密码"}</span>
  </div>
  {#if $settings.hasMasterPassword}
    <div class="row"><label>立即锁定</label><button onclick={lockNow}>锁定</button></div>
  {/if}
  <div class="row"><label>自动锁定 (分钟, 0=关)</label>
    <input type="number" min="0" max="999" value={$settings.autoLockMinutes}
      oninput={(e) => applyAutoLock(+(e.currentTarget as HTMLInputElement).value)} />
  </div>
  <div class="row"><label>当前密码</label>
    <input type="password" bind:value={masterOld} placeholder={$settings.hasMasterPassword ? "必填" : "(首次设置留空)"} />
  </div>
  <div class="row"><label>新密码</label>
    <input type="password" bind:value={masterNew} />
  </div>
  <div class="row"><label>确认新密码</label>
    <input type="password" bind:value={masterNew2} />
  </div>
  <div class="row">
    <label></label>
    <button onclick={saveMasterPassword}>设置主密码</button>
    {#if $settings.hasMasterPassword}
      <button class="ghost" onclick={clearMasterPassword}>清除主密码</button>
    {/if}
  </div>
  <div class="row"><span class="hint">私密笔记：在主界面选中笔记后，在笔记列表加密（通过下一版菜单）。当前版本可在笔记上通过右键暂未实现，将在 M3 完善。</span></div>

  <h2>启动</h2>
  <div class="row"><label>开机自启</label>
    <input type="checkbox" checked={$settings.autostart} onchange={(e) => applyAutostart((e.currentTarget as HTMLInputElement).checked)} />
  </div>

  <h2>数据</h2>
  <div class="row"><label>笔记目录</label>
    <span class="hint">{$settings.notesDir || "(默认: %APPDATA%/Sneaker/notes)"}</span>
  </div>
  <div class="row"><label>同步</label><span class="hint">即将推出</span></div>
</div>
