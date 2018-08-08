#include "alloc.h"
#include "redismodule.h"
#include <stdlib.h>

// init with libc malloc
void* (*redis_malloc)(size_t) = malloc;
// init with libc realloc
void* (*redis_realloc)(void*,size_t) = realloc;
// init with libc free
void (*redis_free)(void*) = free;

/**
 * Rust Entry Point
 */
int RedisModule_DoLoad(RedisModuleCtx *ctx, RedisModuleString **argv, int argc);

/**
 * Redis Entry Point. We hook into C first so we can bootstrap the global allocator
 * with the imported RedisModule_Alloc, RedisModule_Realloc, RedisModule_Free functions.
 *
 * @param ctx
 * @param argv
 * @param argc
 * @return
 */
int RedisModule_OnLoad(RedisModuleCtx *ctx, RedisModuleString **argv, int argc) {
    if (RedisModule_Init(ctx,"slice/d",1,REDISMODULE_APIVER_1)
        == REDISMODULE_ERR) return REDISMODULE_ERR;

    redis_malloc = RedisModule_Alloc;
    redis_realloc = RedisModule_Realloc;
    redis_free = RedisModule_Free;

    return RedisModule_DoLoad(ctx, argv, argc);
}

// RedisModule_Init is defined as a static function and so won't be exported as
// a symbol. Export a version under a slightly different name so that we can
// get access to it from Rust.
int Export_RedisModule_Init(RedisModuleCtx *ctx, const char *name, int ver, int apiver) {
    int r = RedisModule_Init(ctx, name, ver, apiver);

    redis_malloc = RedisModule_Alloc;
    redis_realloc = RedisModule_Realloc;
    redis_free = RedisModule_Free;

    return r;
}

/**
 * Helper function to wire up a new Native Redis Type.
 *
 * @param ctx
 * @param name
 * @param encver
 * @param rdbload
 * @param rdbsave
 * @param aofrewrite
 * @param memusage
 * @param dig
 * @param freeFn
 * @return
 */
RedisModuleType *Export_RedisModule_CreateDataType(RedisModuleCtx *ctx,
                                                   const char *name,
                                                   int encver,
                                                   RedisModuleTypeLoadFunc rdbload,
                                                   RedisModuleTypeSaveFunc rdbsave,
                                                   RedisModuleTypeRewriteFunc aofrewrite,
                                                   RedisModuleTypeMemUsageFunc memusage,
                                                   RedisModuleTypeDigestFunc dig,
                                                   RedisModuleTypeFreeFunc freeFn) {
    RedisModuleTypeMethods tm = {
            .version = REDISMODULE_TYPE_METHOD_VERSION,
            .rdb_load = rdbload,
            .rdb_save = rdbsave,
            .aof_rewrite = aofrewrite,
            .mem_usage = memusage,
            .free = freeFn,
            .digest = dig
    };

    return RedisModule_CreateDataType(ctx, name, encver, &tm);
}

int Export_RedisModule_SubscribeToKeyspaceEvents(RedisModuleCtx *ctx, int types, RedisModuleNotificationFunc cb) {
    return RedisModule_SubscribeToKeyspaceEvents(ctx, types, cb);
}