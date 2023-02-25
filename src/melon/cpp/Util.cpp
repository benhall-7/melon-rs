#include "Util.h"

namespace Util
{
    // https://stackoverflow.com/a/47063995/11423867
    struct OpaqueFunction
    {
        std::function<void()> f;
    };

    void OpaqueFunction_Call(OpaqueFunction *func)
    {
        func->f();
    }

    void OpaqueFunction_Free(OpaqueFunction *func)
    {
        delete func;
    }
}