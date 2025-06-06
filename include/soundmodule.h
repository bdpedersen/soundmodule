#ifndef __soundmodule_h
#define __soundmodule_h
#include <stdint.h>
#include <stddef.h>

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

typedef struct {
    const char *key;   // Identifier for the parameter
    const char *name;  // Human-readable name
} AlgoCParamSet;

/// Sentinel value returned in *basekey when no further element is found
#define ALGOPARAM_KEY_NOT_FOUND ((uint64_t)(-1))

/// Get the name of the first parameter set below the given key.
/// If found, *basekey is updated and a string pointer is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and NULL is returned.
AlgoCParamSet algoparam_get_first_set(const AlgoParamSet *tree, uint64_t *basekey);

/// Get the name of the next parameter set after the given key.
/// If found, *basekey is updated and a string pointer is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and NULL is returned.
AlgoCParamSet algoparam_get_next_set(const AlgoParamSet *tree, uint64_t *basekey);

/// Get the first parameter under the given key.
/// If found, *basekey is updated and a filled AlgoCParam is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and the returned struct has NULL pointers and zeros.
AlgoCParam algoparam_get_first_param(const AlgoParamSet *tree, uint64_t *basekey);

/// Get the next parameter under the given key.
/// If found, *basekey is updated and a filled AlgoCParam is returned.
/// If not found, *basekey is set to ALGOPARAM_KEY_NOT_FOUND and the returned struct has NULL pointers and zeros.
AlgoCParam algoparam_get_next_param(const AlgoParamSet *tree, uint64_t *basekey);


/// @brief Initializes a SoundModule object
void soundmodule_init(void* self, int32_t fs);

/// @brief  Frees a SoundModule object
/// @param self 
void soundmodule_release(void* self);

/// @brief Gets a parameter tree pointer (non-owned)
/// @param self 
/// @return &self.param
void* soundmodule_get_params(void* self);

/// @brief Sends midi data to the sound module
/// @param self SoundModule
/// @param data midi data
/// @param len length of the midi data
/// @param timestamp Sample timestamp relative to block
void soundmodule_send_midi(void* self, uint8_t* data, size_t len, uint64_t timestamp);

/// @brief Sets a parameter in the module
/// @param self SoundModule
/// @param address Address of the parameter
/// @param value Value of the parameter
void soundmodule_set_parameter(void* self, uint64_t address, float value);

/// @brief Gets a parameter in the module
/// @param self SoundModule
/// @param address Address of the parameter
/// @return Value of the parameter
float soundmodule_get_parameter(void* self, uint64_t address);

/// @brief Core of the module
/// @param self Sound module
/// @param lo left output
/// @param ro right output
/// @param li left input
/// @param ri right input
/// @param blksiz blocksize for this call.
void soundmodule_run(void* self, float* lo, float* ro, const float* li, const float* ri, uint32_t blksiz);

#ifdef __cplusplus
}
#endif
#endif