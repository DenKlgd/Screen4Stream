#ifndef FRAME_H
#define FRAME_H

#include <QObject>

struct FrameHeader
{
    qint64 size;
    qint32 width;
    qint32 height;
    FrameHeader& operator= (const FrameHeader& other);
};

struct Frame
{
    char* data = nullptr;
    FrameHeader header;
    ~Frame();
};

#endif