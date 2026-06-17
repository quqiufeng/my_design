# my_design 设计方案

> 架构设计文档 — 记录技术选型、分层设计、数据流和关键决策。

---

## 1. 整体架构

```
┌──────────────────────────────────────────────────────────────────┐
│                        my_design 二进制                          │
│  ┌──────────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│  │   C 入口层        │  │  LuaJIT 逻辑  │  │  Rust GPU 渲染     │   │
│  │  (main.c)         │  │  (main.lua)   │  │  (gui_gpui/)      │   │
│  │                   │  │              │  │                   │   │
│  │  • dlopen .so     │─▶│  • Skill 管理 │─▶│  • DesignView     │   │
│  │  • lua_engine     │  │  • LLM 调用  │  │  • 三栏布局       │   │
│  │  • run_gui()      │  │  • 协程调度  │  │  • 键盘/鼠标事件  │   │
│  └──────────────────┘  └──────┬───────┘  └────────┬──────────┘   │
│                               │                    │              │
│                        ┌──────┴───────┐           │              │
│                        │  C ABI / FFI  │◄──────────┘              │
│                        │  lua_engine   │                          │
│                        │  gui_tick     │                          │
│                        └──────────────┘                          │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2. 技术选型

### 2.1 GUI 框架：gpui（Zed）

选择 gpui 而非其他方案的原因：

| 方案 | 优点 | 缺点 | 结论 |
|------|------|------|------|
| **Electron** | 生态丰富 | 150MB+ 体积，内存高 | ❌ |
| **Tauri** | 体积小 | WebView 依赖，Rust 仅做后端 | ❌ |
| **egui** | 纯 Rust | 组件少，文本渲染弱 | ❌ |
| **gpui** | GPU 加速，原生体验 | 仅 Linux/macOS | ✅ |

gpui 是 Zed 编辑器使用的 GPU 渲染框架，提供：
- GPU 加速 2D 渲染（Vulkan/Metal/DirectX）
- 响应式 UI 模型
- 跨平台窗口和输入事件系统

### 2.2 脚本引擎：LuaJIT

选择 LuaJIT 而非其他脚本：

| 方案 | 体积 | 性能 | FFI | 热更新 |
|------|------|------|-----|--------|
| **LuaJIT** | ~300KB | 极快 | 原生 C FFI | ✅ |
| **Python** | ~50MB | 慢 | 需要 PyO3 | ❌ |
| **WASM** | ~1MB | 快 | 复杂 | ❌ |

LuaJIT 的 FFI 可以直接调用 Rust `.so` 导出的 C 函数，不需要额外绑定层。

### 2.3 组件策略：自封装

不依赖 `gpui-component` 等外部 UI 库，全部组件自己实现：

```
widgets.rs (布局原语)
  ├── h_flex()    — 水平 flex 容器
  └── v_flex()    — 垂直 flex 容器

lib.rs (业务组件)
  ├── token_row()     — Design Token 行
  ├── export_btn()    — 导出按钮
  ├── history_item()  — 历史记录项
  ├── chip()          — 分类标签
  ├── section_title() — 分区标题
  └── divider()       — 分割线
```

优势：完全控制渲染行为、无外部依赖、自由定制样式。

---

## 3. 数据流

### 3.1 用户输入 → AI 响应

```
用户点击 "Generate"
  │
  ▼
Rust gui: on_mouse_down 回调
  │
  ▼
Lua handler (协程):
  ├─ ① build_prompt(skill_id, user_input)   → 生成 AI Prompt
  ├─ ② opencode.http_post(url, body, key)    → 调用 LLM API
  ├─ ③ parse_openai_response(response)       → 解析 AI 回复
  ├─ ④ gui.stream_delta(app, sid, chunk)     → 流式显示
  └─ ⑤ gui.append_message(app, sid, ...)     → 追加完整消息
  │
  ▼
Rust gui: cx.notify() → 重渲染
```

### 3.2 定时器驱动

```
Rust 60fps 定时器 (16ms)
  │
  ├─ my_design_gui_tick(lua_state)   [C 函数]
  │     └─ Lua gui_tick()
  │           └─ resume pending_coroutines[]
  │
  └─ cx.notify()                    [Rust 重绘]
        └─ DesignView::render()
```

---

## 4. Lua ↔ Rust 接口契约

### 4.1 Rust 导出给 Lua 的 C 函数

```rust
void*   gui_app_create(const char* config_json);
void    gui_app_free(void* app);
void    gui_on_user_message(void* app, callback_fn, void* userdata);
int     gui_run(void* app, void* lua_state);
void    gui_stream_delta(void* app, const char* session_id, const char* delta);
void    gui_append_message(void* app, const char* session_id, const char* role, const char* text);
void    gui_append_design(void* app, const char* session_id, const char* html, const char* description);
```

### 4.2 Lua 注册的全局函数（被 Rust 调用）

```lua
function gui_tick()      -- 60fps 定时器，恢复协程
function gui_on_copy()   -- 用户双击复制文本
```

### 4.3 `opencode.*` Lua 全局表

```lua
opencode.get_lua_state()  → lightuserdata (lua_State*)
opencode.set_clipboard()  → 写入系统剪贴板
opencode.http_post()      → curl HTTP POST 请求
opencode.log_info/debug/warn/error()
```

---

## 5. Skill 系统设计

### 5.1 Skill 定义结构

```lua
{
    id = "skill-id",           -- 唯一标识
    name = "显示名称",
    description = "描述",
    category = "mobile|web",   -- 分类

    tokens = {                 -- Design Token
        colors = { ... },      -- 颜色系统
        typography = { ... },  -- 字体系统
        spacing = { ... },     -- 间距系统
        radius = { ... },      -- 圆角系统
        shadows = { ... },     -- 阴影系统（可选）
    },

    components = { ... },      -- 组件清单

    layout_template = { ... }, -- 布局模板

    prompt_template = [[...]], -- AI Prompt 模板
}
```

### 5.2 Skill 加载流程

```
main.lua 启动
  │
  ├─ 读取 skills/ 目录下的所有 .lua 文件
  ├─ 加载到 skills[] 表
  │
  └─ run_gui()
       └─ 用户选择 Skill → build_prompt(skill_id, input)
            └─ 填充 prompt_template 中的 Token 占位符
                 └─ 发送给 LLM API
```

---

## 6. 测试策略

### 6.1 Rust 测试

- `cargo check` — 编译检查（0 warning 标准）
- `cargo build --release` — release 构建

### 6.2 Lua GUI 自动化测试

通过环境变量触发：

```bash
# 测试脚本模式：启动后执行 Lua 测试脚本
MY_DESIGN_GUI_TEST_SCRIPT=./test_gui.lua ./my_design

# 测试消息模式：启动 2 秒后自动发送消息
MY_DESIGN_GUI_TEST_MSG="设计一个登录页" ./my_design
```

### 6.3 测试脚本示例

```lua
-- test_gui.lua
print("=== my_design GUI Test ===")
gui.append_message(app, session_id, "user", "设计一个登录页")
-- 等待 3 秒后检查结果
coroutine.yield()
for _ = 1, 180 do coroutine.yield() end  -- ~3s at 60fps
local msgs = gui.get_messages(app)
print("Messages received: " .. #msgs)
print("=== Test Complete ===")
```

---

## 7. 关键设计决策

### 7.1 为什么不用 Electron/Tauri

- Electron 体积大（150MB+），内存占用高
- Tauri 依赖 WebView，且 Rust 仅做后端
- 我们的 GUI 需要 GPU 加速渲染设计预览，gpui 更合适

### 7.2 为什么用 LuaJIT 而不是纯 Rust

- **热更新**：修改 Lua 不需要重编译 Rust .so
- **快速迭代**：Skill 定义、Prompt 模板、工具逻辑都在 Lua 中
- **FFI 直接**：LuaJIT 可以直接调用 C 接口，无需绑定层

### 7.3 为什么要自封装组件

- gpui-component 库太大，包含 50+ 不需要的组件
- 自封装可以完全控制渲染行为和样式
- 减少编译时间，减小二进制体积

### 7.4 为什么 C 入口而不是直接 Rust 入口

- C 可以 `dlopen` 加载 `.so` 并传递 `lua_state` 指针
- C 可以设置 `RTLD_GLOBAL` 使 `.so` 中的符号可见
- 保持与已有 C 工具链的兼容性

---

## 8. 编译与部署

```bash
# 完整构建
make gui      # Rust .so (~3min)
make          # C 二进制 (~1s)

# 产物
my_design            # 主二进制 (47KB)
libmy_design_gui.so  # Rust 渲染引擎 (35MB)
libmy_design_agent.a # C 静态库 (45KB)
```

---

## 9. 后续规划

- [ ] 设计稿 HTML 预览渲染（右侧画布）
- [ ] Skill 选择器 UI 交互（点击切换风格）
- [ ] 输入框键盘事件捕获（真正的文字输入）
- [ ] 多轮对话修改
- [ ] 代码导出（React / Vue / HTML+CSS）
- [ ] 自定义 Skill 编辑器
- [ ] Design Token 可视化编辑
- [ ] Figma/Sketch 导入导出
- [ ] 暗色/亮色主题切换

---

## 10. 设计稿生成流程（用户视角）

```
用户打开 my_design
        │
        ▼
  ┌─────────────────────────────────────────────┐
  │  三栏界面展示                                │
  │  左侧: Style 列表  中: 空画布  右: Token     │
  │  底部: 输入框 [Describe UI...] [✨ Generate] │
  └─────────────────────────────────────────────┘
        │
        ▼
  用户在输入框中打字（键盘事件捕获）
        │
        ▼
  点击 Generate 按钮
        │
        ▼
  ┌─────────────────────────────────────────────┐
  │  Rust → C 回调 → Lua 协程                   │
  │                                              │
  │  ① 获取当前 Skill ID + 用户输入文本          │
  │  ② build_prompt() → 填充 Token 生成 Prompt  │
  │  ③ opencode.http_post() → 调用 LLM API      │
  │  ④ parse_openai_response() → 提取设计内容   │
  │  ⑤ gui.stream_delta() → 流式输出到画布      │
  │  ⑥ gui.append_message("design", ...)        │
  └─────────────────────────────────────────────┘
        │
        ▼
  ┌─────────────────────────────────────────────┐
  │  画布区域显示：                               │
  │                                              │
  │  [You] 设计一个电商App商品详情页              │
  │  ┌─────────────────────────────────┐         │
  │  │  [AI] 设计描述 +                 │         │
  │  │  🖼 设计预览占位                  │         │
  │  │  - 组件树                        │         │
  │  │  - Layout 说明                  │         │
  │  │  - CSS Variables                │         │
  │  └─────────────────────────────────┘         │
  │                                              │
  └─────────────────────────────────────────────┘
        │
        ▼
  用户继续输入新 prompt 迭代修改
        │
        ▼
  ...（循环）
```

### 流式输出机制

```
LLM 返回内容 → 按 20 字节切块
  │
  ├─ chunk → gui.stream_delta() → Rust 追加到 messages
  ├─ coroutine.yield() → 让出给 60fps 定时器
  └─ 下一帧 → cx.notify() → 渲染最新内容

效果：AI 生成的内容逐字出现在画布上
```

### 设计稿的 Special 处理

当 AI 返回的内容被标记为 `role = "design"` 时：

```lua
gui.append_message(app, sid, "design", "---DESIGN_HTML---\n" .. html_content)
```

Rust 侧收到 `role == "design"` 的消息时：
- 正常显示文字描述
- 额外渲染一个 **设计预览卡片**（彩色边框、预览图标、HTML 标记）
- 后续可以嵌入 WebView 渲染真实 HTML

### 多轮迭代流程

```
用户: "设计一个登录页"
  → AI: 返回设计稿
用户: "改成圆角按钮"
  → AI: 调整 Design Token radius → 返回更新设计稿
用户: "换成深色主题"
  → AI: 调整颜色 Token → 返回更新设计稿
```

每次修改实际上是：
1. 用户新输入 → 追加 context
2. AI 基于已有设计稿 + 新需求 → 生成增量修改
3. 画布更新为新版本

---

## 11. 设计稿渲染方案（规划）

### Phase 1 — 文字描述（当前）
AI 返回设计描述 + 组件树 + CSS Variables，以文本形式展示

### Phase 2 — 结构化预览
将 AI 返回的 HTML 片段渲染为缩略图，显示设计预览卡片

### Phase 3 — 可交互预览
嵌入 WebView，实时渲染设计稿 HTML/CSS，支持点击交互

```
Phase 1                    Phase 2                  Phase 3
┌──────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ 文字描述      │    │  HTML 缩略图 +    │    │  可交互 WebView  │
│              │    │  文字描述          │    │                  │
│ - 设计说明   │    │                   │    │  点击 / 滚动    │
│ - 组件树     │    │  ┌──────────┐     │    │  实时修改反馈   │
│ - Token 清单 │    │  │ 🖼 预览   │     │    │                  │
│              │    │  └──────────┘     │    │                  │
└──────────────┘    └──────────────────┘    └──────────────────┘
      现在                 下一步               未来
```

---

## 12. 页面拆分工作流（v2 核心流程）

### 12.1 完整流程

```
用户: "设计一个生鲜电商App"
       │
       ▼
  ┌──────────────────────────────────────┐
  │  需求录入阶段                         │
  │                                      │
  │  输入框: [设计一个生鲜电商App________] │
  │                         [🔍 拆分需求] │
  └──────────────────────────────────────┘
       │
       ▼
  Lua 调用 LLM，分析需求 → 返回页面列表
       │
       ▼
  ┌──────────────────────────────────────┐
  │  页面确认阶段                         │
  │                                      │
  │  📋 生鲜电商App — 建议页面:          │
  │                                      │
  │  ☑ 🛒 商品列表页    可编辑名称      │
  │  ☑ 📄 商品详情页                    │
  │  ☑ 🛵 购物车页                      │
  │  ☑ 💳 结算页                        │
  │  ☑ 👤 个人中心                      │
  │  ☑ 📦 订单列表                      │
  │                                      │
  │  [+ 添加页面]                        │
  │                                      │
  │  全局 Skill: [🌐 Web Landing  ▼]     │
  │                                      │
  │         [✨ 生成全部] [按页生成]      │
  └──────────────────────────────────────┘
       │  用户确认
       ▼
  ┌──────────────────────────────────────┐
  │  生成阶段                             │
  │                                      │
  │  ┌─ 商品列表页 ──── [■■■■■■■■□□] 80%│
  │  ├─ 商品详情页 ──── [■■■■□□□□□□] 40%│
  │  ├─ 购物车页 ────── [□□□□□□□□□□]  0%│
  │  ├─ 结算页 ──────── 待生成           │
  │  └─ ...                              │
  │                                      │
  │  所有页面共用同一 Skill              │
  └──────────────────────────────────────┘
```

### 12.2 三个核心概念

```
项目 (Project)
  │
  ├─ 用户一句话需求（如"设计一个生鲜电商App"）
  ├─ AI 拆分为多个页面
  └─ 所有页面共享一个 Skill
       │
       ├── 页面1: 商品列表页
       │     ├─ 页面专属 prompt（AI 根据需求自动生成）
       │     └─ + Skill Token → 最终设计稿
       │
       ├── 页面2: 商品详情页
       │     ├─ 页面专属 prompt
       │     └─ + Skill Token → 最终设计稿
       │
       └── 页面3: 购物车页
             ├─ 页面专属 prompt
             └─ + Skill Token → 最终设计稿
```

### 12.3 状态机

```
[Input] ──用户输入需求──▶ [Splitting] ──AI返回页面列表──▶ [Confirm]
  ▲                                                       │
  │                                             用户确认 / 调整
  │                                                       ▼
  │                                                  [Generating]
  │                                                       │
  │                                                 全部生成完成
  │                                                       │
  └────────────────────── 回到 [Input] ◀───────────────────┘
```

### 12.4 Skill 的作用范围

```
Skill 是全局公用的，不绑定到某个页面：

  Skill(iOS HIG)
    ├── + Page(商品列表页) → iOS 风格的商品列表
    ├── + Page(商品详情页) → iOS 风格的商品详情
    └── + Page(购物车页)   → iOS 风格的购物车

  Skill(Material 3)
    ├── + Page(商品列表页) → Material 风格的商品列表
    └── ...
```

一个项目只选一个 Skill，保证**所有页面视觉风格统一**。

### 12.5 与旧流程的对比

| 旧流程（当前） | 新流程（规划） |
|---------------|---------------|
| 用户自己写提示词 | AI 分析需求，自动拆分页面 |
| 单次生成一个设计 | 批量生成整个项目的所有页面 |
| 没有项目概念 | 页面同属一个项目，风格统一 |
| Skill 在左侧列表 | Skill 在确认页面全局选择 |

### 12.6 用户价值

1. **从"写提示词"变成"确认 AI 的理解"** — 设计师不需要会写 prompt
2. **一次需求，整套页面** — 不用一个一个想
3. **风格自动统一** — 所有页面同一个 Skill，不会出现 iOS 风格混搭 Android
4. **可调整** — AI 拆得不准确，用户可以增删改页面

---

## 13. 页面列表 → 单页编辑 → 返回（v2 精炼流程）

### 13.1 三层视图切换

```
┌─────────────────────────────────────────────────────────────┐
│  视图1: 项目页面列表                                         │
│                                                             │
│  📁 生鲜电商App                                             │
│                                                             │
│  ☑ 🛒 商品列表页      [编辑 →]   状态: 待生成              │
│  ☑ 📄 商品详情页      [编辑 →]   状态: ✅ 已保存            │
│  ☑ 🛵 购物车页        [编辑 →]   状态: 待生成              │
│  ☑ 💳 结算页          [编辑 →]   状态: 待生成              │
│  ☑ 👤 个人中心        [编辑 →]   状态: 待生成              │
│                                                             │
│  Skill: [Shadcn/ui ▼]                    [✨ 全部生成]      │
│                                                             │
│         ┌── 点击 [编辑 →] 或双击某页 ──┐                   │
│         ▼                               │                   │
│  ┌────────────────────────────────┐     │                   │
│  │  视图2: 单页编辑                │     │                   │
│  │                                │     │                   │
│  │  ← 返回项目列表                │     │                   │
│  │                                │     │                   │
│  │  📄 商品详情页                 │     │                   │
│  │                                │     │                   │
│  │  补充需求:                     │     │                   │
│  │  [需要展示商品大图、规格选择   │     │                   │
│  │   用户评价、底部购买栏________]│     │                   │
│  │                                │     │                   │
│  │  [+ 补充到 Prompt]            │     │                   │
│  │                                │     │                   │
│  │  ┌─ 设计预览 ────────────┐    │     │                   │
│  │  │  AI 生成的设计稿       │    │     │                   │
│  │  │  (文字描述 + 预览卡片) │    │     │                   │
│  │  └───────────────────────┘    │     │                   │
│  │                                │     │                   │
│  │  [💾 保存并返回列表]          │     │                   │
│  └────────────────────────────────┘     │                   │
│                                         │                   │
└─────────────────────────────────────────┘                   │
                                                              │
  视图3: 批量生成（点击"全部生成"后）                           │
  ┌────────────────────────────────┐                          │
  │  🎨 Generating Designs        │                          │
  │  3/5 pages completed          │                          │
  │                                │                          │
  │  ✅ 🛒 商品列表页    Done     │                          │
  │  ✅ 📄 商品详情页    Done     │                          │
  │  ✅ 🛵 购物车页      Done     │                          │
  │  ⏳ 💳 结算页        Generating...                       │
  │  ⏳ 👤 个人中心      Pending  │                          │
  └────────────────────────────────┘                          │
```

### 13.2 三种视图的状态管理

```
enum ViewState {
    PageList,       // 视图1: 项目页面列表
    SingleEditor,   // 视图2: 单页编辑
    BatchGenerate,  // 视图3: 批量生成中
}
```

| 视图 | 用户操作 | 触发动作 |
|------|---------|---------|
| **PageList** | 点击某页的 [编辑 →] | 切换到 SingleEditor，加载该页数据 |
| **PageList** | 点击 [全部生成] | 切换到 BatchGenerate，逐页调用 LLM |
| **SingleEditor** | 补充需求 + [补充到 Prompt] | 更新该页专属 prompt，可选调用 LLM |
| **SingleEditor** | 点击 [💾 保存并返回列表] | 保存当前页状态，回到 PageList |
| **BatchGenerate** | 全部完成 | 回到 PageList，已生成页标记 ✅ |

### 13.3 单页编辑器的数据模型

```
PageItem {
    id: string,           // 唯一标识
    name: string,         // 页面名称（用户可改）
    selected: bool,       // 是否包含在批量生成中
    done: bool,           // 是否已生成
    prompt: string,       // 页面专属提示词（AI 初始生成 + 用户补充）
    design_result: string,// AI 返回的设计稿内容
}
```

### 13.4 Skill 的作用范围（重申）

```
Skill 是项目级别全局的：

  项目: 生鲜电商App
  Skill: Material 3
    ├── 商品列表页 → Material 3 风格
    ├── 商品详情页 → Material 3 风格
    └── 购物车页   → Material 3 风格
```

用户可以在 PageList 视图切换 Skill，切换后：
- 已生成的页面标记为"风格需更新"
- 重新生成时使用新 Skill

### 13.5 用户操作路径

```
首次使用:
  输入需求 → AI 拆分为页面列表
          → 逐页点击编辑，补充细节
          → 保存返回，切换到下一页
          → 确认所有页无误 → 点击全部生成

迭代修改:
  页面列表 → 点击已生成页 → 编辑补充
          → 重新生成该页 → 保存返回

新项目:
  点击 "New Project" → 输入新需求 → 重复以上流程
```

### 13.6 与 Figma 的类比

| Figma | my_design |
|-------|-----------|
| Pages 面板 | 页面列表视图 |
| 选中 Page | 进入单页编辑 |
| Design Panel | 补充需求输入 + 预览 |
| 返回 Pages | 保存并返回列表 |
| 团队 Library | Skill（全局风格） |

设计师上手成本极低。

---

## 14. 子流程 / 页面层级（v2.1）

### 14.1 为什么需要子流程

用户的一个需求拆出来的页面，本身可能还有子页面：

```
用户: "设计一个生鲜电商App"
       │
       ▼
AI 拆分结果:
  📄 商品详情          ← 普通页面 (depth=0)
    ├── 商品详情       ← 子流程 (depth=1)
    └── 评价列表       ← 子流程 (depth=1)
  🛒 购物车流程        ← 普通页面 (depth=0)
    ├── 购物车列表     ← 子流程 (depth=1)
    ├── 结算页         ← 子流程 (depth=1)
    └── 支付结果       ← 子流程 (depth=1)
  👤 个人中心          ← 普通页面 (depth=0)
    ├── 个人主页       ← 子流程 (depth=1)
    ├── 订单列表       ← 子流程 (depth=1)
    │   ├── 订单详情   ← 子子流程 (depth=2)
    │   └── 物流跟踪   ← 子子流程 (depth=2)
    └── 设置           ← 子流程 (depth=1)
```

### 14.2 数据模型

```rust
struct PageItem {
    name: String,           // 页面名称
    depth: usize,           // 层级深度（0=顶层，1=子流程，2=子子流程）
    selected: bool,         // 用户勾选
    done: bool,             // 是否已生成
    prompt: String,         // 该页专属提示词（用户补充）
    design_preview: String, // AI 返回的设计稿
}
```

### 14.3 UI 展示

```
☑ ─ 🛒 购物车流程          ← depth=0, 正常缩进
☑  └─ 🛒 购物车列表        ← depth=1, 缩进 24px + 分支线
☑  └─ 💳 结算页            ← depth=1
☑  └─ 💳 支付结果          ← depth=1

☑ ─ 📄 商品详情            ← depth=0
☑  └─ 📄 商品详情          ← depth=1
☑  └─ 💬 评价列表          ← depth=1
```

- 每层 depth 增加一级缩进（24px）
- 子流程前显示 `└─` 分支线
- 操作逻辑不变：点击任意行进入该页编辑器

### 14.4 Lua → Rust 数据格式

```json
[
    {"name": "购物车流程", "depth": 0},
    {"name": "购物车列表", "depth": 1},
    {"name": "结算页",     "depth": 1},
    {"name": "商品详情",   "depth": 0},
    {"name": "评价列表",   "depth": 1}
]
```

`gui_set_pages()` 兼容两种格式：
- 旧格式：`["商品列表", "商品详情"]`（所有 depth 默认为 0）
- 新格式：`[{"name":"...", "depth":0}, ...]`

### 14.5 编辑器不变

子流程和普通页面的编辑器完全一样：
- 补充需求 → 生成该页 → 保存返回
- 不区分"这是主流程还是子流程"，对用户来说每一页都是独立的编辑单元
