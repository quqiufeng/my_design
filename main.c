// my_design — 主入口
//
// 启动流程：
//   1. 预加载 libmy_design_gui.so（RTLD_GLOBAL，使 LuaJIT FFI 能找到符号）
//   2. 创建 LuaJIT 引擎
//   3. 加载 main.lua
//   4. 调用 run_gui(session_id, project_root)

#include "lua_engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>
#include <unistd.h>

int main(int argc, char* argv[]) {
    const char* project_root = ".";
    const char* session_id  = "default";

    // Parse args
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--project") == 0 && i + 1 < argc)
            project_root = argv[++i];
        else if (strcmp(argv[i], "--session") == 0 && i + 1 < argc)
            session_id = argv[++i];
        else if (strcmp(argv[i], "--help") == 0) {
            printf("Usage: my_design [--project <path>] [--session <id>]\n");
            return 0;
        }
    }

    // 1. Pre-load the Rust GUI .so with RTLD_GLOBAL so LuaJIT FFI can find it
    const char* so_name = "libmy_design_gui.so";
    void* gui_so = dlopen(so_name, RTLD_NOW | RTLD_GLOBAL);
    if (!gui_so) {
        // Try LD_LIBRARY_PATH or relative path
        char alt_path[512];
        snprintf(alt_path, sizeof(alt_path), "./%s", so_name);
        gui_so = dlopen(alt_path, RTLD_NOW | RTLD_GLOBAL);
    }
    if (!gui_so) {
        fprintf(stderr, "Warning: cannot load %s: %s\n", so_name, dlerror());
        fprintf(stderr, "GUI will not be available.\n");
        // Continue without GUI (CLI mode fallback)
    }

    // 2. Create LuaJIT engine
    lua_engine_t* engine = lua_engine_create();
    if (!engine) {
        fprintf(stderr, "Failed to create LuaJIT engine\n");
        return 1;
    }

    // Set up search path for Lua modules (current dir first)
    lua_State* L = lua_engine_state(engine);
    lua_getglobal(L, "package");
    lua_getfield(L, -1, "path");
    const char* old_path = lua_tostring(L, -1);
    char new_path[4096];
    snprintf(new_path, sizeof(new_path), "./?.lua;./lua/?.lua;%s", old_path ? old_path : "");
    lua_pushstring(L, new_path);
    lua_setfield(L, -3, "path");
    lua_pop(L, 2);

    // 3. Load main.lua
    if (lua_engine_dofile(engine, "main.lua") != 0) {
        fprintf(stderr, "Failed to load main.lua: %s\n", lua_tostring(L, -1));
        lua_engine_free(engine);
        return 1;
    }

    // 4. Call run_gui(session_id, project_root)
    lua_getglobal(L, "run_gui");
    if (!lua_isfunction(L, -1)) {
        fprintf(stderr, "run_gui() not found in main.lua\n");
        lua_engine_free(engine);
        return 1;
    }

    lua_pushstring(L, session_id);
    lua_pushstring(L, project_root);

    fprintf(stderr, "[my_design] Starting GUI...\n");
    if (lua_pcall(L, 2, 1, 0) != LUA_OK) {
        fprintf(stderr, "run_gui() error: %s\n", lua_tostring(L, -1));
        lua_engine_free(engine);
        return 1;
    }

    const char* result = lua_tostring(L, -1);
    if (result) fprintf(stderr, "[my_design] Exit: %s\n", result);
    lua_pop(L, 1);

    lua_engine_free(engine);
    return 0;
}
