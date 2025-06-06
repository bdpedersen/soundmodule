#ifndef __soundmodule_h
#define __soundmodule_h
#include <stdint.h>
#include <stddef.h>

#include "algoparam.h"

#ifdef __cplusplus
extern "C" {
#endif

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