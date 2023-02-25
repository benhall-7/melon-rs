// extra types I need

namespace Util
{
    struct OpaqueFunction;

    void OpaqueFunction_Call(OpaqueFunction *func);
    void OpaqueFunction_Free(OpaqueFunction *func);
}