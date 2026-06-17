// my_design — LuaJIT 引擎头文件（自包含版本）

#ifndef MY_DESIGN_LUA_ENGINE_H
#define MY_DESIGN_LUA_ENGINE_H

#include <lua.h>

#ifdef __cplusplus
extern "C" {
#endif

/* LuaJIT 引擎句柄 */
typedef struct lua_engine lua_engine_t;

/* 创建/销毁（无外部依赖） */
lua_engine_t* lua_engine_create(void);
void          lua_engine_free(lua_engine_t* e);

/* 获取底层 Lua state */
lua_State* lua_engine_state(lua_engine_t* e);

/* 加载并执行 Lua 文件 */
int lua_engine_dofile(lua_engine_t* e, const char* path);

#ifdef __cplusplus
}
#endif

#endif /* MY_DESIGN_LUA_ENGINE_H */
