-- imagen_prompt.lua — Google Imagen / Midjourney 提示词生成器
--
-- 两步走策略:
--   1. build_style_ref_prompt() → 生成"风格参考图"的 prompt
--   2. build_page_prompts()     → 每页 prompt + 引用风格参考图

local M = {}

local device_map = {
    mobile = "mobile app UI, smartphone screen",
    web = "web page UI, desktop browser viewport",
    tablet = "tablet app UI, iPad screen",
}

local style_map = {
    ["Material Design 3"] = "Material Design 3, dynamic color, rounded shapes, elevation",
    ["iOS HIG"] = "iOS Human Interface Guidelines, native iOS, SF Pro, frosted glass",
    ["Ant Design"] = "Ant Design, enterprise UI, professional, clean dashboard",
    ["Shadcn/ui"] = "modern web UI, Tailwind CSS, minimal, accessible components",
    ["Web Landing"] = "modern landing page, marketing design, conversion focused",
}

-- 从 Skill Token 构建关键词片段
local function build_keywords(skill)
    local t = skill.tokens or {}
    local c = t.colors or {}
    local typ = t.typography or {}
    local font = typ.font_family or typ.heading or "sans-serif"
    local r = t.radius
    local radius = type(r) == "table" and (r.lg or r.md or "12px") or tostring(r or "12px")

    return {
        primary = c.primary or "#6366f1",
        background = c.background or "#ffffff",
        text = c.text or c.on_background or "#1e1e1e",
        font = font,
        radius = radius,
        category = skill.category or "web",
        name = skill.name or "Modern UI",
    }
end

-- 第1步: 生成"风格参考图"的 prompt
-- 这张图展示设计系统的所有核心 UI 元素，作为后续页面的风格锚点
function M.build_style_ref_prompt(skill)
    local k = build_keywords(skill)
    local device = device_map[k.category] or device_map.web
    local style = style_map[skill.name] or skill.name

    return string.format([[
%s design system showcase

Core UI elements displayed together:
- Buttons (primary, secondary, outline, ghost)
- Cards with images and text
- Form inputs, checkboxes, toggles
- Navigation bar, tabs, bottom nav
- Typography hierarchy (headings, body, caption)

Design tokens:
- Primary color: %s
- Background: %s
- Text color: %s
- Font: %s
- Border radius: %s
- %s

Clean professional UI component library, perfect for mobile app,
consistent styling across all elements, modern design,
high quality, well organized, design system grid layout
]],
    style, k.primary, k.background, k.text, k.font, k.radius, device)
end

-- 第2步: 为每个页面生成 prompt（引用风格参考图）
function M.build_page_prompt(skill, page_name, page_description)
    local k = build_keywords(skill)
    local device = device_map[k.category] or device_map.web
    local style = style_map[skill.name] or skill.name

    local desc = page_description or ""
    if desc == "" then
        desc = string.format("Clean %s page with standard UI components", page_name)
    end

    return string.format([[
%s %s page

%s

Design tokens:
- Primary: %s
- Background: %s
- Text: %s
- Font: %s
- Radius: %s

%s

Style reference: use the same visual style as the design system reference image.
Consistent colors, typography, spacing, and component styling.
High quality UI design, production ready.
]],
    style, page_name, desc,
    k.primary, k.background, k.text, k.font, k.radius, device)
end

-- 批量生成所有页面的 prompt
function M.build_all(skill, pages)
    local result = {
        style_ref_prompt = M.build_style_ref_prompt(skill),
        pages = {},
    }
    for _, p in ipairs(pages) do
        table.insert(result.pages, {
            name = p.name,
            prompt = M.build_page_prompt(skill, p.name, p.prompt),
        })
    end
    return result
end

return M
