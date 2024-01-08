#pragma once

#include <memory>

#include "types.h"
#include "NDS.h"
#include "Platform.h"

#include "rust/cxx.h"

using namespace melonDS;

namespace Util
{
    enum struct MelonFileMode : u8 {
        Read = Platform::FileMode::Read,
        Write = Platform::FileMode::Write,
        Preserve = Platform::FileMode::Preserve,
        NoCreate = Platform::FileMode::NoCreate,
        Text = Platform::FileMode::Text,
    };
    struct OpaqueFunction;

    void OpaqueFunction_Call(OpaqueFunction *func);
    void OpaqueFunction_Free(OpaqueFunction *func);
}
