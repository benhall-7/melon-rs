#include <stdio.h>
#include <string>
#include "melon-rs/src/melon/sys.rs.h"

// I might have been able to implement these all in Rust, but they presented
// at least two challenges. The first being OpenFile has a default param. The
// second is that both require returning FILE* pointers, which were tricky to
// define in Rust, and could also be nullptr.

namespace Platform
{
    FILE *OpenFile(std::string path, std::string mode, bool mustexist)
    {
        FILE *f;
        // check if it exists, kind of???
        // if we were to use write mode, apparently this would automatically
        // create a file. So to just check, use read only mode.
        f = fopen(path.c_str(), "r");
        if (!f && mustexist)
        {
            return nullptr;
        }
        fclose(f);

        f = fopen(path.c_str(), mode.c_str());
        return f;
    }

    FILE *OpenLocalFile(std::string path, std::string mode)
    {
        auto localPathRust = Rust::LocalizePath(path);
        std::string localPath(localPathRust);
        return OpenFile(localPath, mode, mode[0] != 'w');
    }

}
