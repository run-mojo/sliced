#include "sds.h"
//#include "listpack.h"
#include "rax.h"
#include "redismodule.h"
#include <stdlib.h>

#include <stdint.h>

#define LP_INTBUF_SIZE 21 /* 20 digits of -2^63 + 1 null term = 21. */

/* lpInsert() where argument possible values: */
#define LP_BEFORE 0
#define LP_AFTER 1
#define LP_REPLACE 2

//unsigned char *lpNew(void);
//void lpFree(unsigned char *lp);
unsigned char *lpInsert(unsigned char *lp, unsigned char *ele, uint32_t size, unsigned char *p, int where, unsigned char **newp);
unsigned char *lpAppend(unsigned char *lp, unsigned char *ele, uint32_t size);
unsigned char *lpDelete(unsigned char *lp, unsigned char *p, unsigned char **newp);
//uint32_t lpLength(unsigned char *lp);
unsigned char *lpGet(unsigned char *p, int64_t *count, unsigned char *intbuf);
unsigned char *lpFirst(unsigned char *lp);
unsigned char *lpLast(unsigned char *lp);
unsigned char *lpNext(unsigned char *lp, unsigned char *p);
unsigned char *lpPrev(unsigned char *lp, unsigned char *p);
uint32_t lpBytes(unsigned char *lp);
unsigned char *lpSeek(unsigned char *lp, long index);

// void *SD_Alloc(size_t size) {
//     return zmalloc(size);
// }

// void *SD_Realloc(void *ptr, size_t size) {
//     return zrealloc(ptr, size);
// }

// void SD_Free(void *ptr) {
//     zfree(ptr);
// }

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

int Export_RedisModule_SubscribeToKeyspaceEvents(RedisModuleCtx *ctx, int types, RedisModuleNotificationFunc cb) {
    return RedisModule_SubscribeToKeyspaceEvents(ctx, types, cb);
}