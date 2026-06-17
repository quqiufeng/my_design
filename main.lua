-- my_design main Lua runtime: UI design generation tool
--
-- Architecture:
--   Lua business logic  →  LuaJIT FFI  →  Rust/gpui rendering (.so)
--   Skills define design patterns + tokens
--   LLM generates design descriptions → rendered as previews

local json = require("json")
local log = require("log")

-- Pending coroutines resumed by the GUI tick callback.
local pending_coroutines = {}

function add_pending_coroutine(co)
    table.insert(pending_coroutines, co)
end

-- Called by the Rust GUI timer ~60fps to resume sleeping Lua coroutines.
function gui_on_copy(text)
    if opencode.set_clipboard then
        opencode.set_clipboard(text)
    end
end

function gui_tick()
    local i = 1
    while i <= #pending_coroutines do
        local co = pending_coroutines[i]
        if coroutine.status(co) == "suspended" then
            local ok, err = coroutine.resume(co)
            if not ok then
                log.error("coroutine error: %s", err)
                table.remove(pending_coroutines, i)
            else
                i = i + 1
            end
        elseif coroutine.status(co) == "dead" then
            table.remove(pending_coroutines, i)
        else
            i = i + 1
        end
    end
end

-- Built-in Skills (design patterns)
local skills = {
    {
        id = "web-landing",
        name = "Web Landing Page",
        description = "Modern web landing page with hero, features, CTA sections",
        tokens = {
            colors = { primary = "#4f46e5", secondary = "#7c3aed", background = "#ffffff", text = "#1e1e1e" },
            spacing = { base = 4, lg = 8, xl = 16 },
            typography = { heading = "Inter, sans-serif", body = "Inter, sans-serif" },
            radius = "8px",
        },
        prompt_template = [[
You are a UI design expert. Generate a complete {skill_name} design.

Style tokens:
- Primary color: {primary}
- Background: {background}
- Text color: {text}
- Border radius: {radius}

User request: {user_input}

Output format:
1. Brief design description
2. Component tree (hierarchical)
3. Layout specification
4. Key CSS/design variables
5. HTML structure (for preview)
]]
    },
    {
        id = "mobile-ios",
        name = "Mobile App - iOS",
        description = "iOS-style mobile app interface following HIG",
        tokens = {
            colors = { primary = "#007aff", secondary = "#5856d6", background = "#f2f2f7", text = "#1e1e1e" },
            spacing = { base = 4, lg = 8, xl = 16 },
            typography = { heading = "SF Pro, sans-serif", body = "SF Pro, sans-serif" },
            radius = "12px",
        },
        prompt_template = [[
You are a UI design expert. Generate a complete {skill_name} design.

Style tokens:
- Primary color: {primary}
- Background: {background}
- Text color: {text}
- Border radius: {radius}

User request: {user_input}

Output format:
1. Brief design description
2. Component tree (hierarchical)
3. Layout specification
4. Key design variables
5. HTML structure (for preview)
]]
    },
    {
        id = "mobile-material",
        name = "Mobile App - Material",
        description = "Material Design 3 Android app interface",
        tokens = {
            colors = { primary = "#6750a4", secondary = "#625b71", background = "#fffbff", text = "#1c1b1f" },
            spacing = { base = 4, lg = 8, xl = 16 },
            typography = { heading = "Roboto, sans-serif", body = "Roboto, sans-serif" },
            radius = "16px",
        },
        prompt_template = [[
You are a UI design expert. Generate a complete {skill_name} design following Material Design 3.

Style tokens:
- Primary color: {primary}
- Background: {background}
- Text color: {text}
- Border radius: {radius}

User request: {user_input}

Output format:
1. Brief design description
2. Component tree (hierarchical)
3. Layout specification
4. Key design tokens
5. HTML structure (for preview)
]]
    },
    {
        id = "dashboard",
        name = "Dashboard / Admin",
        description = "Data dashboard with charts, tables, and metrics",
        tokens = {
            colors = { primary = "#2563eb", secondary = "#7c3aed", background = "#f8fafc", text = "#0f172a" },
            spacing = { base = 4, lg = 8, xl = 16 },
            typography = { heading = "Inter, sans-serif", body = "Inter, sans-serif" },
            radius = "8px",
        },
        prompt_template = [[
You are a UI design expert. Generate a complete {skill_name} design.

Style tokens:
- Primary color: {primary}
- Background: {background}
- Text color: {text}
- Border radius: {radius}

User request: {user_input}

Output format:
1. Brief design description
2. Component tree (hierarchical)
3. Layout specification
4. Key design variables
5. HTML structure (for preview)
]]
    },
}

function get_skills()
    return skills
end

function get_skill_by_id(id)
    for _, s in ipairs(skills) do
        if s.id == id then return s end
    end
    return skills[1] -- default to first
end

-- Build prompt from skill template + user input
function build_prompt(skill_id, user_input)
    local skill = get_skill_by_id(skill_id)
    local tpl = skill.prompt_template
    local result = tpl
    result = result:gsub("{skill_name}", skill.name)
    result = result:gsub("{primary}", skill.tokens.colors.primary)
    result = result:gsub("{secondary}", skill.tokens.colors.secondary)
    result = result:gsub("{background}", skill.tokens.colors.background)
    result = result:gsub("{text}", skill.tokens.colors.text)
    result = result:gsub("{radius}", skill.tokens.radius)
    result = result:gsub("{user_input}", user_input)
    return result, skill
end

-- Parse OpenAI response to extract design content
local function parse_openai_response(resp_json)
    local resp, err = json.decode(resp_json)
    if not resp then return nil, "invalid json: " .. tostring(err) end
    if resp.error then
        return nil, "api error: " .. (resp.error.message or resp.error.type or json.encode(resp.error))
    end
    local choice = resp.choices and resp.choices[1]
    if not choice then return nil, "no choices in response" end
    return (choice.message.content or ""), nil
end

-- Stream content in chunks via coroutine yield
local function stream_content(app, sid, content)
    if not content or content == "" then return end
    gui.append_message(app, sid, "assistant", "")
    local chunk_size = 20
    local pos = 1
    while pos <= #content do
        local next_pos = math.min(pos + chunk_size, #content + 1)
        while next_pos > pos do
            local byte = content:byte(next_pos)
            if byte == nil or byte < 128 or byte >= 192 then break end
            next_pos = next_pos - 1
        end
        local chunk = content:sub(pos, next_pos - 1)
        gui.stream_delta(app, sid, chunk)
        pos = next_pos
        coroutine.yield()
    end
end

function run_gui(session_id, project_ns)
    local gui = require("gui")
    local app = gui.create({
        title = "my_design - AI UI Designer",
        project_root = project_ns or ".",
    })

    gui.append_message(app, session_id, "assistant",
        "🎨 Welcome to my_design!\n\nDescribe the UI you want in the input box below, select a style from the left panel, and click Generate. I'll create a design for you!")

    gui.on_user_message(app, function(sid, text)
        gui.append_message(app, sid, "user", text)

        -- Create async coroutine
        local co = coroutine.create(function()
            local skill_id = "web-landing" -- default; UI will select
            local prompt_text, skill = build_prompt(skill_id, text)
            log.info("prompt built, skill: %s", skill.name)

            -- Call LLM API
            local base_url = os.getenv("OPENAI_BASE_URL") or "https://api.openai.com/v1"
            local url = base_url .. "/chat/completions"
            local api_key = os.getenv("OPENAI_API_KEY") or ""
            local model = os.getenv("OPENAI_MODEL") or "gpt-4o-mini"


            local request_body = {
                model = model,
                messages = {
                    { role = "system", content = "You are a UI/UX design expert. Generate clean, modern UI designs." },
                    { role = "user", content = prompt_text }
                },
                temperature = 0.7,
                max_tokens = 4096,
            }

            local body_json = json.encode(request_body)
            log.debug("request body: %s", body_json:sub(1, 300))

            local response, err = opencode.http_post(url, body_json, api_key)
            if not response then
                gui.append_message(app, sid, "assistant", "Error: " .. tostring(err))
                return
            end

            local content, parse_err = parse_openai_response(response)
            if parse_err then
                gui.append_message(app, sid, "assistant", "Parse error: " .. parse_err)
                return
            end

            -- Stream the design description
            stream_content(app, sid, content)
            gui.append_message(app, sid, "design", "---DESIGN_HTML---\n" .. (content or ""))

            gui.append_message(app, sid, "assistant", "✅ Design generated! You can refine it by sending another message.")
        end)

        table.insert(pending_coroutines, co)
        coroutine.resume(co)
    end)


    -- Automated GUI test script (optional)
    local test_script = os.getenv("MY_DESIGN_GUI_TEST_SCRIPT")
    if test_script then
        log.info("scheduling gui test script: %s", test_script)
        local test_co = coroutine.create(function()
            -- Wait briefly for GUI setup
            for _ = 1, 10 do coroutine.yield() end
            local test_env = {
                _G = _G,
                app = app,
                gui = gui,
                session_id = session_id,
                opencode = opencode,
                require = require,
                print = print,
                io = io,
                os = os,
                table = table,
                tostring = tostring,
                tonumber = tonumber,
                pcall = pcall,
                assert = assert,
                error = error,
                math = math,
                string = string,
                coroutine = coroutine,
            }
            local chunk, load_err = loadfile(test_script)
            if chunk then
                setfenv(chunk, test_env)
                local ok, res = pcall(chunk)
                if not ok then
                    log.error("gui test script error: %s", res)
                    print("TEST FAILED: " .. tostring(res))
                end
            else
                log.error("failed to load gui test script: %s", load_err)
                print("TEST FAILED: cannot load script: " .. tostring(load_err))
            end
        end)
        table.insert(pending_coroutines, test_co)
    end


    gui.run(app)
    gui.free(app)
end

log.info("my_design Lua runtime loaded")
