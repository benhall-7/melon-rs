#include <stdio.h>
#include <string>

#include "Platform.h"
#include "Util.cpp"

#include "melon-rs/src/melon/sys.rs.h"

namespace Platform
{
    // void Init(int argc, char **argv);
    // void DeInit();
    void StopEmu()
    {
        return Glue::StopEmu();
    }
    int InstanceID()
    {
        return Glue::InstanceID();
    }
    // initialize std::string with ::rust::String
    std::string InstanceFileSuffix()
    {
        return std::string(Glue::InstanceFileSuffix());
    }
    int GetConfigInt(ConfigEntry entry)
    {
        return Glue::GetConfigInt(entry);
    }
    bool GetConfigBool(ConfigEntry entry)
    {
        return Glue::GetConfigBool(entry);
    }
    std::string GetConfigString(ConfigEntry entry)
    {
        return std::string(Glue::GetConfigString(entry));
    }

    // I might have been able to implement these two in Rust, but they presented
    // at least two challenges. The first being OpenFile has a default param. The
    // second is that both require returning FILE* pointers, which were tricky to
    // define in Rust, and could also be nullptr.
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

    Thread *Thread_Create(std::function<void()> func)
    {
        Util::OpaqueFunction *opaque = new Util::OpaqueFunction;
        opaque->f = func;
        return Glue::Thread_Create(opaque);
    }
    void Thread_Free(Thread *thread)
    {
        return Glue::Thread_Free(thread);
    }
    void Thread_Wait(Thread *thread)
    {
        return Glue::Thread_Wait(thread);
    }

    Mutex *Mutex_Create()
    {
        return Glue::Mutex_Create();
    }
    void Mutex_Free(Mutex *mutex)
    {
        return Glue::Mutex_Free(mutex);
    }
    void Mutex_Lock(Mutex *mutex)
    {
        return Glue::Mutex_Lock(mutex);
    }
    void Mutex_Unlock(Mutex *mutex)
    {
        return Glue::Mutex_Unlock(mutex);
    }
    bool Mutex_TryLock(Mutex *mutex)
    {
        return Glue::Mutex_TryLock(mutex);
    }

    void WriteNDSSave(const u8 *savedata, u32 savelen, u32 writeoffset, u32 writelen)
    {
        return Glue::WriteNDSSave(savedata, savelen, writeoffset, writelen);
    }
    bool MP_Init()
    {
        return Glue::MP_Init();
    }
    void MP_DeInit()
    {
        return Glue::MP_DeInit();
    }
    void MP_Begin()
    {
        return Glue::MP_Begin();
    }
    void MP_End()
    {
        return Glue::MP_End();
    }
    int MP_SendPacket(u8 *data, int len, u64 timestamp)
    {
        return Glue::MP_SendPacket(data, len, timestamp);
    }
    int MP_RecvPacket(u8 *data, u64 *timestamp)
    {
        return Glue::MP_RecvPacket(data, timestamp);
    }
    int MP_SendCmd(u8 *data, int len, u64 timestamp)
    {
        return Glue::MP_SendCmd(data, len, timestamp);
    }
    int MP_SendReply(u8 *data, int len, u64 timestamp, u16 aid)
    {
        return Glue::MP_SendReply(data, len, timestamp, aid);
    }
    int MP_SendAck(u8 *data, int len, u64 timestamp)
    {
        return Glue::MP_SendAck(data, len, timestamp);
    }
    int MP_RecvHostPacket(u8 *data, u64 *timestamp)
    {
        return Glue::MP_RecvHostPacket(data, timestamp);
    }
    u16 MP_RecvReplies(u8 *data, u64 timestamp, u16 aidmask)
    {
        return Glue::MP_RecvReplies(data, timestamp, aidmask);
    }
    bool LAN_Init()
    {
        return Glue::LAN_Init();
    }
    void LAN_DeInit()
    {
        return Glue::LAN_DeInit();
    }
    int LAN_SendPacket(u8 *data, int len)
    {
        return Glue::LAN_SendPacket(data, len);
    }
    int LAN_RecvPacket(u8 *data)
    {
        return Glue::LAN_RecvPacket(data);
    }
    void Camera_Start(int num)
    {
        return Glue::Camera_Start(num);
    }
    void Camera_Stop(int num)
    {
        return Glue::Camera_Stop(num);
    }
}
