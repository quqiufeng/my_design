-- imagen_prompt.lua — Google Imagen 提示词生成器
--
-- 从 Skill Design Token + 页面描述 → Imagen 提示词
-- 每页一个 prompt，Skill 前缀相同保证风格统一

local M = {}

-- 平台/设备关键词映射
local device_map = {
    mobile = "mobile app UI, smartphone screen, 375x812 viewport",
    web = "web page UI, desktop browser, 1440x900 viewport",
    tablet = "tablet app UI, iPad screen, 1024x1366 viewport",
}

-- 风格体系映射
local style_map = {
    ["Material Design 3"] = "Material Design 3, dynamic color system, rounded shapes, elevation shadows",
    ["iOS HIG"] = "iOS Human Interface Guidelines, native iOS style, SF Pro typography, frosted glass",
    ["Ant Design"] = "Ant Design system, enterprise UI, professional data display, clean dashboard",
    ["Shadcn/ui"] = "modern web UI, Tailwind CSS, minimal design, accessible components, clean borders",
    ["web_landing"] = "modern landing page, marketing design, conversion focused, bold typography",
}

-- 从 Skill Token 构建 Imagen prompt 前缀（所有页面共用）
function M.build_skill_prefix(skill)
    local tokens = skill.tokens or {}
    local colors = tokens.colors or {}
    local typography = tokens.typography or {}
    local font_family = typography.font_family or typography.heading or "Inter, sans-serif"
    local radius = tokens.radius
    local radius_str = type(radius) == "table" and (radius.lg or radius.md or "12px") or tostring(radius or "12px")
    local category = skill.category or "web"
    local device = device_map[category] or device_map.web
    local style_desc = style_map[skill.name] or skill.name

    local lines = {
        style_desc,
        device,
        "",
        "Design Tokens:",
    }

    if colors.primary then
        table.insert(lines, "- Primary color: " .. colors.primary)
    end
    if colors.background then
        table.insert(lines, "- Background: " .. colors.background)
    end
    if colors.text or colors.on_background then
        table.insert(lines, "- Text color: " .. (colors.text or colors.on_background or "#1e1e1e"))
    end
    if colors.surface then
        table.insert(lines, "- Surface: " .. colors.surface)
    end

    table.insert(lines, "- Font family: " .. font_family)
    table.insert(lines, "- Border radius: " .. radius_str)
    table.insert(lines, "")
    table.insert(lines, "High quality UI design, clean layout, professional, well-organized,")
    table.insert(lines, "perfect for production, consistent design system.")

    return table.concat(lines, "\n")
end

-- 为指定页面生成完整 Imagen prompt
function M.build_page_prompt(skill, page_name, page_description)
    local prefix = M.build_skill_prefix(skill)
    local lines = {
        prefix,
        "",
        "---",
        "",
        "Page: " .. page_name,
        "",
        page_description or "Clean UI design page.",
    }
    return table.concat(lines, "\n")
end

-- 为项目所有页面生成 prompts（返回数组）
function M.build_all_prompts(skill, pages)
    local prompts = {}
    for _, p in ipairs(pages) do
        table.insert(prompts, {
            name = p.name,
            prompt = M.build_page_prompt(skill, p.name, p.prompt),
        })
    end
    return prompts
end

return M
