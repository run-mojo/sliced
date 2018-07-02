#include "redismodule.h"

/**
 * Directly link with Redis' version of "malloc"
 *
 * @param size
 * @return
 */
void *zmalloc(size_t size);

/**
 * Directly link with Redis' version of "zrealloc"
 *
 * @param ptr
 * @param size
 * @return
 */
void *zrealloc(void *ptr, size_t size);

/**
 * Directly link with Redis' version of "free"
 * @param ptr
 */
void zfree(void *ptr);

// RedisModule_Init is defined as a static function and so won't be exported as
// a symbol. Export a version under a slightly different name so that we can
// get access to it from Rust.
int Export_RedisModule_Init(RedisModuleCtx *ctx, const char *name, int ver, int apiver) {
    return RedisModule_Init(ctx, name, ver, apiver);
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