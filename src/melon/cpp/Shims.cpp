#include "Shims.h"

#include "GPU.h"
#include "NDS.h"
#include "types.h"
#include "NDSCart.h"

#include "rust/cxx.h"

using namespace melonDS;

namespace Shims
{
    std::unique_ptr<NDS> New_NDS()
    {
        auto nds = std::make_unique<NDS>();
        NDS::Current = nds.get();
        return nds;
    }

    bool Copy_Framebuffers(const NDS &nds, u8 *dest, bool index)
    {
        int ind = (int)index;
        int frontbuf = nds.GPU.FrontBuffer;

        auto screens = nds.GPU.Framebuffer[frontbuf];
        if (!screens[0] || !screens[1])
        {
            return false;
        }

        memcpy(dest, screens[ind].get(), 4 * 256 * 192);
        return true;
    }

    int SPU_ReadOutput(NDS &nds, s16 *data, int samples)
    {
        return nds.SPU.ReadOutput(data, samples);
    }

    bool ReadSavestate(NDS &nds, u8 *source, s32 len)
    {
        // note to self: "&source" is a pointer to a pointer, this screwed me up a bit
        Savestate state(source, len, false);
        return nds.DoSavestate(&state);
    }

    std::unique_ptr<std::vector<u8>> WriteSavestate(NDS &nds)
    {
        Savestate state;
        if (nds.DoSavestate(&state))
        {
            auto buffer = (u8 *)state.Buffer();
            auto len = state.Length();

            return std::make_unique<std::vector<u8>>(buffer, buffer + len);
        }
        return std::make_unique<std::vector<u8>>();
    }

    u32 CurrentFrame(const NDS &nds)
    {
        return nds.NumFrames;
    }

    const u8 *MainRAM(const NDS &nds)
    {
        return nds.MainRAM;
    }

    u8 *MainRAMMut(NDS &nds)
    {
        return nds.MainRAM;
    }

    u32 MainRAMMaxSize(const NDS &nds)
    {
        return nds.MainRAMMaxSize;
    }

    void NDS_SetupDirectBoot(NDS &nds, rust::string romname)
    {
        nds.SetupDirectBoot(std::string(romname));
    }

    void NDS_SetNDSCart(NDS &nds, std::unique_ptr<NDSCart::CartCommon> cart)
    {
        nds.SetNDSCart(std::move(cart));
    }

    // *Rust to C++*: Look what you need to mimic a fraction of my power!
    std::unique_ptr<NDSCart::CartCommon> ParseROMWithSave(const u8 *romdata, u32 romlen, const u8 *savedata, u32 savelen)
    {
        if (savedata == nullptr)
        {
            return NDSCart::ParseROM(romdata, romlen, nullptr, std::nullopt);
        }
        else
        {
            auto save = std::make_unique<u8[]>(savelen);
            memcpy(save.get(), savedata, savelen);

            NDSCart::NDSCartArgs cart_args{
                .SDCard = std::nullopt,
                .SRAM = std::move(save),
                .SRAMLength = savelen,
            };

            return NDSCart::ParseROM(romdata, romlen, nullptr, std::move(cart_args));
        }
    }

    void RTC_SetDateTime(NDS &nds, int year, int month, int day, int hour, int minute, int second)
    {
        nds.RTC.SetDateTime(year, month, day, hour, minute, second);
    }
}
