#include <memory>

#include "rust/cxx.h"
#include "NDS.h"
#include "NDSCart.h"

using namespace melonDS;

namespace Shims
{
    std::unique_ptr<NDS> New_NDS();

    bool Copy_Framebuffers(const NDS &nds, u8 *dest, bool index);

    bool ReadSavestate(NDS &nds, const u8 *source, s32 len);
    bool WriteSavestate(NDS &nds, rust::Fn<void(const u8 *source, s32 len)> store);

    u32 CurrentFrame(const NDS &nds);

    const u8 *MainRAM(const NDS &nds);
    u8 *MainRAMMut(NDS &nds);
    u32 MainRAMMaxSize(const NDS &nds);

    void NDS_SetNDSCart(NDS &nds, std::unique_ptr<NDSCart::CartCommon> cart);

    // CartCommon

    std::unique_ptr<NDSCart::CartCommon> ParseROMWithSave(const u8 *romdata, u32 romlen, const u8 *savedata, u32 savelen);
}
