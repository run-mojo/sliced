/*
 * Copyright (c) 2017, Salvatore Sanfilippo <antirez at gmail dot com>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 *   * Redistributions of source code must retain the above copyright notice,
 *     this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in the
 *     documentation and/or other materials provided with the distribution.
 *   * Neither the name of Redis nor the names of its contributors may be used
 *     to endorse or promote products derived from this software without
 *     specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

#include "endianconv.h"
#include "stream.h"

#define STREAM_BYTES_PER_LISTPACK 2048

/* Every stream item inside the listpack, has a flags field that is used to
 * mark the entry as deleted, or having the same field as the "master"
 * entry at the start of the listpack> */
#define STREAM_ITEM_FLAG_NONE 0             /* No special flags. */
#define STREAM_ITEM_FLAG_DELETED (1<<0)     /* Entry is delted. Skip it. */
#define STREAM_ITEM_FLAG_SAMEFIELDS (1<<1)  /* Same fields as master entry. */



void streamFreeCG(streamCG *cg);
void streamFreeNACK(streamNACK *na);


/* -----------------------------------------------------------------------
 * Low level stream encoding: a radix tree of listpacks.
 * ----------------------------------------------------------------------- */

/* Create a new stream data structure. */
stream *streamNew(void) {
    stream *s = zmalloc(sizeof(*s));
    s->rax = raxNew();
    s->length = 0;
    s->last_id.ms = 0;
    s->last_id.seq = 0;
    s->cgroups = NULL; /* Created on demand to save memory when not used. */
    return s;
}

/* Free a stream, including the listpacks stored inside the radix tree. */
void freeStream(stream *s) {
    raxFreeWithCallback(s->rax,(void(*)(void*))lpFree);
    if (s->cgroups)
        raxFreeWithCallback(s->cgroups,(void(*)(void*))streamFreeCG);
    zfree(s);
}

/* Generate the next stream item ID given the previous one. If the current
 * milliseconds Unix time is greater than the previous one, just use this
 * as time part and start with sequence part of zero. Otherwise we use the
 * previous time (and never go backward) and increment the sequence. */
void streamNextID(streamID *last_id, streamID *new_id) {
    uint64_t ms = mstime();
    if (ms > last_id->ms) {
        new_id->ms = ms;
        new_id->seq = 0;
    } else {
        new_id->ms = last_id->ms;
        new_id->seq = last_id->seq+1;
    }
}

/* This is just a wrapper for lpAppend() to directly use a 64 bit integer
 * instead of a string. */
unsigned char *lpAppendInteger(unsigned char *lp, int64_t value) {
    char buf[LONG_STR_SIZE];
    int slen = ll2string(buf,sizeof(buf),value);
    return lpAppend(lp,(unsigned char*)buf,slen);
}

/* This is just a wrapper for lpReplace() to directly use a 64 bit integer
 * instead of a string to replace the current element. The function returns
 * the new listpack as return value, and also updates the current cursor
 * by updating '*pos'. */
unsigned char *lpReplaceInteger(unsigned char *lp, unsigned char **pos, int64_t value) {
    char buf[LONG_STR_SIZE];
    int slen = ll2string(buf,sizeof(buf),value);
    return lpInsert(lp, (unsigned char*)buf, slen, *pos, LP_REPLACE, pos);
}

/* This is a wrapper function for lpGet() to directly get an integer value
 * from the listpack (that may store numbers as a string), converting
 * the string if needed. */
int64_t lpGetInteger(unsigned char *ele) {
    int64_t v;
    unsigned char *e = lpGet(ele,&v,NULL);
    if (e == NULL) return v;
    /* The following code path should never be used for how listpacks work:
     * they should always be able to store an int64_t value in integer
     * encoded form. However the implementation may change. */
    long long ll;
    int retval = string2ll((char*)e,v,&ll);
    serverAssert(retval != 0);
    v = ll;
    return v;
}

/* Debugging function to log the full content of a listpack. Useful
 * for development and debugging. */
void streamLogListpackContent(unsigned char *lp) {
    unsigned char *p = lpFirst(lp);
    while(p) {
        unsigned char buf[LP_INTBUF_SIZE];
        int64_t v;
        unsigned char *ele = lpGet(p,&v,buf);
//        serverLog(LL_WARNING,"- [%d] '%.*s'", (int)v, (int)v, ele);
        p = lpNext(lp,p);
    }
}

/* Convert the specified stream entry ID as a 128 bit big endian number, so
 * that the IDs can be sorted lexicographically. */
void streamEncodeID(void *buf, streamID *id) {
    uint64_t e[2];
    e[0] = htonu64(id->ms);
    e[1] = htonu64(id->seq);
    memcpy(buf,e,sizeof(e));
}

/* This is the reverse of streamEncodeID(): the decoded ID will be stored
 * in the 'id' structure passed by reference. The buffer 'buf' must point
 * to a 128 bit big-endian encoded ID. */
void streamDecodeID(void *buf, streamID *id) {
    uint64_t e[2];
    memcpy(e,buf,sizeof(e));
    id->ms = ntohu64(e[0]);
    id->seq = ntohu64(e[1]);
}

/* Compare two stream IDs. Return -1 if a < b, 0 if a == b, 1 if a > b. */
int streamCompareID(streamID *a, streamID *b) {
    if (a->ms > b->ms) return 1;
    else if (a->ms < b->ms) return -1;
        /* The ms part is the same. Check the sequence part. */
    else if (a->seq > b->seq) return 1;
    else if (a->seq < b->seq) return -1;
    /* Everything is the same: IDs are equal. */
    return 0;
}

/* Adds a new item into the stream 's' having the specified number of
 * field-value pairs as specified in 'numfields' and stored into 'argv'.
 * Returns the new entry ID populating the 'added_id' structure.
 *
 * If 'use_id' is not NULL, the ID is not auto-generated by the function,
 * but instead the passed ID is used to add the new entry. In this case
 * adding the entry may fail as specified later in this comment.
 *
 * The function returns C_OK if the item was added, this is always true
 * if the ID was generated by the function. However the function may return
 * C_ERR if an ID was given via 'use_id', but adding it failed since the
 * current top ID is greater or equal. */
int streamAppendItem(stream *s, robj **argv, int64_t numfields, streamID *added_id, streamID *use_id) {
    /* If an ID was given, check that it's greater than the last entry ID
     * or return an error. */
    if (use_id && streamCompareID(use_id,&s->last_id) <= 0) return C_ERR;

    /* Add the new entry. */
    raxIterator ri;
    raxStart(&ri,s->rax);
    raxSeek(&ri,"$",NULL,0);

//    **sds argv2 = (**sds)argv;

    size_t lp_bytes = 0;        /* Total bytes in the tail listpack. */
    unsigned char *lp = NULL;   /* Tail listpack pointer. */

    /* Get a reference to the tail node listpack. */
    if (raxNext(&ri)) {
        lp = ri.data;
        lp_bytes = lpBytes(lp);
    }
    raxStop(&ri);

    /* Generate the new entry ID. */
    streamID id;
    if (use_id)
        id = *use_id;
    else
        streamNextID(&s->last_id,&id);

    /* We have to add the key into the radix tree in lexicographic order,
     * to do so we consider the ID as a single 128 bit number written in
     * big endian, so that the most significant bytes are the first ones. */
    uint64_t rax_key[2];    /* Key in the radix tree containing the listpack.*/
    streamID master_id;     /* ID of the master entry in the listpack. */

    /* Create a new listpack and radix tree node if needed. Note that when
     * a new listpack is created, we populate it with a "master entry". This
     * is just a set of fields that is taken as refernce in order to compress
     * the stream entries that we'll add inside the listpack.
     *
     * Note that while we use the first added entry fields to create
     * the master entry, the first added entry is NOT represented in the master
     * entry, which is a stand alone object. But of course, the first entry
     * will compress well because it's used as reference.
     *
     * The master entry is composed like in the following example:
     *
     * +-------+---------+------------+---------+--/--+---------+---------+-+
     * | count | deleted | num-fields | field_1 | field_2 | ... | field_N |0|
     * +-------+---------+------------+---------+--/--+---------+---------+-+
     *
     * count and deleted just represent respectively the total number of
     * entries inside the listpack that are valid, and marked as deleted
     * (delted flag in the entry flags set). So the total number of items
     * actually inside the listpack (both deleted and not) is count+deleted.
     *
     * The real entries will be encoded with an ID that is just the
     * millisecond and sequence difference compared to the key stored at
     * the radix tree node containing the listpack (delta encoding), and
     * if the fields of the entry are the same as the master enty fields, the
     * entry flags will specify this fact and the entry fields and number
     * of fields will be omitted (see later in the code of this function).
     *
     * The "0" entry at the end is the same as the 'lp-count' entry in the
     * regular stream entries (see below), and marks the fact that there are
     * no more entries, when we scan the stream from right to left. */

    /* First of all, check if we can append to the current macro node or
     * if we need to switch to the next one. 'lp' will be set to NULL if
     * the current node is full. */
    if (lp != NULL) {
//        if (server.stream_node_max_bytes &&
//            lp_bytes > server.stream_node_max_bytes)
//        {
//            lp = NULL;
//        } else if (server.stream_node_max_entries) {
//            int64_t count = lpGetInteger(lpFirst(lp));
//            if (count > server.stream_node_max_entries) lp = NULL;
//        }
       if (OBJ_STREAM_NODE_MAX_BYTES &&
               lp_bytes > OBJ_STREAM_NODE_MAX_BYTES)
        {
            lp = NULL;
        } else if (OBJ_STREAM_NODE_MAX_ENTRIES) {
            int64_t count = lpGetInteger(lpFirst(lp));
            if (count > OBJ_STREAM_NODE_MAX_ENTRIES) lp = NULL;
        }
    }

    int flags = STREAM_ITEM_FLAG_NONE;
//    if (lp == NULL || lp_bytes > server.stream_node_max_bytes) {
    if (lp == NULL || lp_bytes > OBJ_STREAM_NODE_MAX_BYTES) {
        master_id = id;
        streamEncodeID(rax_key,&id);
        /* Create the listpack having the master entry ID and fields. */
        lp = lpNew();
        lp = lpAppendInteger(lp,1); /* One item, the one we are adding. */
        lp = lpAppendInteger(lp,0); /* Zero deleted so far. */
        lp = lpAppendInteger(lp,numfields);
        for (int64_t i = 0; i < numfields; i++) {
            sds field = argv[i*2]->ptr;
            lp = lpAppend(lp,(unsigned char*)field,sdslen(field));
        }
        lp = lpAppendInteger(lp,0); /* Master entry zero terminator. */
        raxInsert(s->rax,(unsigned char*)&rax_key,sizeof(rax_key),lp,NULL);
        /* The first entry we insert, has obviously the same fields of the
         * master entry. */
        flags |= STREAM_ITEM_FLAG_SAMEFIELDS;
    } else {
        serverAssert(ri.key_len == sizeof(rax_key));
        memcpy(rax_key,ri.key,sizeof(rax_key));

        /* Read the master ID from the radix tree key. */
        streamDecodeID(rax_key,&master_id);
        unsigned char *lp_ele = lpFirst(lp);

        /* Update count and skip the deleted fields. */
        int64_t count = lpGetInteger(lp_ele);
        lp = lpReplaceInteger(lp,&lp_ele,count+1);
        lp_ele = lpNext(lp,lp_ele); /* seek deleted. */
        lp_ele = lpNext(lp,lp_ele); /* seek master entry num fields. */

        /* Check if the entry we are adding, have the same fields
         * as the master entry. */
        int64_t master_fields_count = lpGetInteger(lp_ele);
        lp_ele = lpNext(lp,lp_ele);
        if (numfields == master_fields_count) {
            int64_t i;
            for (i = 0; i < master_fields_count; i++) {
                sds field = argv[i*2]->ptr;
                int64_t e_len;
                unsigned char buf[LP_INTBUF_SIZE];
                unsigned char *e = lpGet(lp_ele,&e_len,buf);
                /* Stop if there is a mismatch. */
                if (sdslen(field) != (size_t)e_len ||
                    memcmp(e,field,e_len) != 0) break;
                lp_ele = lpNext(lp,lp_ele);
            }
            /* All fields are the same! We can compress the field names
             * setting a single bit in the flags. */
            if (i == master_fields_count) flags |= STREAM_ITEM_FLAG_SAMEFIELDS;
        }
    }

    /* Populate the listpack with the new entry. We use the following
     * encoding:
     *
     * +-----+--------+----------+-------+-------+-/-+-------+-------+--------+
     * |flags|entry-id|num-fields|field-1|value-1|...|field-N|value-N|lp-count|
     * +-----+--------+----------+-------+-------+-/-+-------+-------+--------+
     *
     * However if the SAMEFIELD flag is set, we have just to populate
     * the entry with the values, so it becomes:
     *
     * +-----+--------+-------+-/-+-------+--------+
     * |flags|entry-id|value-1|...|value-N|lp-count|
     * +-----+--------+-------+-/-+-------+--------+
     *
     * The entry-id field is actually two separated fields: the ms
     * and seq difference compared to the master entry.
     *
     * The lp-count field is a number that states the number of listpack pieces
     * that compose the entry, so that it's possible to travel the entry
     * in reverse order: we can just start from the end of the listpack, read
     * the entry, and jump back N times to seek the "flags" field to read
     * the stream full entry. */
    lp = lpAppendInteger(lp,flags);
    lp = lpAppendInteger(lp,id.ms - master_id.ms);
    lp = lpAppendInteger(lp,id.seq - master_id.seq);
    if (!(flags & STREAM_ITEM_FLAG_SAMEFIELDS))
        lp = lpAppendInteger(lp,numfields);
    for (int64_t i = 0; i < numfields; i++) {
        sds field = argv[i*2]->ptr, value = argv[i*2+1]->ptr;
        if (!(flags & STREAM_ITEM_FLAG_SAMEFIELDS))
            lp = lpAppend(lp,(unsigned char*)field,sdslen(field));
        lp = lpAppend(lp,(unsigned char*)value,sdslen(value));
    }
    /* Compute and store the lp-count field. */
    int64_t lp_count = numfields;
    lp_count += 3; /* Add the 3 fixed fields flags + ms-diff + seq-diff. */
    if (!(flags & STREAM_ITEM_FLAG_SAMEFIELDS)) {
        /* If the item is not compressed, it also has the fields other than
         * the values, and an additional num-fileds field. */
        lp_count += numfields+1;
    }
    lp = lpAppendInteger(lp,lp_count);

    /* Insert back into the tree in order to update the listpack pointer. */
    raxInsert(s->rax,(unsigned char*)&rax_key,sizeof(rax_key),lp,NULL);
    s->length++;
    s->last_id = id;
    if (added_id) *added_id = id;
    return C_OK;
}

/* Trim the stream 's' to have no more than maxlen elements, and return the
 * number of elements removed from the stream. The 'approx' option, if non-zero,
 * specifies that the trimming must be performed in a approximated way in
 * order to maximize performances. This means that the stream may contain
 * more elements than 'maxlen', and elements are only removed if we can remove
 * a *whole* node of the radix tree. The elements are removed from the head
 * of the stream (older elements).
 *
 * The function may return zero if:
 *
 * 1) The stream is already shorter or equal to the specified max length.
 * 2) The 'approx' option is true and the head node had not enough elements
 *    to be deleted, leaving the stream with a number of elements >= maxlen.
 */
int64_t streamTrimByLength(stream *s, size_t maxlen, int approx) {
    if (s->length <= maxlen) return 0;

    raxIterator ri;
    raxStart(&ri,s->rax);
    raxSeek(&ri,"^",NULL,0);

    int64_t deleted = 0;
    while(s->length > maxlen && raxNext(&ri)) {
        unsigned char *lp = ri.data, *p = lpFirst(lp);
        int64_t entries = lpGetInteger(p);

        /* Check if we can remove the whole node, and still have at
         * least maxlen elements. */
        if (s->length - entries >= maxlen) {
            lpFree(lp);
            raxRemove(s->rax,ri.key,ri.key_len,NULL);
            raxSeek(&ri,">=",ri.key,ri.key_len);
            s->length -= entries;
            deleted += entries;
            continue;
        }

        /* If we cannot remove a whole element, and approx is true,
         * stop here. */
        if (approx) break;

        /* Otherwise, we have to mark single entries inside the listpack
         * as deleted. We start by updating the entries/deleted counters. */
        int64_t to_delete = s->length - maxlen;
        serverAssert(to_delete < entries);
        lp = lpReplaceInteger(lp,&p,entries-to_delete);
        p = lpNext(lp,p); /* Seek deleted field. */
        int64_t marked_deleted = lpGetInteger(p);
        lp = lpReplaceInteger(lp,&p,marked_deleted+to_delete);
        p = lpNext(lp,p); /* Seek num-of-fields in the master entry. */

        /* Skip all the master fields. */
        int64_t master_fields_count = lpGetInteger(p);
        p = lpNext(lp,p); /* Seek the first field. */
        for (int64_t j = 0; j < master_fields_count; j++)
            p = lpNext(lp,p); /* Skip all master fields. */
        p = lpNext(lp,p); /* Skip the zero master entry terminator. */

        /* 'p' is now pointing to the first entry inside the listpack.
         * We have to run entry after entry, marking entries as deleted
         * if they are already not deleted. */
        while(p) {
            int flags = lpGetInteger(p);
            int to_skip;

            /* Mark the entry as deleted. */
            if (!(flags & STREAM_ITEM_FLAG_DELETED)) {
                flags |= STREAM_ITEM_FLAG_DELETED;
                lp = lpReplaceInteger(lp,&p,flags);
                deleted++;
                s->length--;
                if (s->length <= maxlen) break; /* Enough entries deleted. */
            }

            p = lpNext(lp,p); /* Skip ID ms delta. */
            p = lpNext(lp,p); /* Skip ID seq delta. */
            p = lpNext(lp,p); /* Seek num-fields or values (if compressed). */
            if (flags & STREAM_ITEM_FLAG_SAMEFIELDS) {
                to_skip = master_fields_count;
            } else {
                to_skip = lpGetInteger(p);
                to_skip = 1+(to_skip*2);
            }

            while(to_skip--) p = lpNext(lp,p); /* Skip the whole entry. */
            p = lpNext(lp,p); /* Skip the final lp-count field. */
        }

        /* Here we should perform garbage collection in case at this point
         * there are too many entries deleted inside the listpack. */
        entries -= to_delete;
        marked_deleted += to_delete;
        if (entries + marked_deleted > 10 && marked_deleted > entries/2) {
            /* TODO: perform a garbage collection. */
        }

        /* Update the listpack with the new pointer. */
        raxInsert(s->rax,ri.key,ri.key_len,lp,NULL);

        break; /* If we are here, there was enough to delete in the current
                  node, so no need to go to the next node. */
    }

    raxStop(&ri);
    return deleted;
}

/* Initialize the stream iterator, so that we can call iterating functions
 * to get the next items. This requires a corresponding streamIteratorStop()
 * at the end. The 'rev' parameter controls the direction. If it's zero the
 * iteration is from the start to the end element (inclusive), otherwise
 * if rev is non-zero, the iteration is reversed.
 *
 * Once the iterator is initalized, we iterate like this:
 *
 *  streamIterator myiterator;
 *  streamIteratorStart(&myiterator,...);
 *  int64_t numfields;
 *  while(streamIteratorGetID(&myiterator,&ID,&numfields)) {
 *      while(numfields--) {
 *          unsigned char *key, *value;
 *          size_t key_len, value_len;
 *          streamIteratorGetField(&myiterator,&key,&value,&key_len,&value_len);
 *
 *          ... do what you want with key and value ...
 *      }
 *  }
 *  streamIteratorStop(&myiterator); */
void streamIteratorStart(streamIterator *si, stream *s, streamID *start, streamID *end, int rev) {
    /* Intialize the iterator and translates the iteration start/stop
     * elements into a 128 big big-endian number. */
    if (start) {
        streamEncodeID(si->start_key,start);
    } else {
        si->start_key[0] = 0;
        si->start_key[0] = 0;
    }

    if (end) {
        streamEncodeID(si->end_key,end);
    } else {
        si->end_key[0] = UINT64_MAX;
        si->end_key[0] = UINT64_MAX;
    }

    /* Seek the correct node in the radix tree. */
    raxStart(&si->ri,s->rax);
    if (!rev) {
        if (start && (start->ms || start->seq)) {
            raxSeek(&si->ri,"<=",(unsigned char*)si->start_key,
                    sizeof(si->start_key));
            if (raxEOF(&si->ri)) raxSeek(&si->ri,"^",NULL,0);
        } else {
            raxSeek(&si->ri,"^",NULL,0);
        }
    } else {
        if (end && (end->ms || end->seq)) {
            raxSeek(&si->ri,"<=",(unsigned char*)si->end_key,
                    sizeof(si->end_key));
            if (raxEOF(&si->ri)) raxSeek(&si->ri,"$",NULL,0);
        } else {
            raxSeek(&si->ri,"$",NULL,0);
        }
    }
    si->stream = s;
    si->lp = NULL; /* There is no current listpack right now. */
    si->lp_ele = NULL; /* Current listpack cursor. */
    si->rev = rev;  /* Direction, if non-zero reversed, from end to start. */
}

/* Return 1 and store the current item ID at 'id' if there are still
 * elements within the iteration range, otherwise return 0 in order to
 * signal the iteration terminated. */
int streamIteratorGetID(streamIterator *si, streamID *id, int64_t *numfields) {
    while(1) { /* Will stop when element > stop_key or end of radix tree. */
        /* If the current listpack is set to NULL, this is the start of the
         * iteration or the previous listpack was completely iterated.
         * Go to the next node. */
        if (si->lp == NULL || si->lp_ele == NULL) {
            if (!si->rev && !raxNext(&si->ri)) return 0;
            else if (si->rev && !raxPrev(&si->ri)) return 0;
            serverAssert(si->ri.key_len == sizeof(streamID));
            /* Get the master ID. */
            streamDecodeID(si->ri.key,&si->master_id);
            /* Get the master fields count. */
            si->lp = si->ri.data;
            si->lp_ele = lpFirst(si->lp);           /* Seek items count */
            si->lp_ele = lpNext(si->lp,si->lp_ele); /* Seek deleted count. */
            si->lp_ele = lpNext(si->lp,si->lp_ele); /* Seek num fields. */
            si->master_fields_count = lpGetInteger(si->lp_ele);
            si->lp_ele = lpNext(si->lp,si->lp_ele); /* Seek first field. */
            si->master_fields_start = si->lp_ele;
            /* Skip master fileds to seek the first entry. */
            for (uint64_t i = 0; i < si->master_fields_count; i++)
                si->lp_ele = lpNext(si->lp,si->lp_ele);
            /* We are now pointing the zero term of the master entry. If
             * we are iterating in reverse order, we need to seek the
             * end of the listpack. */
            if (si->rev) si->lp_ele = lpLast(si->lp);
        } else if (si->rev) {
            /* If we are itereating in the reverse order, and this is not
             * the first entry emitted for this listpack, then we already
             * emitted the current entry, and have to go back to the previous
             * one. */
            int lp_count = lpGetInteger(si->lp_ele);
            while(lp_count--) si->lp_ele = lpPrev(si->lp,si->lp_ele);
            /* Seek lp-count of prev entry. */
            si->lp_ele = lpPrev(si->lp,si->lp_ele);
        }

        /* For every radix tree node, iterate the corresponding listpack,
         * returning elements when they are within range. */
        while(1) {
            if (!si->rev) {
                /* If we are going forward, skip the previous entry
                 * lp-count field (or in case of the master entry, the zero
                 * term field) */
                si->lp_ele = lpNext(si->lp,si->lp_ele);
                if (si->lp_ele == NULL) break;
            } else {
                /* If we are going backward, read the number of elements this
                 * entry is composed of, and jump backward N times to seek
                 * its start. */
                int64_t lp_count = lpGetInteger(si->lp_ele);
                if (lp_count == 0) { /* We reached the master entry. */
                    si->lp = NULL;
                    si->lp_ele = NULL;
                    break;
                }
                while(lp_count--) si->lp_ele = lpPrev(si->lp,si->lp_ele);
            }

            /* Get the flags entry. */
            si->lp_flags = si->lp_ele;
            int flags = lpGetInteger(si->lp_ele);
            si->lp_ele = lpNext(si->lp,si->lp_ele); /* Seek ID. */

            /* Get the ID: it is encoded as difference between the master
             * ID and this entry ID. */
            *id = si->master_id;
            id->ms += lpGetInteger(si->lp_ele);
            si->lp_ele = lpNext(si->lp,si->lp_ele);
            id->seq += lpGetInteger(si->lp_ele);
            si->lp_ele = lpNext(si->lp,si->lp_ele);
            unsigned char buf[sizeof(streamID)];
            streamEncodeID(buf,id);

            /* The number of entries is here or not depending on the
             * flags. */
            if (flags & STREAM_ITEM_FLAG_SAMEFIELDS) {
                *numfields = si->master_fields_count;
            } else {
                *numfields = lpGetInteger(si->lp_ele);
                si->lp_ele = lpNext(si->lp,si->lp_ele);
            }

            /* If current >= start, and the entry is not marked as
             * deleted, emit it. */
            if (!si->rev) {
                if (memcmp(buf,si->start_key,sizeof(streamID)) >= 0 &&
                    !(flags & STREAM_ITEM_FLAG_DELETED))
                {
                    if (memcmp(buf,si->end_key,sizeof(streamID)) > 0)
                        return 0; /* We are already out of range. */
                    si->entry_flags = flags;
                    if (flags & STREAM_ITEM_FLAG_SAMEFIELDS)
                        si->master_fields_ptr = si->master_fields_start;
                    return 1; /* Valid item returned. */
                }
            } else {
                if (memcmp(buf,si->end_key,sizeof(streamID)) <= 0 &&
                    !(flags & STREAM_ITEM_FLAG_DELETED))
                {
                    if (memcmp(buf,si->start_key,sizeof(streamID)) < 0)
                        return 0; /* We are already out of range. */
                    si->entry_flags = flags;
                    if (flags & STREAM_ITEM_FLAG_SAMEFIELDS)
                        si->master_fields_ptr = si->master_fields_start;
                    return 1; /* Valid item returned. */
                }
            }

            /* If we do not emit, we have to discard if we are going
             * forward, or seek the previous entry if we are going
             * backward. */
            if (!si->rev) {
                int64_t to_discard = (flags & STREAM_ITEM_FLAG_SAMEFIELDS) ?
                                     *numfields : *numfields*2;
                for (int64_t i = 0; i < to_discard; i++)
                    si->lp_ele = lpNext(si->lp,si->lp_ele);
            } else {
                int64_t prev_times = 4; /* flag + id ms + id seq + one more to
                                           go back to the previous entry "count"
                                           field. */
                /* If the entry was not flagged SAMEFIELD we also read the
                 * number of fields, so go back one more. */
                if (!(flags & STREAM_ITEM_FLAG_SAMEFIELDS)) prev_times++;
                while(prev_times--) si->lp_ele = lpPrev(si->lp,si->lp_ele);
            }
        }

        /* End of listpack reached. Try the next/prev radix tree node. */
    }
}

/* Get the field and value of the current item we are iterating. This should
 * be called immediately after streamIteratorGetID(), and for each field
 * according to the number of fields returned by streamIteratorGetID().
 * The function populates the field and value pointers and the corresponding
 * lengths by reference, that are valid until the next iterator call, assuming
 * no one touches the stream meanwhile. */
void streamIteratorGetField(streamIterator *si, unsigned char **fieldptr, unsigned char **valueptr, int64_t *fieldlen, int64_t *valuelen) {
    if (si->entry_flags & STREAM_ITEM_FLAG_SAMEFIELDS) {
        *fieldptr = lpGet(si->master_fields_ptr,fieldlen,si->field_buf);
        si->master_fields_ptr = lpNext(si->lp,si->master_fields_ptr);
    } else {
        *fieldptr = lpGet(si->lp_ele,fieldlen,si->field_buf);
        si->lp_ele = lpNext(si->lp,si->lp_ele);
    }
    *valueptr = lpGet(si->lp_ele,valuelen,si->value_buf);
    si->lp_ele = lpNext(si->lp,si->lp_ele);
}

/* Remove the current entry from the stream: can be called after the
 * GetID() API or after any GetField() call, however we need to iterate
 * a valid entry while calling this function. Moreover the function
 * requires the entry ID we are currently iterating, that was previously
 * returned by GetID().
 *
 * Note that after calling this function, next calls to GetField() can't
 * be performed: the entry is now deleted. Instead the iterator will
 * automatically re-seek to the next entry, so the caller should continue
 * with GetID(). */
void streamIteratorRemoveEntry(streamIterator *si, streamID *current) {
    unsigned char *lp = si->lp;
    int64_t aux;

    /* We do not really delete the entry here. Instead we mark it as
     * deleted flagging it, and also incrementing the count of the
     * deleted entries in the listpack header.
     *
     * We start flagging: */
    int flags = lpGetInteger(si->lp_flags);
    flags |= STREAM_ITEM_FLAG_DELETED;
    lp = lpReplaceInteger(lp,&si->lp_flags,flags);

    /* Change the valid/deleted entries count in the master entry. */
    unsigned char *p = lpFirst(lp);
    aux = lpGetInteger(p);
    lp = lpReplaceInteger(lp,&p,aux-1);
    p = lpNext(lp,p); /* Seek deleted field. */
    aux = lpGetInteger(p);
    lp = lpReplaceInteger(lp,&p,aux+1);

    /* Update the number of entries counter. */
    si->stream->length--;

    /* Re-seek the iterator to fix the now messed up state. */
    streamID start, end;
    if (si->rev) {
        streamDecodeID(si->start_key,&start);
        end = *current;
    } else {
        start = *current;
        streamDecodeID(si->end_key,&end);
    }
    streamIteratorStop(si);
    streamIteratorStart(si,si->stream,&start,&end,si->rev);

    /* TODO: perform a garbage collection here if the ration between
     * deleted and valid goes over a certain limit. */
}

/* Stop the stream iterator. The only cleanup we need is to free the rax
 * itereator, since the stream iterator itself is supposed to be stack
 * allocated. */
void streamIteratorStop(streamIterator *si) {
    raxStop(&si->ri);
}

/* Delete the specified item ID from the stream, returning 1 if the item
 * was deleted 0 otherwise (if it does not exist). */
int streamDeleteItem(stream *s, streamID *id) {
    int deleted = 0;
    streamIterator si;
    streamIteratorStart(&si,s,id,id,0);
    streamID myid;
    int64_t numfields;
    if (streamIteratorGetID(&si,&myid,&numfields)) {
        streamIteratorRemoveEntry(&si,&myid);
        deleted = 1;
    }
    return deleted;
}

/* Emit a reply in the client output buffer by formatting a Stream ID
 * in the standard <ms>-<seq> format, using the simple string protocol
 * of REPL. */
//void addReplyStreamID(client *c, streamID *id) {
//    sds replyid = sdscatfmt(sdsempty(),"+%U-%U\r\n",id->ms,id->seq);
//    addReplySds(c,replyid);
//}

/* Similar to the above function, but just creates an object, usually useful
 * for replication purposes to create arguments. */
//robj *createObjectFromStreamID(streamID *id) {
//    return createObject(OBJ_STRING, sdscatfmt(sdsempty(),"%U-%U",
//                                              id->ms,id->seq));
//}

/* -----------------------------------------------------------------------
 * Stream commands implementation
 * ----------------------------------------------------------------------- */

/* Helper function to convert a string to an unsigned long long value.
 * The function attempts to use the faster string2ll() function inside
 * Redis: if it fails, strtoull() is used instead. The function returns
 * 1 if the conversion happened successfully or 0 if the number is
 * invalid or out of range. */
int string2ull(const char *s, unsigned long long *value) {
    long long ll;
    if (string2ll(s,strlen(s),&ll)) {
        if (ll < 0) return 0; /* Negative values are out of range. */
        *value = ll;
        return 1;
    }
    errno = 0;
    char *endptr = NULL;
    *value = strtoull(s,&endptr,10);
    if (errno == EINVAL || errno == ERANGE || !(*s != '\0' && *endptr == '\0'))
        return 0; /* strtoull() failed. */
    return 1; /* Conversion done! */
}

/* -----------------------------------------------------------------------
 * Low level implementation of consumer groups
 * ----------------------------------------------------------------------- */

/* Create a NACK entry setting the delivery count to 1 and the delivery
 * time to the current time. The NACK consumer will be set to the one
 * specified as argument of the function. */
streamNACK *streamCreateNACK(streamConsumer *consumer) {
    streamNACK *nack = zmalloc(sizeof(*nack));
    nack->delivery_time = mstime();
    nack->delivery_count = 1;
    nack->consumer = consumer;
    return nack;
}

/* Free a NACK entry. */
void streamFreeNACK(streamNACK *na) {
    zfree(na);
}

/* Free a consumer and associated data structures. Note that this function
 * will not reassign the pending messages associated with this consumer
 * nor will delete them from the stream, so when this function is called
 * to delete a consumer, and not when the whole stream is destroyed, the caller
 * should do some work before. */
void streamFreeConsumer(streamConsumer *sc) {
    raxFree(sc->pel); /* No value free callback: the PEL entries are shared
                         between the consumer and the main stream PEL. */
    sdsfree(sc->name);
    zfree(sc);
}

/* Create a new consumer group in the context of the stream 's', having the
 * specified name and last server ID. If a consumer group with the same name
 * already existed NULL is returned, otherwise the pointer to the consumer
 * group is returned. */
streamCG *streamCreateCG(stream *s, char *name, size_t namelen, streamID *id) {
    if (s->cgroups == NULL) s->cgroups = raxNew();
    if (raxFind(s->cgroups,(unsigned char*)name,namelen) != raxNotFound)
        return NULL;

    streamCG *cg = zmalloc(sizeof(*cg));
    cg->pel = raxNew();
    cg->consumers = raxNew();
    cg->last_id = *id;
    raxInsert(s->cgroups,(unsigned char*)name,namelen,cg,NULL);
    return cg;
}

/* Free a consumer group and all its associated data. */
void streamFreeCG(streamCG *cg) {
    raxFreeWithCallback(cg->pel,(void(*)(void*))streamFreeNACK);
    raxFreeWithCallback(cg->consumers,(void(*)(void*))streamFreeConsumer);
    zfree(cg);
}

/* Lookup the consumer group in the specified stream and returns its
 * pointer, otherwise if there is no such group, NULL is returned. */
streamCG *streamLookupCG(stream *s, sds groupname) {
    if (s->cgroups == NULL) return NULL;
    streamCG *cg = raxFind(s->cgroups,(unsigned char*)groupname,
                           sdslen(groupname));
    return (cg == raxNotFound) ? NULL : cg;
}

/* Lookup the consumer with the specified name in the group 'cg': if the
 * consumer does not exist it is automatically created as a side effect
 * of calling this function, otherwise its last seen time is updated and
 * the existing consumer reference returned. */
streamConsumer *streamLookupConsumer(streamCG *cg, sds name, int create) {
    streamConsumer *consumer = raxFind(cg->consumers,(unsigned char*)name,
                                       sdslen(name));
    if (consumer == raxNotFound) {
        if (!create) return NULL;
        consumer = zmalloc(sizeof(*consumer));
        consumer->name = sdsdup(name);
        consumer->pel = raxNew();
        raxInsert(cg->consumers,(unsigned char*)name,sdslen(name),
                  consumer,NULL);
    }
    consumer->seen_time = mstime();
    return consumer;
}

/* Delete the consumer specified in the consumer group 'cg'. The consumer
 * may have pending messages: they are removed from the PEL, and the number
 * of pending messages "lost" is returned. */
uint64_t streamDelConsumer(streamCG *cg, sds name) {
    streamConsumer *consumer = streamLookupConsumer(cg,name,0);
    if (consumer == NULL) return 0;

    uint64_t retval = raxSize(consumer->pel);

    /* Iterate all the consumer pending messages, deleting every corresponding
     * entry from the global entry. */
    raxIterator ri;
    raxStart(&ri,consumer->pel);
    raxSeek(&ri,"^",NULL,0);
    while(raxNext(&ri)) {
        streamNACK *nack = ri.data;
        raxRemove(cg->pel,ri.key,ri.key_len,NULL);
        streamFreeNACK(nack);
    }
    raxStop(&ri);

    /* Deallocate the consumer. */
    raxRemove(cg->consumers,(unsigned char*)name,sdslen(name),NULL);
    streamFreeConsumer(consumer);
    return retval;
}

