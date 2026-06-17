// my_design — LuaJIT 引擎（自包含版本）
//
// 注册 opencode.* 全局函数供 Lua 调用：
//   opencode.get_lua_state()  → 返回 lua_State* 指针（给 gui.run 传入）
//   opencode.set_clipboard()  → 复制文本到剪贴板
//   opencode.log_*()          → 日志输出

#define _GNU_SOURCE
#include "lua_engine.h"
#include <lauxlib.h>
#include <lualib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <dlfcn.h>

/* ── GUI 符号动态解析（从 libmy_design_gui.so 加载） ──────────── */

__attribute__((unused)) static void* gui_sym(const char* name) {
    return dlsym(RTLD_DEFAULT, name);
}

struct lua_engine {
    lua_State* L;
};

/* ── 日志 ───────────────────────────────────────────────────── */

static int l_log_info(lua_State* L) {
    fprintf(stderr, "[my_design] %s\n", luaL_checkstring(L, 1));
    return 0;
}

static int l_log_debug(lua_State* L) {
    const char* env = getenv("MY_DESIGN_LOG_LEVEL");
    if (env && strcmp(env, "debug") == 0)
        fprintf(stderr, "[DEBUG] %s\n", luaL_checkstring(L, 1));
    return 0;
}

static int l_log_warn(lua_State* L) {
    fprintf(stderr, "[WARN] %s\n", luaL_checkstring(L, 1));
    return 0;
}

static int l_log_error(lua_State* L) {
    fprintf(stderr, "[ERROR] %s\n", luaL_checkstring(L, 1));
    return 0;
}

/* ── 剪贴板 ─────────────────────────────────────────────────── */

static int l_set_clipboard(lua_State* L) {
    const char* text = luaL_checkstring(L, 1);
    const char* cmd = NULL;
    if (access("/usr/bin/wl-copy", X_OK) == 0) {
        cmd = "wl-copy";
    } else if (access("/usr/bin/xclip", X_OK) == 0) {
        cmd = "xclip -selection clipboard";
    } else if (access("/usr/bin/xsel", X_OK) == 0) {
        cmd = "xsel --clipboard --input";
    }
    if (!cmd) {
        // Write to a temp file as fallback
        FILE* f = fopen("/tmp/my_design_clipboard.txt", "w");
        if (f) { fputs(text, f); fclose(f); }
        lua_pushboolean(L, 0);
        lua_pushstring(L, "no clipboard utility, saved to /tmp/my_design_clipboard.txt");
        return 2;
    }
    FILE* pipe = popen(cmd, "w");
    if (!pipe) {
        lua_pushboolean(L, 0);
        lua_pushstring(L, "clipboard pipe failed");
        return 2;
    }
    fputs(text, pipe);
    pclose(pipe);
    lua_pushboolean(L, 1);
    return 1;
}

/* ── Lua State 指针（传给 Rust GUI 用于 tick 回调） ───────────── */

static int l_get_lua_state(lua_State* L) {
    lua_pushlightuserdata(L, L);
    return 1;
}

/* ── HTTP 辅助（通过 curl 命令，简单可靠） ────────────────────── */

static int l_http_post(lua_State* L) {
    const char* url  = luaL_checkstring(L, 1);
    const char* data = luaL_checkstring(L, 2);
    const char* auth = luaL_optstring(L, 3, "");

    // Build temp file for request body
    char reqfile[] = "/tmp/my_design_http_req_XXXXXX";
    int fd = mkstemp(reqfile);
    if (fd < 0) {
        lua_pushnil(L);
        lua_pushstring(L, "failed to create temp file");
        return 2;
    }
    if (write(fd, data, strlen(data)) < 0) { close(fd); unlink(reqfile); lua_pushnil(L); lua_pushstring(L, "write failed"); return 2; }
    close(fd);

    // Build curl command
    char cmd[4096];
    int n = snprintf(cmd, sizeof(cmd),
        "curl -s -X POST '%s'"
        " -H 'Content-Type: application/json'"
        " -H 'Authorization: Bearer %s'"
        " -d @%s"
        " 2>/dev/null",
        url, auth, reqfile);

    unlink(reqfile); // clean up temp file

    if (n >= (int)sizeof(cmd)) {
        lua_pushnil(L);
        lua_pushstring(L, "command too long");
        return 2;
    }

    // Execute and read response
    FILE* pipe = popen(cmd, "r");
    if (!pipe) {
        lua_pushnil(L);
        lua_pushstring(L, "curl pipe failed");
        return 2;
    }

    char buf[65536];
    size_t len = 0;
    while (len < sizeof(buf) - 1) {
        size_t r = fread(buf + len, 1, sizeof(buf) - 1 - len, pipe);
        if (r == 0) break;
        len += r;
    }
    buf[len] = '\0';
    int rc = pclose(pipe);

    if (rc != 0 && len == 0) {
        lua_pushnil(L);
        lua_pushstring(L, "curl request failed");
        return 2;
    }

    lua_pushlstring(L, buf, len);
    return 1;
}

/* ── 注册表 ─────────────────────────────────────────────────── */

static const luaL_Reg my_design_lib[] = {
    {"get_lua_state",  l_get_lua_state},
    {"set_clipboard",  l_set_clipboard},
    {"http_post",      l_http_post},
    {"log_info",       l_log_info},
    {"log_debug",      l_log_debug},
    {"log_warn",       l_log_warn},
    {"log_error",      l_log_error},
    {NULL, NULL}
};

/* ── lifecycle ──────────────────────────────────────────────── */

lua_engine_t* lua_engine_create(void) {
    lua_State* L = luaL_newstate();
    if (!L) return NULL;
    luaL_openlibs(L);

    lua_engine_t* e = malloc(sizeof(lua_engine_t));
    if (!e) { lua_close(L); return NULL; }
    e->L = L;

    // Register my_design C bindings as global "opencode" table
    luaL_newlib(L, my_design_lib);
    lua_setglobal(L, "opencode");

    // Set up package.cpath for cjson.so
    lua_getglobal(L, "package");
    lua_getfield(L, -1, "cpath");
    const char* old_cpath = lua_tostring(L, -1);
    char cpath[2048];
    if (old_cpath && *old_cpath)
        snprintf(cpath, sizeof(cpath), "/usr/local/lualib/?.so;%s", old_cpath);
    else
        snprintf(cpath, sizeof(cpath), "/usr/local/lualib/?.so");
    lua_pushstring(L, cpath);
    lua_setfield(L, -3, "cpath");
    lua_pop(L, 2);

    return e;
}

void lua_engine_free(lua_engine_t* e) {
    if (!e) return;
    if (e->L) lua_close(e->L);
    free(e);
}

lua_State* lua_engine_state(lua_engine_t* e) {
    return e ? e->L : NULL;
}

int lua_engine_dofile(lua_engine_t* e, const char* path) {
    if (!e || !path) return -1;
    return luaL_dofile(e->L, path);
}
