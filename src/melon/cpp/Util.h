#include "types.h"

#include "rust/cxx.h"

namespace Util
{
    struct OpaqueFunction;

    void OpaqueFunction_Call(OpaqueFunction *func);
    void OpaqueFunction_Free(OpaqueFunction *func);

    bool Copy_Framebuffers(u8 *dest, bool index);

    void NDS_SetupDirectBoot(rust::String romname);
}