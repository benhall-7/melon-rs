#pragma once

// https://stackoverflow.com/a/617559/11423867

#include <time.h>

namespace Replacements
{
    int32_t EmulatedTime(int32_t *seconds);
}
// #define time(x) Replacements::EmulatedTime(x)
