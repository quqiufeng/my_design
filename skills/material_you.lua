-- Skill: Material Design 3 (Material You / Google)
local M = {}

M.id = "material-you"
M.name = "Material Design 3"
M.description = "Google Material Design 3 with dynamic color, rounded shapes, and motion"
M.category = "mobile"

M.tokens = {
    colors = {
        primary = "#6750a4",
        on_primary = "#ffffff",
        primary_container = "#eaddff",
        secondary = "#625b71",
        on_secondary = "#ffffff",
        secondary_container = "#e8def8",
        tertiary = "#7d5260",
        background = "#fffbff",
        surface = "#fffbff",
        surface_variant = "#e7e0ec",
        error = "#b3261e",
        on_background = "#1c1b1f",
        on_surface = "#1c1b1f",
        outline = "#79747e",
    },
    spacing = { xs = 4, sm = 8, md = 16, lg = 24, xl = 32 },
    typography = {
        display = { family = "Google Sans, Roboto", weight = 400, size = 36 },
        headline = { family = "Google Sans, Roboto", weight = 400, size = 28 },
        title = { family = "Google Sans, Roboto", weight = 500, size = 22 },
        body = { family = "Roboto, sans-serif", weight = 400, size = 16 },
        label = { family = "Roboto, sans-serif", weight = 500, size = 14 },
    },
    radius = { sm = 4, md = 8, lg = 16, xl = 28 },
    elevation = { level0 = 0, level1 = 1, level2 = 3, level3 = 6, level4 = 8, level5 = 12 },
}

M.components = {
    "TopAppBar", "BottomAppBar", "NavigationBar", "NavigationDrawer",
    "FloatingActionButton", "Card", "BottomSheet", "Dialog",
    "TextField", "Switch", "Checkbox", "RadioButton", "Slider",
    "Chip", "Snackbar", "Banner", "ProgressIndicator",
}

M.layout_template = {
    { type = "app_bar", component = "TopAppBar", title = "Screen" },
    { type = "content", component = "ScrollableContent" },
    { type = "fab", component = "FloatingActionButton", icon = "+" },
    { type = "nav", component = "NavigationBar" },
}

M.prompt_template = [[
You are a UI designer specializing in Google Material Design 3 (Material You).

Design Tokens:
- Primary: {primary}
- Secondary: {secondary}
- Background: {background}
- Surface: {surface}
- Error: {error}
- On Background: {on_background}
- Border radius: {radius_lg}

Material Design 3 Principles:
1. Dynamic color based on wallpaper (use the provided palette)
2. Rounded, fluid shapes (M3 shape system)
3. Clear typography hierarchy (Google Sans / Roboto)
4. Meaningful motion and transitions
5. Elevation with shadows for depth

Key Components:
- Top App Bar (large or medium, with leading icon and actions)
- Cards with rounded corners and subtle elevation
- Floating Action Button (FAB) for primary action
- Bottom Navigation Bar with 3-5 destinations
- Text fields with filled or outlined style

Generate design for: {user_input}

Output format:
1. Design approach (Material Design 3 principles applied)
2. Component hierarchy
3. Layout grid (4dp grid system)
4. Color usage (primary, secondary, tertiary roles)
5. HTML/CSS structure with M3 styling
]]

return M
