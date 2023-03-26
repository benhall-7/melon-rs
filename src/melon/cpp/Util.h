#include "types.h"

namespace Util
{
    struct OpaqueFunction;

    void OpaqueFunction_Call(OpaqueFunction *func);
    void OpaqueFunction_Free(OpaqueFunction *func);

    bool Copy_Framebuffers(u32 *top, u32 *bottom);
}