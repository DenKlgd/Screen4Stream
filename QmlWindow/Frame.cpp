#include "Frame.h"


FrameHeader& FrameHeader::operator= (const FrameHeader& other)
{
    size =  other.size;
    width = other.width;
    height = other.height;
    return *this;
}

Frame::~Frame()
{
    if (data != nullptr)
    {
        delete[] data;
        data = nullptr;
    }
}
