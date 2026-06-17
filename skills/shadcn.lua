-- Skill: Shadcn/ui + Radix (Modern Web Components)
local M = {}

M.id = "shadcn-ui"
M.name = "Shadcn/ui Modern Web"
M.description = "Modern accessible web components using Radix UI primitives + Tailwind CSS styling"
M.category = "web"

M.tokens = {
    colors = {
        primary = "#18181b",
        primary_foreground = "#fafafa",
        secondary = "#f4f4f5",
        secondary_foreground = "#18181b",
        muted = "#f4f4f5",
        muted_foreground = "#71717a",
        accent = "#f4f4f5",
        accent_foreground = "#18181b",
        background = "#ffffff",
        foreground = "#09090b",
        card = "#ffffff",
        card_foreground = "#09090b",
        popover = "#ffffff",
        popover_foreground = "#09090b",
        border = "#e4e4e7",
        input = "#e4e4e7",
        ring = "#18181b",
        destructive = "#ef4444",
        destructive_foreground = "#fafafa",
        success = "#22c55e",
        warning = "#f59e0b",
        info = "#3b82f6",
    },
    spacing = { xs = 2, sm = 4, md = 8, lg = 12, xl = 16, xxl = 24 },
    typography = {
        font_family = "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        mono_family = "'JetBrains Mono', 'Fira Code', ui-monospace, monospace",
        h1 = { size = 36, weight = 800, tracking = "-0.03em" },
        h2 = { size = 30, weight = 700, tracking = "-0.02em" },
        h3 = { size = 24, weight = 600, tracking = "-0.01em" },
        h4 = { size = 20, weight = 600 },
        p = { size = 16, weight = 400, line_height = 1.75 },
        small = { size = 14, weight = 400 },
        muted = { size = 14, weight = 400 },
        code = { size = 14, weight = 400 },
    },
    radius = { sm = 4, md = 6, lg = 8, xl = 12 },
    shadow = {
        sm = "0 1px 2px 0 rgba(0,0,0,0.05)",
        md = "0 4px 6px -1px rgba(0,0,0,0.1)",
        lg = "0 10px 15px -3px rgba(0,0,0,0.1)",
        xl = "0 20px 25px -5px rgba(0,0,0,0.1)",
    },
}

M.components = {
    "Button", "Input", "Select", "Checkbox", "RadioGroup", "Switch",
    "Slider", "Card", "Dialog", "DropdownMenu", "Popover", "Tooltip",
    "Tabs", "Accordion", "Sheet", "Toast", "Table", "Badge", "Avatar",
    "Command (kbd)", "Calendar", "Form", "Separator", "Skeleton",
    "Alert", "Progress", "Skeleton", "Textarea",
}

M.layout_template = {
    { type = "nav", component = "HeaderNav", sticky = true },
    { type = "hero", component = "HeroSection" },
    { type = "features", component = "FeatureCards", cols = 3 },
    { type = "content", component = "ContentSection" },
    { type = "footer", component = "Footer" },
}

M.prompt_template = [[
You are a UI designer specializing in modern web design with Shadcn/ui + Radix + Tailwind CSS.

Design Tokens:
- Primary: {primary}
- Background: {background}
- Foreground: {foreground}
- Muted: {muted}
- Border: {border}
- Radius: {radius_md}

Shadcn/ui Design Principles:
1. Accessible by default (Radix UI primitives)
2. Copy-paste friendly components
3. Customizable with Tailwind CSS
4. Clean, minimal aesthetic
5. Consistent spacing and typography

Modern Web Patterns:
- Clean white/gray backgrounds with accent colors
- Subtle shadows and borders for depth
- Responsive grid layouts
- Card-based content organization
- Smooth transitions and hover states

Generate design for: {user_input}

Output:
1. Design approach
2. Component tree with Shadcn/ui components
3. Layout structure (responsive)
4. Color and typography application
5. React/Next.js component code with Tailwind classes
]]

return M
