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

    FileHandle *OpenFile(const std::string &path, FileMode mode)
    {
        return PlatformImpl::OpenFile(path, mode);
    }
    FileHandle *OpenLocalFile(const std::string &path, FileMode mode)
    {
        return PlatformImpl::OpenLocalFile(path, mode);
    }

    bool FileExists(const std::string &name)
    {
        return PlatformImpl::FileExists(name);
    }
    bool LocalFileExists(const std::string &name)
    {
        return PlatformImpl::LocalFileExists(name);
    }

    bool CloseFile(FileHandle *file)
    {
        return PlatformImpl::CloseFile(file);
    }

    bool IsEndOfFile(FileHandle *file)
    {
        return PlatformImpl::IsEndOfFile(file);
    }

    bool FileReadLine(char *str, int count, FileHandle *file)
    {
        return PlatformImpl::FileReadLine((u8 *)str, count, file);
    }

    bool FileSeek(FileHandle *file, s64 offset, FileSeekOrigin origin)
    {
        return PlatformImpl::FileSeek(file, offset, origin);
    }

    void FileRewind(FileHandle *file)
    {
        return PlatformImpl::FileRewind(file);
    }

    u64 FileRead(void *data, u64 size, u64 count, FileHandle *file)
    {
        return PlatformImpl::FileRead((u8 *)data, size, count, file);
    }

    bool FileFlush(FileHandle *file)
    {
        return PlatformImpl::FileFlush(file);
    }

    u64 FileWrite(const void *data, u64 size, u64 count, FileHandle *file)
    {
        return PlatformImpl::FileWrite((u8 *)data, size, count, file);
    }

    u64 FileWriteFormatted(FileHandle *file, const char *fmt, ...)
    {
        if (fmt == nullptr)
            return 0;

        va_list args;
        va_start(args, fmt);
        // get required length
        int buf_len = std::snprintf(nullptr, 0, fmt, args);
        if (buf_len <= 0)
        {
            va_end(args);
            return 0;
        }
        auto buf = std::make_unique<char[]>(buf_len);
        // actually write into buffer
        snprintf(buf.get(), buf_len, fmt, args);
        va_end(args);
        return FileWrite((void *)buf.get(), 1, buf_len, file);
    }

    u64 FileLength(FileHandle *file)
    {
        return PlatformImpl::FileLength(file);
    }

    void SignalStop(StopReason reason) {
        // no op
    }
    void Log(LogLevel level, const char* fmt, ...) {
        if (fmt == nullptr)
            return;

        va_list args;
        va_start(args, fmt);
        vprintf(fmt, args);
        va_end(args);
    }
    void WriteFirmware(const Firmware& firmware, u32 writeoffset, u32 writelen) {
        // no op
    }
    void WriteDateTime(int year, int month, int day, int hour, int minute, int second) {
        // no op
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
