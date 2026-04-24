# Sneaker

**语言：** **简体中文** · [English](README.en.md)

一款基于 **Tauri 2 + Svelte 5 + Rust** 的 Windows 桌面悬浮笔记 & 任务工具。

默认专注模式：无边框、半透明、随时可见，但从不挡路。

---

## 功能

### 窗口与外观
- 无边框、透明、可一键切换置顶
- `Ctrl + 滚轮` 实时调整透明度；设置页提供滑块
- "背景全透明"模式（透明度仅作用于文字与控件）
- 深色 / 浅色主题，切换时背景色自动匹配
- Windows 原生 Acrylic / Mica 毛玻璃效果（`window-vibrancy`）
- 顶部标题栏与底部工具栏可独立显隐
- 非穿透状态下，按住修饰键（Alt / Ctrl / Shift / Win）+ 左键可在窗口任意处拖动

### 交互
- 鼠标穿透**三态**（关 / 半穿透 / 全穿透）；标题栏隐藏时半穿透自动禁用
- 托盘图标 + 完整菜单（显示 / 隐藏 / 置顶 / 穿透 / 设置 / 锁定 / 退出）
  - 左键**双击** → 恢复并聚焦主窗
  - 右键 → 弹出菜单
  - 任务栏图标可在设置中隐藏（`WS_EX_TOOLWINDOW`）
- 全局快捷键（均可重绑，按字段检测冲突，失败自动回滚）
  - 显/隐主窗
  - 切换置顶
  - 循环切换穿透模式
  - 老板键（瞬隐 + 上锁）
  - 快速捕获（唤出窗口 + 直达新建任务输入框）
- 本地快捷键：`Ctrl+1` 笔记 · `Ctrl+2` 任务 · `Ctrl+,` 设置 · `Ctrl+N` 聚焦输入 · `Ctrl+B` 侧栏显隐

### 任务
- 分组：🔥 紧急置顶 · ▶ Working · 📋 Todo · ✓ 今日完成 · ⌄ 更早完成（自动折叠）
- 快速输入前缀：`!xxx` → 紧急置顶；`>xxx` → 直接进 Working
- 双击标题重命名；悬停出现 ▶/⏸、📌 置顶、✕ 删除
- 存储为单一 `tasks.json`，易同步、易 diff、可手工编辑

### 笔记
- 拖拽 `.txt` / `.md` / `.log` / `.json` / ... 进窗口，纯文本打开（不渲染）
- 侧栏列表支持搜索、重命名、删除、自动保存
- 文件以纯 `.md` 存放于 `%APPDATA%\Sneaker\notes\`

### 安全
- 可选主密码（Argon2 KDF），Telegram 风格两级锁
- 老板键同时触发"瞬隐 + 上锁"
- 空闲超时自动锁定（分钟可配）
- 单条笔记加密（ChaCha20-Poly1305 后端已就绪，UI 放在 M3）

### 持久化
- 窗口位置/尺寸自动记忆与恢复（`tauri-plugin-window-state`）
- 设置保存于 `%APPDATA%\Sneaker\settings.json`
- 开机自启可选（`tauri-plugin-autostart`）

---

## 构建

### 依赖
- Node ≥ 20，pnpm ≥ 10
- Rust ≥ 1.80，`rustup target add x86_64-pc-windows-msvc`
- Visual Studio 2022 Build Tools（C++ 工作负载）
- WebView2 运行时（Win11 预装）

### 命令
```bash
pnpm install           # 安装前端依赖
pnpm tauri dev         # 热重载开发模式
pnpm tauri build       # 发布版 exe + NSIS 安装包
```

产物：
- `src-tauri/target/release/sneaker.exe` — 绿色版
- `src-tauri/target/release/bundle/nsis/Sneaker_*-setup.exe` — 安装包

---

## 数据位置

| 类型 | 路径 |
|---|---|
| 设置 | `%APPDATA%\Sneaker\settings.json` |
| 笔记 | `%APPDATA%\Sneaker\notes\*.md` |
| 任务 | `%APPDATA%\Sneaker\tasks.json` |
| 窗口状态 | `%APPDATA%\Sneaker\.window-state.json` |

全部纯文本 / JSON，可用 Git、OneDrive、坚果云等直接同步。

---

## 架构

```
┌─── 前端 (Svelte 5 + Vite + TS) ────────────────────┐
│  App.svelte           状态与视图路由               │
│  views/Main.svelte    笔记                         │
│  views/Tasks.svelte   任务（多分组）               │
│  views/Settings.svelte                             │
│  views/Lock.svelte    主密码锁屏                   │
│  components/          TitleBar, BottomBar          │
│  lib/api.ts           tauri invoke 封装            │
│  lib/store.ts         Svelte stores                │
└─────────────────────────────────────────────────────┘
                           ▲
                   invoke  │  events
                           ▼
┌─── 后端 (Rust + Tauri 2) ──────────────────────────┐
│  lib.rs         装配与启动                          │
│  commands.rs    #[tauri::command] 处理器            │
│  settings.rs    类型化配置                          │
│  storage.rs     笔记、Argon2、ChaCha20-Poly1305     │
│  tasks.rs       任务 JSON 存储                      │
│  tray.rs        托盘图标与菜单                      │
│  hotkeys.rs     全局快捷键注册                      │
│  win_util.rs    WS_EX_TOOLWINDOW、WS_EX_TRANSPARENT │
└─────────────────────────────────────────────────────┘
```

---

## 快捷键速查

### 全局（可在设置中重绑）
| 动作 | 默认 |
|---|---|
| 显/隐窗口 | `Ctrl+Alt+N` |
| 切置顶 | `Ctrl+Alt+T` |
| 切穿透模式 | `Ctrl+Alt+P` |
| 老板键（隐藏+锁定） | `Ctrl+Alt+H` |
| 快速捕获到任务 | `Ctrl+Alt+Q` |

### 本地
| 动作 | 按键 |
|---|---|
| 切到笔记视图 | `Ctrl+1` |
| 切到任务视图 | `Ctrl+2` |
| 切到设置视图 | `Ctrl+,` |
| 聚焦新建输入 | `Ctrl+N`（在任务视图） |
| 侧栏显隐 | `Ctrl+B` |
| 调整透明度 | `Ctrl + 滚轮` |
| 任意位置拖动窗口 | `Alt + 左键拖动`（修饰键可改） |

---

## Roadmap

- [ ] 单条笔记加密 UI（后端已就绪）
- [ ] 标签 / 双向链接 `[[笔记]]`
- [ ] 多便签独立窗口
- [ ] 同步 Provider（WebDAV / Git）—— trait 已预留
- [ ] Markdown 编辑器模式
- [ ] 任务拖拽排序

---

## 许可

MIT
