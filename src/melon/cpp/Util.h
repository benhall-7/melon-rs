#include <memory>

#include "types.h"
#include "NDS.h"

#include "rust/cxx.h"

using namespace melonDS;

namespace Util
{
    struct OpaqueFunction;

    void OpaqueFunction_Call(OpaqueFunction *func);
    void OpaqueFunction_Free(OpaqueFunction *func);
}
