-- Skill: Ant Design (Alibaba / Ant Group)
local M = {}

M.id = "ant-design"
M.name = "Ant Design Pro"
M.description = "Enterprise-grade UI design system with detailed data display, complex forms, and dashboards"
M.category = "web"

M.tokens = {
    colors = {
        primary = "#1677ff",
        primary_light = "#e6f4ff",
        success = "#52c41a",
        warning = "#faad14",
        error = "#ff4d4f",
        info = "#1677ff",
        background = "#f5f5f5",
        component_bg = "#ffffff",
        text = "#000000d9",
        text_secondary = "#00000073",
        text_tertiary = "#00000040",
        border = "#d9d9d9",
        border_secondary = "#f0f0f0",
    },
    spacing = { xs = 4, sm = 8, md = 16, lg = 24, xl = 32, xxl = 48 },
    typography = {
        font_family = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
        code_family = "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
        h1 = { size = 38, weight = 600 },
        h2 = { size = 30, weight = 600 },
        h3 = { size = 24, weight = 600 },
        h4 = { size = 20, weight = 500 },
        h5 = { size = 16, weight = 500 },
        body = { size = 14, weight = 400 },
        small = { size = 12, weight = 400 },
    },
    radius = { sm = 4, md = 6, lg = 8, xl = 12 },
    shadow = {
        sm = "0 1px 2px 0 rgba(0,0,0,0.03)",
        md = "0 3px 6px -4px rgba(0,0,0,0.12)",
        lg = "0 6px 16px -8px rgba(0,0,0,0.08)",
        xl = "0 12px 48px 16px rgba(0,0,0,0.03)",
    },
    breakpoints = { xs = 480, sm = 576, md = 768, lg = 992, xl = 1200, xxl = 1600 },
}

M.components = {
    "Button (Primary/Default/Dashed/Link/Text)", "Table", "Form", "Modal", "Drawer",
    "Menu", "Breadcrumb", "Steps", "Tabs", "Card", "Collapse", "Descriptions",
    "Statistic", "Progress", "Tag", "Badge", "Alert", "Message", "Notification",
    "Popover", "Tooltip", "Select", "DatePicker", "Upload", "Tree", "Dropdown",
}

M.layout_template = {
    { type = "layout", component = "ProLayout", sidebar = true },
    { type = "header", component = "Header", actions = true },
    { type = "content", component = "PageContainer" },
    { type = "table", component = "ProTable", features = "search, export, columns" },
    { type = "footer", component = "Footer" },
}

M.prompt_template = [[
You are a UI designer specializing in Ant Design (enterprise design system).

Design Tokens:
- Primary: {primary}
- Success: {success}
- Warning: {warning}
- Error: {error}
- Background: {background}
- Text: {text}
- Border radius: {radius_md}

Ant Design Principles:
1. Natural: interactions should feel intuitive and natural
2. Certain: make decisions clear and outcomes predictable
3. Meaningful: every element serves a purpose
4. Growing: learn from users and evolve
5. Efficient: simplify complex tasks

Common Enterprise Patterns:
- ProLayout with sidebar navigation + header
- Data-heavy tables with search, filter, pagination
- Complex forms with validation wizards
- Dashboard with statistic cards and charts
- Drawer/Modal for secondary content

Generate design for: {user_input}

Output:
1. Design system application
2. Page structure / layout
3. Component selection rationale
4. Data display patterns
5. HTML/React structure with Ant Design components
]]

return M
