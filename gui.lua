-- LuaJIT FFI binding for libmy_design_gui.so

local ffi = require("ffi")
local json = require("json")
local log = require("log")

ffi.cdef[[
    void* gui_app_create(const char* config_json);
    void  gui_app_free(void* app);
    void free(void* p);

    void gui_on_user_message(
        void* app,
        void (*callback)(const char* session_id, const char* text, void* userdata),
        void* userdata
    );

    int gui_run(void* app, void* lua_state);

    void gui_stream_delta(void* app, const char* session_id, const char* delta);
    void gui_append_message(void* app, const char* session_id, const char* role, const char* text);

    void gui_set_pages(void* app, const char* pages_json);
    void gui_set_page_done(void* app, int index);
]]

local lib = ffi.load("my_design_gui", true)

local M = {}
M._apps = {}

local function make_user_callback(lua_handler)
    return ffi.cast("void (*)(const char*, const char*, void*)", function(session_id, text, userdata)
        local s = ffi.string(session_id)
        local t = ffi.string(text)
        local ok, err = pcall(lua_handler, s, t)
        if not ok then log.error("[GUI] handler error: %s", err) end
    end)
end

function M.create(config)
    config = config or {}
    local cfg_json = json.encode(config)
    local app = lib.gui_app_create(cfg_json)
    if app == nil then error("failed to create GUI app") end
    local handle = ffi.cast("void*", app)
    M._apps[handle] = { handle = handle, user_cb = nil }
    return handle
end

function M.free(app)
    local state = M._apps[app]
    if state then
        if state.user_cb then state.user_cb:free() end
        M._apps[app] = nil
    end
    lib.gui_app_free(app)
end

function M.on_user_message(app, handler)
    local state = M._apps[app]
    if not state then error("unknown GUI app") end
    if state.user_cb then state.user_cb:free() end
    state.user_cb = make_user_callback(handler)
    lib.gui_on_user_message(app, state.user_cb, nil)
end

function M.run(app)
    local L = opencode.get_lua_state()
    return lib.gui_run(app, L)
end

function M.stream_delta(app, session_id, delta)
    lib.gui_stream_delta(app, session_id, delta)
end

function M.append_message(app, session_id, role, text)
    lib.gui_append_message(app, session_id, role, text)
end

function M.set_pages(app, pages_json)
    lib.gui_set_pages(app, pages_json)
end

function M.set_page_done(app, index)
    lib.gui_set_page_done(app, index)
end

return M
