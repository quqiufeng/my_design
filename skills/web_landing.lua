-- Skill: Web Landing Page
local M = {}

M.id = "web-landing"
M.name = "Web Landing Page"
M.description = "Modern web landing page with hero, features, CTA sections"
M.category = "web"

M.tokens = {
    colors = {
        primary = "#4f46e5",
        secondary = "#7c3aed",
        accent = "#f59e0b",
        background = "#ffffff",
        surface = "#f8fafc",
        text = "#1e1e1e",
        text_secondary = "#64748b",
    },
    spacing = { xs = 4, sm = 8, md = 16, lg = 24, xl = 48, "2xl" = 80 },
    typography = {
        heading = { family = "Inter, sans-serif", weight = 700, size = { h1 = 48, h2 = 36, h3 = 24 } },
        body = { family = "Inter, sans-serif", weight = 400, size = 16 },
    },
    radius = { sm = 4, md = 8, lg = 12, xl = 16 },
    shadows = {
        sm = "0 1px 2px rgba(0,0,0,0.05)",
        md = "0 4px 6px rgba(0,0,0,0.1)",
        lg = "0 10px 15px rgba(0,0,0,0.1)",
    },
}

M.components = {
    "NavigationBar",
    "HeroSection",
    "FeatureGrid",
    "TestimonialCard",
    "CTASection",
    "Footer",
}

M.layout_template = {
    { type = "nav", component = "NavigationBar" },
    { type = "hero", component = "HeroSection" },
    { type = "features", component = "FeatureGrid", cols = 3 },
    { type = "testimonials", component = "TestimonialCard", count = 3 },
    { type = "cta", component = "CTASection" },
    { type = "footer", component = "Footer" },
}

M.prompt_template = [[
You are a UI design expert specializing in modern web landing pages.

Design Tokens:
- Primary: {primary}
- Secondary: {secondary}
- Background: {background}
- Text: {text}
- Border radius: {radius_md}

Layout structure (in order):
1. Navigation bar (logo + nav links + CTA button)
2. Hero section (headline, subtext, primary CTA, hero image)
3. Features grid (3-column grid with icon, title, description)
4. Testimonials (3 customer quote cards)
5. CTA section (heading, subtext, action button)
6. Footer (links, social, copyright)

Generate design for: {user_input}

Output in this format:
---DESIGN---
## Design Description
[2-3 sentences about the design approach]

## Component Tree
```
[visual hierarchy]
```

## Key Design Variables
```css
:root {
  --primary: {primary};
  --spacing: 16px;
  ...
}
```

## HTML Structure
```html
[responsive HTML structure with Tailwind or CSS classes]
```
]]

return M
