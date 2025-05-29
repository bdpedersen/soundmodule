#ifndef ALGOPARAM_H
#define ALGOPARAM_H

#include <stdint.h>  // for uint64_t, int32_t
#include <stddef.h>  // for NULL
#ifdef __cplusplus
extern "C" {
#endif

// Opaque pointer to user-supplied AlgoParamSet tree
typedef void AlgoParamSet;

/// C representation of a parameter
typedef struct {
    const char *key;   // Identifier for the parameter
    const char *name;  // Human-readable name
    float min;         // Minimum value
    float max;         // Maximum value
    float defvalue;
    int32_t dtype;     // Unit or data type code
    const char ** dependents;
} AlgoCParam;

/// Sentinel value returned in *basekey when no further element is found
#define ALGOPARAM_KEY_NOT_FOUND ((uint64_t)(-1))

/// Get the name of the first parameter set below the given key.
/// If found, *basekey is updated and a string pointer is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and NULL is returned.
const char *algoparam_get_first_set(const AlgoParamSet *tree, uint64_t *basekey);

/// Get the name of the next parameter set after the given key.
/// If found, *basekey is updated and a string pointer is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and NULL is returned.
const char *algoparam_get_next_set(const AlgoParamSet *tree, uint64_t *basekey);

/// Get the first parameter under the given key.
/// If found, *basekey is updated and a filled AlgoCParam is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and the returned struct has NULL pointers and zeros.
AlgoCParam algoparam_get_first_param(const AlgoParamSet *tree, uint64_t *basekey);

/// Get the next parameter under the given key.
/// If found, *basekey is updated and a filled AlgoCParam is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and the returned struct has NULL pointers and zeros.
AlgoCParam algoparam_get_next_param(const AlgoParamSet *tree, uint64_t *basekey);

#ifdef __cplusplus
}
#endif

#endif // ALGOPARAM_H
