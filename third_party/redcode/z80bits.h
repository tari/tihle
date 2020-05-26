#ifndef _TIHLE_Z80BITS_H_
#define _TIHLE_Z80BITS_H_

#include <stddef.h>
#include <stdint.h>

#define Z_EMPTY
#define Z_OFFSET_OF offsetof
#define Z_UINT32(x) x
#define IS_BIG_ENDIAN (!*unsigned char *)&(uint16_t){1})
#define Z_BOP(type, base, offset) ((type)(((zuint8 *)(base)) + (offset)))

#define Z_8BIT_ROTATE_LEFT(value, rotation) \
	(((value) << (rotation)) | ((value) >> (8 - (rotation))))

#define Z_8BIT_ROTATE_RIGHT(value, rotation) \
	(((value) >> (rotation)) | ((value) << (8 - (rotation))))

#ifdef _MSC_VER
#define Z_INLINE __forceinline
#else
#define Z_INLINE __inline__ __attribute__((always_inline))
#endif

#define Z_C_SYMBOLS_BEGIN
#define Z_C_SYMBOLS_END

#define Z_DEFINE_STRICT_STRUCTURE_BEGIN typedef struct {
#define Z_DEFINE_STRICT_STRUCTURE_END	}

#define TRUE 1
#define FALSE 0

typedef uint8_t zboolean;
typedef int zsint;
typedef unsigned int zuint;
typedef size_t zusize;

typedef int8_t zsint8;
typedef int16_t zsint16;
typedef int32_t zsint32;

typedef uint8_t zuint8;
typedef uint16_t zuint16;
typedef uint32_t zuint32;

/*
 * Z{n}Bit is a union of n bits providing memberwise access to smaller values
 * within it.
 *
 * These are trimmed only to the necessary fields.
 */
typedef union {
    zuint16 value_uint16;

#ifdef IS_BIG_ENDIAN
    // 0 is the low-order byte of these aggregates
    struct {
        zuint8 index1;
        zuint8 index0;
    } values_uint8;
#else
    struct {
        zuint8 index0;
        zuint8 index1;
    } values_uint8;
#endif
} Z16Bit;

typedef union {
    zuint32 value_uint32;
    zuint8 array_uint8[4];
    zsint8 array_sint8[4];
} Z32Bit;

#include "Z/Z80.h"

#endif _TIHLE_Z80BITS_H_
