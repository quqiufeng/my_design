# my_design — AI 驱动的 UI 设计系统

> 基于 AI 的 UI 设计系统，集成主流 App 与网站 UI 设计方案，助力快速生成高质量界面。

---

## 项目概述

`my_design` 是一个 **AI 驱动的 UI 设计系统**，使用 Rust + gpui 构建桌面 GUI，通过 LuaJIT 支撑业务逻辑。

**核心流程：** 输入提示词描述需求 → 选择设计风格（Skill） → AI 生成设计稿 → 预览 & 导出

---

## 架构

```
┌─────────────┐    ┌──────────────┐    ┌──────────────────┐
│  C main.c   │───▶│  LuaJIT 引擎  │───▶│  Rust GPU 渲染    │
│  (入口)     │    │  (业务逻辑)   │    │  (libmy_design    │
│             │    │              │    │   _gui.so)        │
│  dlopen .so │    │  main.lua    │    │                   │
│  → LuaJIT  │    │  gui.lua     │    │  gpui 框架        │
│  → run_gui │    │  skills/*    │    │  三栏布局         │
└─────────────┘    └──────────────┘    └──────────────────┘
       │                  │                     │
       └────── 3 层桥接 ──┴──── C ABI / FFI ────┘
```

### 分层说明

| 层 | 语言 | 职责 |
|----|------|------|
| **入口层** | C | 加载 .so、初始化 LuaJIT、调用 run_gui() |
| **业务逻辑层** | LuaJIT | Skill 管理、LLM 调用、协程调度、测试脚本 |
| **桥接层** | C + LuaJIT FFI | `opencode.*` 全局函数注册、gui_tick 定时器回呼 |
| **渲染层** | Rust (gpui) | GPU 加速窗口渲染、三栏布局、鼠标/键盘事件 |

---

## 项目结构

```
my_design/
├── gui_gpui/              # Rust GPU 渲染引擎
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # C ABI 导出 + DesignView 三栏布局
│       ├── widgets.rs      # 自封装 h_flex / v_flex 布局
│       └── components/     # 设计组件（预留）
├── skills/                # 设计 Skill 定义
│   ├── SKILLS.md          # Skill 使用指南
│   ├── web_landing.lua    # Web 着陆页
│   ├── material_you.lua   # Material Design 3
│   ├── ios_hig.lua        # iOS HIG
│   ├── ant_design.lua     # Ant Design 企业后台
│   └── shadcn.lua         # Shadcn/ui 现代 Web
├── main.lua               # 主入口：GUI 模式 + LLM 协程
├── gui.lua                # LuaJIT FFI 绑定（精简版）
├── json.lua               # JSON 工具（cjson 封装）
├── log.lua                # 日志模块
├── lua_engine.c           # C ↔ Lua 桥接（自包含，无外部依赖）
├── gui_tick.c / .h        # 定时器 / 协程调度
├── main.c                 # C 入口
├── Makefile               # 编译脚本
├── libmy_design_gui.so    # Rust 编译产物（35MB）
└── my_design              # 最终二进制（47KB）
```

---

## 界面布局

```
┌──────────────┬──────────────────────────────────┬──────────────┐
│  STYLES       │  Design Canvas                    │  PROPERTIES   │
│              │                                   │               │
│  [M] my_design│  ✨ What would you like to        │  DESIGN TOKENS│
│              │     design today?                  │  ■ Primary   │
│  🌐 Web      │                                   │  ■ Secondary │
│  📱 iOS App  │   Type your idea below and        │  ■ Background│
│  🟣 Material │   click Generate                   │  ■ Text      │
│  📊 Dashboard│                                   │  ■ Radius    │
│  🛍️ E-Commerce│                                   │               │
│  ⚙️ Settings │                                   │  EXPORT       │
│              │                                   │  ● React     │
│  AI UI       │                                   │  ● HTML+CSS  │
│  Designer    │                                   │  ● Figma     │
│  v0.1        │                                   │               │
│              │                                   │  RECENT       │
│              │                                   │  ● Landing v3│
│              │                                   │  ● Login scr │
│              │                                   │  ● Dashboard │
├──────────────┴──────────────────────────────────┴──────────────┤
│  [Web] [Mobile] [Dashboard]   Describe UI...     [✨ Generate] │
└───────────────────────────────────────────────────────────────┘
```

**设计特点：**
- 三栏专业布局（Figma/Sketch 风格）
- 浅色时尚主题（`#f5f7fa` 背景，`#6366f1` 紫色主色）
- 左侧 Style 导航 + 右侧 Design Token 属性面板
- 中央画布区 + 底部提示词输入 + Generate 按钮
- 0 外部 UI 库依赖，全部组件自封装

---

## Skill 机制

每个 Skill 是一组设计模式定义：

| 组成部分 | 说明 |
|---------|------|
| **Design Token** | 配色、间距、字体、圆角等设计变量 |
| **组件图谱** | 该场景下的典型 UI 组件组合 |
| **布局模板** | 页面结构的最佳实践 |
| **Prompt 模板** | 传递给 AI 的增强提示词 |

### 内置 Skills

| Skill | 风格 | 适用场景 |
|-------|------|---------|
| **iOS HIG** | Apple 原生 iOS | iPhone/iPad App |
| **Material Design 3** | Google Material You | Android / 跨平台移动端 |
| **Shadcn/ui** | 现代简约 Web | SaaS / Landing Page |
| **Ant Design** | 企业级中后台 | Dashboard / 数据管理 |
| **Web Landing** | 营销着陆页 | 产品官网 / 品牌展示 |

详见 [`skills/SKILLS.md`](./skills/SKILLS.md)。

---

## 快速开始

```bash
# 1. 编译 Rust GUI
make gui

# 2. 编译 C 入口 + Lua 引擎
make

# 3. 运行（需要桌面环境 X11/Wayland）
./my_design

# 自动化测试
MY_DESIGN_GUI_TEST_SCRIPT=./test.lua ./my_design
```

### 环境变量

| 变量 | 用途 |
|------|------|
| `MY_DESIGN_GUI_TEST_SCRIPT` | GUI 自动化测试脚本路径 |
| `MY_DESIGN_GUI_TEST_MSG` | 启动后自动发送的测试消息 |
| `OPENAI_API_KEY` | OpenAI API 密钥 |
| `OPENAI_BASE_URL` | API 地址（默认 `https://api.openai.com/v1`） |
| `OPENAI_MODEL` | 模型名（默认 `gpt-4o-mini`） |

---

## 技术栈

| 组件 | 方案 |
|------|------|
| GUI 框架 | gpui（Zed GPU 渲染引擎） |
| 窗口/事件 | gpui_platform |
| 组件库 | 全部自封装，零外部 UI 依赖 |
| 业务脚本 | LuaJIT 5.1 |
| AI API | OpenAI-compatible (HTTP POST) |
| 构建 | Makefile + Cargo |
| 二进制入口 | C (47KB) + Rust .so (35MB) |

---

## 许可证

基于 [Apache License 2.0](./LICENSE) 开源。
