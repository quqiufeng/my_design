-- Skill: iOS Human Interface Guidelines (Apple)
local M = {}

M.id = "ios-hig"
M.name = "iOS Design (HIG)"
M.description = "Apple iOS Human Interface Guidelines — native iOS app design with standard components"
M.category = "mobile"

M.tokens = {
    colors = {
        primary = "#007aff",
        secondary = "#5856d6",
        tertiary = "#ff2d55",
        background = "#f2f2f7",
        grouped_bg = "#f2f2f7",
        surface = "#ffffff",
        elevated_bg = "#ffffff",
        separator = "#c6c6c8",
        opaque_separator = "#545458",
        label = "#000000",
        secondary_label = "#3c3c4399",
        tertiary_label = "#3c3c434d",
        quaternary_label = "#3c3c432e",
        link = "#007aff",
        system_blue = "#007aff", system_green = "#34c759", system_indigo = "#5856d6",
        system_orange = "#ff9500", system_pink = "#ff2d55", system_purple = "#af52de",
        system_red = "#ff3b30", system_teal = "#5ac8fa", system_yellow = "#ffcc00",
    },
    spacing = { xs = 4, sm = 8, md = 12, lg = 16, xl = 20, xxl = 24 },
    typography = {
        font_family = "'SF Pro', -apple-system, Helvetica Neue, sans-serif",
        mono_family = "'SF Mono', ui-monospace, monospace",
        large_title = { size = 34, weight = 700 },
        title1 = { size = 28, weight = 700 },
        title2 = { size = 22, weight = 700 },
        title3 = { size = 20, weight = 600 },
        headline = { size = 17, weight = 600 },
        body = { size = 17, weight = 400 },
        callout = { size = 16, weight = 400 },
        subhead = { size = 15, weight = 400 },
        footnote = { size = 13, weight = 400 },
        caption1 = { size = 12, weight = 400 },
        caption2 = { size = 11, weight = 400 },
    },
    radius = {
        sm = 5, md = 10, lg = 13, xl = 20,
        button = 14, corner_radius = 10,
    },
    safe_area = { top = 44, bottom = 34, left = 0, right = 0 },
    tab_bar_height = 50,
    nav_bar_height = 44,
}

M.components = {
    "NavigationBar", "TabBar", "TableView", "CollectionView",
    "Button", "TextField", "SearchBar", "SegmentedControl", "Slider",
    "Switch", "Stepper", "PageControl", "ProgressView", "ActivityIndicator",
    "AlertController", "ActionSheet", "Popover", "ModalPresentation",
}

M.layout_template = {
    { type = "nav", component = "NavigationBar", large_title = true },
    { type = "content", component = "TableView", style = "inset_grouped" },
    { type = "tab", component = "TabBar", items = 5 },
}

M.prompt_template = [[
You are an iOS designer specializing in Apple Human Interface Guidelines.

Design Tokens:
- Primary: {primary}
- Background: {background}
- Label: {label}
- Separator: {separator}
- Corner radius: {corner_radius}

iOS HIG Core Principles:
1. Clarity: text is legible, icons are precise, adornments are subtle
2. Deference: fluid motion and a clean interface help users understand content
3. Depth: visual layers and realistic motion convey hierarchy and vitality

iOS Navigation Patterns:
- Navigation Bar (large title preferred)
- Tab Bar (for flat navigation, 3-5 tabs)
- Modal presentation for focused tasks
- Push navigation for hierarchical content

iOS Layout Guidelines:
- Safe area insets (44pt top, 34pt bottom for iPhone with notch)
- Standard 16pt margins
- 8pt vertical grid
- Grouped table view for settings/info

Generate design for: {user_input}

Output:
1. iOS design approach and HIG compliance notes
2. Navigation structure (tab bar, nav bar)
3. View hierarchy
4. Color usage and SF Symbol recommendations
5. SwiftUI structure (or UIKit layout)
]]

return M
