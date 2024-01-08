#include <stdio.h>
#include <string>

#include "Platform.h"
#include "Util.cpp"

#include "melon-rs/src/melon/sys.rs.h"

using namespace melonDS;

// this file implements the interface defined by melonDS in Platform.h
// wherever possible, it uses the minimum amount of C++ code to invoke
// the Rust solution

namespace melonDS::Platform
{
    // initialize std::string with ::rust::String
    std::string InstanceFileSuffix()
    {
        return std::string(PlatformImpl::InstanceFileSuffix());
    }

    // FILE *OpenFile(std::string path, )
    // {
    //     FILE *f;
    //     // check if it exists, kind of???
    //     // if we were to use write mode, apparently this would automatically
    //     // create a file. So to just check, use read only mode.
    //     f = fopen(path.c_str(), "r");
    //     if (!f && mustexist)
    //     {
    //         return nullptr;
    //     }
    //     fclose(f);

    //     f = fopen(path.c_str(), mode.c_str());
    //     return f;
    // }

    // FILE *OpenLocalFile(std::string path, std::string mode)
    // {
    //     auto localPathRust = Rust::LocalizePath(path);
    //     std::string localPath(localPathRust);
    //     return OpenFile(localPath, mode, mode[0] != 'w');
    // }

    int InstanceID()
    {
        return PlatformImpl::InstanceID();
    }

    // synchronization primitives
    Thread *Thread_Create(std::function<void()> func)
    {
        Util::OpaqueFunction *opaque = new Util::OpaqueFunction;
        opaque->f = func;
        return PlatformImpl::Thread_Create(opaque);
    }
    void Thread_Free(Thread *thread)
    {
        return PlatformImpl::Thread_Free(thread);
    }
    void Thread_Wait(Thread *thread)
    {
        return PlatformImpl::Thread_Wait(thread);
    }

    Semaphore *Semaphore_Create()
    {
        return PlatformImpl::Semaphore_Create();
    }

    void Semaphore_Free(Semaphore *sema)
    {
        return PlatformImpl::Semaphore_Free(sema);
    }

    void Semaphore_Reset(Semaphore *sema)
    {
        return PlatformImpl::Semaphore_Reset(sema);
    }

    void Semaphore_Wait(Semaphore *sema)
    {
        return PlatformImpl::Semaphore_Wait(sema);
    }

    void Semaphore_Post(Semaphore *sema, int count)
    {
        return PlatformImpl::Semaphore_Post(sema, count);
    }

    Mutex *Mutex_Create()
    {
        return PlatformImpl::Mutex_Create();
    }
    void Mutex_Free(Mutex *mutex)
    {
        return PlatformImpl::Mutex_Free(mutex);
    }
    void Mutex_Lock(Mutex *mutex)
    {
        return PlatformImpl::Mutex_Lock(mutex);
    }
    void Mutex_Unlock(Mutex *mutex)
    {
        return PlatformImpl::Mutex_Unlock(mutex);
    }
    bool Mutex_TryLock(Mutex *mutex)
    {
        return PlatformImpl::Mutex_TryLock(mutex);
    }

    void WriteNDSSave(const u8 *savedata, u32 savelen, u32 writeoffset, u32 writelen)
    {
        return PlatformImpl::WriteNDSSave(savedata, savelen, writeoffset, writelen);
    }
    bool MP_Init()
    {
        return PlatformImpl::MP_Init();
    }
    void MP_DeInit()
    {
        return PlatformImpl::MP_DeInit();
    }
    void MP_Begin()
    {
        return PlatformImpl::MP_Begin();
    }
    void MP_End()
    {
        return PlatformImpl::MP_End();
    }
    int MP_SendPacket(u8 *data, int len, u64 timestamp)
    {
        return PlatformImpl::MP_SendPacket(data, len, timestamp);
    }
    int MP_RecvPacket(u8 *data, u64 *timestamp)
    {
        return PlatformImpl::MP_RecvPacket(data, timestamp);
    }
    int MP_SendCmd(u8 *data, int len, u64 timestamp)
    {
        return PlatformImpl::MP_SendCmd(data, len, timestamp);
    }
    int MP_SendReply(u8 *data, int len, u64 timestamp, u16 aid)
    {
        return PlatformImpl::MP_SendReply(data, len, timestamp, aid);
    }
    int MP_SendAck(u8 *data, int len, u64 timestamp)
    {
        return PlatformImpl::MP_SendAck(data, len, timestamp);
    }
    int MP_RecvHostPacket(u8 *data, u64 *timestamp)
    {
        return PlatformImpl::MP_RecvHostPacket(data, timestamp);
    }
    u16 MP_RecvReplies(u8 *data, u64 timestamp, u16 aidmask)
    {
        return PlatformImpl::MP_RecvReplies(data, timestamp, aidmask);
    }
    bool LAN_Init()
    {
        return PlatformImpl::LAN_Init();
    }
    void LAN_DeInit()
    {
        return PlatformImpl::LAN_DeInit();
    }
    int LAN_SendPacket(u8 *data, int len)
    {
        return PlatformImpl::LAN_SendPacket(data, len);
    }
    int LAN_RecvPacket(u8 *data)
    {
        return PlatformImpl::LAN_RecvPacket(data);
    }
    void Camera_Start(int num)
    {
        return PlatformImpl::Camera_Start(num);
    }
    void Camera_Stop(int num)
    {
        return PlatformImpl::Camera_Stop(num);
    }
    void Camera_CaptureFrame(int num, u32 *frame, int width, int height, bool yuv)
    {
        return PlatformImpl::Camera_CaptureFrame(num, frame, width, height, yuv);
    }
}
