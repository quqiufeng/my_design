# my_design Skills — 设计风格指南

> 每个 Skill 是一个完整的设计模式包，包含 **Design Token**（颜色/字体/间距）、**组件图谱**、**布局模板** 和 **AI Prompt 模板**。

---

## 📱 Mobile 移动端

### iOS Design (HIG) — `ios_hig.lua`

| 项目 | 说明 |
|------|------|
| **风格** | Apple 原生 iOS 风格，遵循 Human Interface Guidelines |
| **适用** | iPhone/iPad 原生 App、SwiftUI 界面 |
| **配色** | 系统蓝 `#007aff`、纯白背景 `#f2f2f7`、SF Pro 字体 |
| **特点** | Safe area 适配、Tab Bar 导航、大标题导航栏、Grouped TableView |
| **典型页面** | 设置页、消息列表、个人主页、媒体浏览 |

**示例 Prompt:**
> "一个 iOS 风格的电商商品详情页，包含大标题导航、商品轮播图、规格选择器和底部购买栏"

---

### Material Design 3 — `material_you.lua`

| 项目 | 说明 |
|------|------|
| **风格** | Google Material Design 3 (Material You)，动态取色 + 圆润造型 |
| **适用** | Android App、跨平台移动应用 |
| **配色** | 紫色主色 `#6750a4`、暖白背景 `#fffbff`、Google Sans 字体 |
| **特点** | 动态颜色系统、M3 形状系统（大圆角）、FAB、底部导航 |
| **典型页面** | 首页 Feed、详情页、设置页、搜索页 |

**示例 Prompt:**
> "Material Design 3 风格的音乐播放器界面，包含专辑封面、播放控制条、歌词滚动和底部播放列表"

---

## 🌐 Web 网页端

### Shadcn/ui Modern Web — `shadcn.lua`

| 项目 | 说明 |
|------|------|
| **风格** | 现代简约 Web 风格，Shadcn/ui + Radix + Tailwind CSS |
| **适用** | SaaS 产品、落地页、博客、Web 应用 |
| **配色** | 黑/白极简 `#18181b` / `#ffffff`、Inter 字体 |
| **特点** | 可访问性优先（Radix 原语）、干净留白、微妙阴影 |
| **典型页面** | 产品 Landing Page、定价页、Dashboard、文档页 |

**示例 Prompt:**
> "一个现代化的 SaaS 定价页面，包含三个价格档位卡片、功能对比表和 CTA 按钮"

---

### Ant Design Pro — `ant_design.lua`

| 项目 | 说明 |
|------|------|
| **风格** | 企业级中后台设计系统，数据密集、功能复杂 |
| **适用** | 管理后台、数据分析平台、企业级 Web 应用 |
| **配色** | 科技蓝 `#1677ff`、浅灰背景 `#f5f5f5` |
| **特点** | ProLayout 侧边栏、ProTable 数据表格、复杂表单、权限管理 |
| **典型页面** | 数据 Dashboard、用户管理、订单列表、配置页 |

**示例 Prompt:**
> "Ant Design 风格的数据分析 Dashboard，包含统计卡片、折线图、数据表格和筛选器"

---

### Web Landing Page — `web_landing.lua`

| 项目 | 说明 |
|------|------|
| **风格** | 现代营销着陆页，高转化率设计 |
| **适用** | 产品官网、营销页面、品牌展示 |
| **配色** | 靛蓝主色 `#4f46e5`、白色背景、Inter 字体 |
| **特点** | Hero 大标题、特性网格、客户评价、CTA 行动号召 |
| **典型页面** | 产品首页、App 推广页、活动注册页 |

**示例 Prompt:**
> "一个 AI 产品的着陆页，包含 Hero 视频区、三个核心特性介绍、客户评价轮播和底部 CTA"

---

## 🎯 如何选择 Skill

```
你想设计什么？
│
├─ 📱 移动 App
│   ├─ iOS 原生风格  →  ios_hig
│   └─ Material 风格  →  material_you
│
├─ 🌐 Web 页面
│   ├─ 现代简约 Web  →  shadcn
│   ├─ 企业后台/数据  →  ant_design
│   └─ 营销着陆页    →  web_landing
│
└─ 🔧 自定义
    └─ 在 skills/ 目录创建新的 .lua 文件
```

---

## 🧩 Skill 文件结构

每个 Skill 是一个 Lua 文件，包含以下字段：

```lua
{
    id = "skill-id",           -- 唯一标识
    name = "Skill 名称",
    description = "简要描述",
    category = "mobile|web",

    tokens = {                 -- Design Token
        colors = { ... },
        typography = { ... },
        spacing = { ... },
        radius = { ... },
    },

    components = { ... },      -- 可用组件清单

    layout_template = { ... }, -- 页面布局模板

    prompt_template = [[...]], -- AI 生成提示词模板
}
```

---

## 🚀 使用方式

在 my_design 中输入提示词时，Skill 会自动：
1. 提供 **Design Token** 给 AI，确保生成的设计风格统一
2. 提供 **组件参考**，引导 AI 使用合适的 UI 元素
3. 提供 **布局建议**，保证页面结构合理
4. 优化 **Prompt 模板**，让 AI 输出更精确的设计描述

---

*更多 Skill 持续添加中...*
