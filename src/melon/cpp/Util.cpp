#include "Util.h"

#include "GPU.h"
#include "NDS.h"
#include "types.h"

#include "rust/cxx.h"

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

    bool Copy_Framebuffers(u8 *dest, bool index)
    {
        int ind = (int)index;
        int frontbuf = GPU::FrontBuffer;

        auto screens = GPU::Framebuffer[frontbuf];
        if (!screens[0] || !screens[1])
        {
            return false;
        }

        memcpy(dest, screens[ind], 4 * 256 * 192);
        return true;
    }

    void NDS_SetupDirectBoot(rust::String romname)
    {
        NDS::SetupDirectBoot(std::string(romname));
    }

    bool ReadSavestate(rust::String filename)
    {
        Savestate state(std::string(filename), false);
        return NDS::DoSavestate(&state);
    }

    bool WriteSavestate(rust::String filename)
    {
        Savestate state(std::string(filename), true);
        return NDS::DoSavestate(&state);
    }

    u32 CurrentFrame()
    {
        return NDS::NumFrames;
    }
}