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

    bool Copy_Framebuffers(u8 *dest, bool index);

    // NDS

    std::unique_ptr<NDS> NDS_CreateUniquePtr();

    void NDS_SetupDirectBoot(rust::String romname);

    bool ReadSavestate(rust::String filename);
    bool WriteSavestate(rust::String filename);

    u32 CurrentFrame();

    u8* MainRAM();
    u32 MainRAMMaxSize();

    // End NDS
}